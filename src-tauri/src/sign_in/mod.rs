//! Sign in.
//!
//! Gives every other feature a person to attach data to. On this machine the
//! `account` and `session` tables say who has signed in and the OS credential
//! store holds their tokens. The interface only ever asks Rust what to draw
//! and asks it to start, cancel or end a sign-in. No token or Clerk code ever
//! crosses into the interface. See docs/decisions/0003-sign-in.md.

pub mod auth_state;
pub mod clerk;
pub mod clock;
pub mod config;
pub mod credentials;
pub mod handoff;
pub mod renewal;
pub mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};

use auth_state::{AccountView, AuthState};
use clerk::SignInError;
use credentials::SessionTokens;
use handoff::{Callback, Handoff};
use store::Store;

/// The one SQLite file that holds everything EchoScribe persists.
const DB_FILE: &str = "echoscribe.sqlite3";

/// A sign-in in flight. Present only while the browser handoff is running.
struct PendingSignIn {
    cancel: Arc<AtomicBool>,
}

/// Everything the sign-in feature keeps for the running app. Managed by Tauri.
/// Holds no token: tokens live only in the OS credential store.
pub struct SignIn {
    store: Store,
    pending: Mutex<Option<PendingSignIn>>,
    /// True once Clerk has confirmed the session at least once this launch.
    refresh_confirmed: AtomicBool,
    /// True once the interface has been told this launch is working offline, so
    /// a connection that stays down does not repeat the event every minute.
    offline_announced: AtomicBool,
}

impl SignIn {
    fn current_state(&self) -> Result<AuthState, String> {
        let signing_in = self
            .pending
            .lock()
            .map_err(|_| "sign-in state is unavailable".to_string())?
            .is_some();
        let confirmed = self.refresh_confirmed.load(Ordering::SeqCst);
        auth_state::compute(&self.store, signing_in, confirmed).map_err(|e| e.to_string())
    }

    /// Reset what this launch believes about Clerk. Used whenever the session
    /// changes underneath us.
    fn forget_clerk_confirmation(&self) {
        self.refresh_confirmed.store(false, Ordering::SeqCst);
        self.offline_announced.store(false, Ordering::SeqCst);
    }

    fn clear_pending(&self) {
        if let Ok(mut pending) = self.pending.lock() {
            *pending = None;
        }
    }
}

/// Open the database in the app data directory, run migrations, register state.
/// Called once from the Tauri setup hook.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(SignIn {
        store,
        pending: Mutex::new(None),
        refresh_confirmed: AtomicBool::new(false),
        offline_announced: AtomicBool::new(false),
    });
    // From here on, one background thread confirms and renews the session. Its
    // first check is what turns a remembered sign-in into a confirmed one.
    renewal::start(app.handle().clone());
    Ok(())
}

/// Empty the session row and delete its credential entry. Local only: it never
/// touches the network, so it works with no connection at all (AC-15), and it
/// deletes no account row and no account-owned data (AC-9).
///
/// Returns the tokens it removed, when there were any, so a deliberate sign-out
/// can go on to revoke them at Clerk.
fn clear_local_session(sign_in: &SignIn) -> Result<Option<SessionTokens>, String> {
    let mut removed = None;
    if let Some(session) = sign_in.store.read_session().map_err(|e| e.to_string())? {
        if !session.credential_target.is_empty() {
            removed = credentials::load(&session.credential_target).unwrap_or(None);
            let _ = credentials::delete(&session.credential_target);
        }
    }
    sign_in.store.clear_session().map_err(|e| e.to_string())?;
    sign_in.forget_clerk_confirmation();
    Ok(removed)
}

/// What screen to draw. Called on load and after a reload.
#[tauri::command]
pub fn get_auth_state(state: State<'_, SignIn>) -> Result<AuthState, String> {
    state.current_state()
}

/// The signed-in account id, or `None` when nobody is signed in on this
/// machine. Other features read this to attach their data to a person and to
/// gate behaviour that must do nothing when signed out (record 0002 AC-16).
/// This is the one thing the sign-in feature exposes to the rest of the app;
/// it will move to a shared session module once a third feature needs it.
pub fn account_id_from(app: &AppHandle) -> Option<String> {
    let sign_in = app.try_state::<SignIn>()?;
    let session = sign_in.store.read_session().ok().flatten()?;
    if session.account_id.is_empty() {
        None
    } else {
        Some(session.account_id)
    }
}

/// Begin a sign-in: check that Clerk answers, then make the PKCE and state
/// values, bind the loopback listener, and open the browser. Returns once the
/// browser is open. The rest of the flow runs on a background thread and ends
/// with an `auth:` event.
///
/// Signing in always needs the internet. Rather than open a browser onto its
/// own error page and leave the person staring at a waiting screen for five
/// minutes, this asks Clerk first and stops here if it does not answer. On that
/// path nothing is stored, no port is bound and no browser opens
/// (docs/decisions/0003-sign-in.md AC-15). It can take up to five seconds to
/// return.
#[tauri::command]
pub fn start_sign_in(app: AppHandle, state: State<'_, SignIn>) -> Result<(), String> {
    if !matches!(state.current_state()?, AuthState::SignedOut) {
        return Err("a sign-in is already underway or you are already signed in".to_string());
    }

    if !clerk::clerk_answers() {
        return Err(SignInError::CannotReachClerk.code().to_string());
    }

    let handoff = handoff::start().map_err(|e| e.code().to_string())?;
    let cancel = Arc::new(AtomicBool::new(false));
    *state
        .pending
        .lock()
        .map_err(|_| "sign-in state is unavailable".to_string())? = Some(PendingSignIn {
        cancel: cancel.clone(),
    });

    let app = app.clone();
    std::thread::spawn(move || run_handoff(app, handoff, cancel));
    Ok(())
}

/// Abandon an in-progress sign-in and shut the listener. The waiting screen's
/// cancel action.
#[tauri::command]
pub fn cancel_sign_in(state: State<'_, SignIn>) -> Result<(), String> {
    if let Some(pending) = state
        .pending
        .lock()
        .map_err(|_| "sign-in state is unavailable".to_string())?
        .as_ref()
    {
        pending.cancel.store(true, Ordering::SeqCst);
    }
    Ok(())
}

/// Sign out. Empties the session and deletes the credential entry first, so
/// signing out always works even with no network, then tells Clerk to revoke
/// the refresh token on a background thread. Never deletes an account row or
/// any account-owned data.
#[tauri::command]
pub fn sign_out(app: AppHandle, state: State<'_, SignIn>) -> Result<(), String> {
    let removed = clear_local_session(&state)?;
    let _ = app.emit("auth:signed_out", json!({ "reason": "you_signed_out" }));

    // Best effort, and off the command's thread: the person is already signed
    // out and nothing on screen is waiting for Clerk to answer.
    if let Some(tokens) = removed {
        std::thread::spawn(move || clerk::revoke_refresh_token(&tokens.refresh_token));
    }
    Ok(())
}

/// The background half of a sign-in: wait for the callback, exchange the code,
/// store the result, and emit exactly one outcome event.
fn run_handoff(app: AppHandle, handoff: Handoff, cancel: Arc<AtomicBool>) {
    let outcome = handoff::wait_for_callback(&handoff, &cancel);

    let result = match outcome {
        Callback::Code(code) => finish_sign_in(&app, &code, &handoff),
        Callback::Abandoned => Err(SignInError::BrowserClosed),
        Callback::TimedOut => {
            eprintln!("sign-in: no callback arrived within the timeout");
            Err(SignInError::TimedOut)
        }
        Callback::StateMismatch => {
            eprintln!("sign-in: the callback's state value did not match this attempt");
            Err(SignInError::SecurityCheckFailed)
        }
        Callback::ClerkError(oauth_error) => {
            match &oauth_error {
                Some(code) => {
                    eprintln!("sign-in: Clerk redirected back with error={code}")
                }
                None => eprintln!("sign-in: the callback had neither a code nor an error"),
            }
            Err(SignInError::ClerkRejected)
        }
    };

    if let Some(sign_in) = app.try_state::<SignIn>() {
        sign_in.clear_pending();
    }

    match result {
        Ok(view) => {
            let _ = app.emit("auth:signed_in", view);
        }
        Err(reason) => {
            // A deliberate cancel is silent: the interface already went back to
            // the initial screen.
            if !cancel.load(Ordering::SeqCst) {
                let _ = app.emit("auth:sign_in_failed", json!({ "code": reason.code() }));
            }
        }
    }
}

fn finish_sign_in(
    app: &AppHandle,
    code: &str,
    handoff: &Handoff,
) -> Result<AccountView, SignInError> {
    let tokens = clerk::exchange_code(code, &handoff.pkce_verifier, &handoff.redirect_uri)?;
    let account = clerk::fetch_account(&tokens.access_token)?;

    let sign_in = app
        .try_state::<SignIn>()
        .ok_or(SignInError::ClerkRejected)?;
    let now = clock::now_iso8601();

    // Past this point Clerk was fine; a failure is local storage. "Try again"
    // (ClerkRejected) is still the right next step for the person, but the
    // diagnostic says storage so we are not chasing Clerk.
    sign_in
        .store
        .upsert_account(&account.id, &account.email, &account.name, &now)
        .map_err(|e| {
            eprintln!("sign-in: could not write the account row: {e}");
            SignInError::ClerkRejected
        })?;

    let target = credentials::target_for(&account.id);
    credentials::store(&target, &tokens).map_err(|e| {
        eprintln!("sign-in: could not store the session tokens: {e}");
        SignInError::ClerkRejected
    })?;

    sign_in
        .store
        .set_session(&account.id, &target, &now)
        .map_err(|e| {
            eprintln!("sign-in: could not write the session row: {e}");
            SignInError::ClerkRejected
        })?;

    sign_in.refresh_confirmed.store(true, Ordering::SeqCst);
    sign_in.offline_announced.store(false, Ordering::SeqCst);

    let stored = sign_in
        .store
        .read_account(&account.id)
        .map_err(|_| SignInError::ClerkRejected)?
        .ok_or(SignInError::ClerkRejected)?;
    Ok(AccountView::from_account(&stored))
}
