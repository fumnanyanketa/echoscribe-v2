//! Keeping a session alive without the person doing anything.
//!
//! One background thread, started once at launch. It does the first check
//! immediately, which is what turns "signed in from last time" into "Clerk
//! agrees you are signed in", and then keeps looking in on the session so the
//! access token is renewed before it expires
//! (docs/decisions/0003-sign-in.md, build plan step 2).
//!
//! This thread is also the only thing that answers whether the app is online.
//! There is no separate network probe: the refresh either succeeds, is refused,
//! or cannot be delivered, and those three outcomes are exactly the three
//! things the app needs to know.
//!
//! It touches no token except to hand one to `clerk` and put the replacement
//! back in the credential store. Nothing it emits carries a token.

use std::sync::atomic::Ordering;
use std::time::Duration;

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use super::auth_state::AccountView;
use super::clerk::{self, RefreshError};
use super::clock;
use super::config;
use super::credentials;
use super::SignIn;

/// Start the upkeep thread. Called once, from `init`.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || loop {
        tick(&app);
        std::thread::sleep(Duration::from_secs(config::UPKEEP_INTERVAL_SECS));
    });
}

/// One look at the session. Does nothing at all when nobody is signed in, or
/// when the access token is still good and Clerk has already confirmed the
/// session this launch.
fn tick(app: &AppHandle) {
    let Some(sign_in) = app.try_state::<SignIn>() else {
        return;
    };

    let session = match sign_in.store.read_session() {
        Ok(Some(session)) if !session.account_id.is_empty() => session,
        // Signed out, or the row is unreadable. Either way there is nothing to
        // keep alive.
        _ => return,
    };
    if session.credential_target.is_empty() {
        return;
    }

    let confirmed = sign_in.refresh_confirmed.load(Ordering::SeqCst);

    let tokens = match credentials::load(&session.credential_target) {
        Ok(Some(tokens)) => tokens,
        // The session row says signed in but the tokens are gone from the
        // credential store. We cannot prove the session any more and cannot get
        // it back, so end it rather than pretending.
        Ok(None) => {
            eprintln!("refresh: the session row has no matching credential entry");
            end_session(app, &sign_in);
            return;
        }
        Err(e) => {
            eprintln!("refresh: could not read the credential store: {e}");
            return;
        }
    };

    let expiring = tokens.access_expires_at - clock::now_unix() <= config::RENEW_MARGIN_SECS;
    if confirmed && !expiring {
        return;
    }

    match clerk::refresh_tokens(&tokens.refresh_token) {
        Ok(fresh) => {
            if let Err(e) = credentials::store(&session.credential_target, &fresh) {
                // Clerk is happy; we just could not keep the new tokens. Leave
                // the state alone and try again on the next look.
                eprintln!("refresh: could not store the renewed session: {e}");
                return;
            }
            if let Err(e) = sign_in.store.mark_verified(&clock::now_iso8601()) {
                eprintln!("refresh: could not record the verification time: {e}");
            }
            // Only the first confirmation of a launch changes what is on
            // screen. Later renewals are silent, which is the point of them.
            if !sign_in.refresh_confirmed.swap(true, Ordering::SeqCst) {
                sign_in.offline_announced.store(false, Ordering::SeqCst);
                announce_signed_in(app, &sign_in, &session.account_id);
            }
        }
        // Clerk answered and said no: signed out on another device, or revoked
        // (AC-16).
        Err(RefreshError::Refused) => end_session(app, &sign_in),
        // Never reached Clerk. The session is untouched and the person keeps
        // working from their stored identity (AC-14).
        Err(RefreshError::Unreachable) => {
            if !confirmed && !sign_in.offline_announced.swap(true, Ordering::SeqCst) {
                let _ = app.emit("auth:offline", json!({}));
            }
        }
    }
}

/// End a session that Clerk will no longer honour. Local only: there is nothing
/// left to revoke, and the account row and everything it owns stay put.
fn end_session(app: &AppHandle, sign_in: &SignIn) {
    if let Err(e) = super::clear_local_session(sign_in) {
        eprintln!("refresh: could not clear the ended session: {e}");
        return;
    }
    let _ = app.emit(
        "auth:signed_out",
        json!({ "reason": "session_ended_elsewhere" }),
    );
}

/// Tell the interface the app is fully signed in now, not working offline.
fn announce_signed_in(app: &AppHandle, sign_in: &SignIn, account_id: &str) {
    match sign_in.store.read_account(account_id) {
        Ok(Some(account)) => {
            let _ = app.emit("auth:signed_in", AccountView::from_account(&account));
        }
        Ok(None) => eprintln!("refresh: the session points at an account row that is not there"),
        Err(e) => eprintln!("refresh: could not read the account row: {e}"),
    }
}
