//! Dictate with a hotkey (record 0002).
//!
//! Milestone 1: the hotkey and the pill. Double tapping the chosen modifier
//! opens a floating pill and plays the "device connected" sound; double tapping
//! again closes it and plays the "device disconnected" sound. The pill is
//! placed by Rust on the focused screen at the account's remembered spot and
//! can be dragged. Nothing happens at all unless somebody is signed in. There
//! is no microphone, no network and no transcription yet; those are later
//! milestones.

mod hook;
mod machine;
mod pill_mouse;
mod pill_window;
mod sound;
pub mod store;

use std::sync::mpsc;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Listener, Manager, State};

use store::{Hotkey, Store};

/// The one SQLite file that holds everything EchoScribe persists (AGENTS.md
/// data rules). The sign-in feature opens it first; this feature opens its own
/// connection to the same file. The name is shared infrastructure and will move
/// to a shared module once a third feature needs it.
const DB_FILE: &str = "echoscribe.sqlite3";

/// Whether the microphone (and so the pill) is currently open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Session {
    Idle,
    Listening,
}

/// What the interface may ask about the current dictation. Serialised with a
/// `state` tag, matching the sign-in feature's shape.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DictationState {
    Idle,
    Listening,
}

/// Everything this feature keeps for the running app. Managed by Tauri.
pub struct Dictate {
    store: Store,
    session: Mutex<Session>,
}

/// Open this feature's database connection, create the pill window, start the
/// keyboard-hook plumbing, and arm it if somebody is already signed in.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(Dictate {
        store,
        session: Mutex::new(Session::Idle),
    });

    pill_window::create(&handle)?;

    // One consumer thread turns "a double tap completed" into open-or-close.
    let (toggle_tx, toggle_rx) = mpsc::channel::<()>();
    let consumer_handle = handle.clone();
    std::thread::Builder::new()
        .name("echoscribe-dictate-toggle".into())
        .spawn(move || {
            while toggle_rx.recv().is_ok() {
                on_double_tap(&consumer_handle);
            }
        })?;

    // Arm now if a session is already remembered, and follow sign-in and
    // sign-out from here on. Signing out disarms the hotkey (AC-16) and closes
    // the pill if it was open.
    arm_if_signed_in(&handle, &toggle_tx);

    let signed_in_handle = handle.clone();
    let signed_in_tx = toggle_tx.clone();
    app.listen("auth:signed_in", move |_| {
        arm_if_signed_in(&signed_in_handle, &signed_in_tx);
    });

    let signed_out_handle = handle.clone();
    app.listen("auth:signed_out", move |_| {
        hook::disarm();
        close_if_listening(&signed_out_handle, "you_signed_out");
    });

    Ok(())
}

/// Install the hook (once) and arm it for the current account's chosen
/// modifier, but only if somebody is signed in.
fn arm_if_signed_in(app: &AppHandle, toggle_tx: &mpsc::Sender<()>) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let modifier = app
        .try_state::<Dictate>()
        .and_then(|state| state.store.setting_for(&account_id).ok())
        .map(|setting| modifier_of(setting.hotkey))
        .unwrap_or(hook::Modifier::Ctrl);

    hook::install(toggle_tx.clone());
    hook::arm(modifier);
}

fn modifier_of(hotkey: Hotkey) -> hook::Modifier {
    match hotkey {
        Hotkey::DoubleTapCtrl => hook::Modifier::Ctrl,
        Hotkey::DoubleTapAlt => hook::Modifier::Alt,
    }
}

/// The hotkey fired. Open the pill if it is closed, close it if it is open.
/// Does nothing when nobody is signed in (record 0002 AC-16): the hook is
/// disarmed in that case, and this is a second guard.
fn on_double_tap(app: &AppHandle) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return;
    };
    let setting = state.store.setting_for(&account_id).unwrap_or_default();

    let mut session = state
        .session
        .lock()
        .expect("dictate session mutex poisoned");
    match *session {
        Session::Idle => {
            pill_window::open(app, &setting);
            sound::play_open(setting.sounds_enabled);
            *session = Session::Listening;
        }
        Session::Listening => {
            pill_window::close(app, "you_stopped_it");
            sound::play_close(setting.sounds_enabled);
            *session = Session::Idle;
        }
    }
}

fn close_if_listening(app: &AppHandle, reason: &str) {
    if let Some(state) = app.try_state::<Dictate>() {
        let mut session = state
            .session
            .lock()
            .expect("dictate session mutex poisoned");
        if *session == Session::Listening {
            pill_window::close(app, reason);
            *session = Session::Idle;
        }
    }
}

/// What the current dictation is doing. Lets the pill recover after a reload.
#[tauri::command]
pub fn get_dictation_state(state: State<'_, Dictate>) -> DictationState {
    match *state
        .session
        .lock()
        .expect("dictate session mutex poisoned")
    {
        Session::Idle => DictationState::Idle,
        Session::Listening => DictationState::Listening,
    }
}

/// The person let go of the pill's grip. Read where it ended up, convert to
/// fractions of that screen's working area, and remember it for next time
/// (AC-24). Called by `pill_mouse` when its own drag ends; the interface is not
/// involved and never sees or sends a coordinate.
pub(super) fn remember_pill_spot(app: &AppHandle) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return;
    };
    let Some((x, y)) = pill_window::spot_after_drag(app) else {
        return;
    };
    let now = crate::sign_in::clock::now_iso8601();
    if let Err(e) = state.store.save_pill_spot(&account_id, x, y, &now) {
        eprintln!("dictate: could not remember where the pill was left: {e}");
    }
}
