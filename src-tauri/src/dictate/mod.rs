//! Dictate with a hotkey (record 0002).
//!
//! Milestone 1: the hotkey and the pill. Double tapping the chosen modifier
//! opens a floating pill and plays the "device connected" sound; double tapping
//! again closes it and plays the "device disconnected" sound. The pill is
//! placed by Rust on the focused screen at the account's remembered spot and
//! can be dragged. Nothing happens at all unless somebody is signed in.
//!
//! Milestone 2: the microphone and the waveform. The hotkey now opens the
//! actual microphone before it opens the pill, and a loudness number goes to
//! the pill's level meter every 60 ms. Five minutes closes it on its own. If
//! the microphone will not open, the pill never appears and the cause is named
//! (AC-15). Audio is discarded as it arrives and nothing leaves the machine.
//!
//! Still to come: Deepgram, transcription and typing at the cursor (milestones
//! 3 and 4), history and settings (milestone 5). Two parts of milestone 2 are
//! deliberately unfinished and are recorded as such in
//! `docs/evidence/dictate-with-a-hotkey/`: AC-8's 30 second silence cap, whose
//! only named source is Deepgram's final results, and where a person reads
//! AC-15's message, which collides with the pill being unclickable by design.

mod hook;
mod limits;
mod machine;
mod microphone;
mod pill_mouse;
mod pill_window;
mod sound;
pub mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Listener, Manager, State};

use microphone::{MicError, Microphone};
use store::{Hotkey, Store};

/// The one SQLite file that holds everything EchoScribe persists (AGENTS.md
/// data rules). The sign-in feature opens it first; this feature opens its own
/// connection to the same file. The name is shared infrastructure and will move
/// to a shared module once a third feature needs it.
const DB_FILE: &str = "echoscribe.sqlite3";

/// The one thing that changes what this feature is doing. Everything arrives on
/// one channel and is handled by one thread, so two sources can never race to
/// open and close at the same moment.
enum Command {
    /// A clean double tap of the chosen modifier. Open if closed, close if open.
    Toggle,
    /// A cap ran out (record 0002 AC-8). Carries the reason for
    /// `dictation:closed`.
    CloseBecause(&'static str),
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
///
/// `listening` holds the open microphone, and holding it is what "listening"
/// means: there is no separate flag that could disagree with the device. `None`
/// is idle, and the microphone is shut.
pub struct Dictate {
    store: Store,
    listening: Mutex<Option<Microphone>>,
}

/// Open this feature's database connection, create the pill window, start the
/// keyboard-hook plumbing, and arm it if somebody is already signed in.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(Dictate {
        store,
        listening: Mutex::new(None),
    });

    pill_window::create(&handle)?;

    // One consumer thread handles every command, in the order they arrive.
    let (commands, command_rx) = mpsc::channel::<Command>();
    let consumer_handle = handle.clone();
    let consumer_commands = commands.clone();
    std::thread::Builder::new()
        .name("echoscribe-dictate-commands".into())
        .spawn(move || {
            while let Ok(command) = command_rx.recv() {
                match command {
                    Command::Toggle => on_double_tap(&consumer_handle, &consumer_commands),
                    Command::CloseBecause(reason) => close_if_listening(&consumer_handle, reason),
                }
            }
        })?;

    // Arm now if a session is already remembered, and follow sign-in and
    // sign-out from here on. Signing out disarms the hotkey (AC-16) and closes
    // the pill and the microphone if they were open.
    arm_if_signed_in(&handle, &commands);

    let signed_in_handle = handle.clone();
    let signed_in_commands = commands.clone();
    app.listen("auth:signed_in", move |_| {
        arm_if_signed_in(&signed_in_handle, &signed_in_commands);
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
fn arm_if_signed_in(app: &AppHandle, commands: &Sender<Command>) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let modifier = app
        .try_state::<Dictate>()
        .and_then(|state| state.store.setting_for(&account_id).ok())
        .map(|setting| modifier_of(setting.hotkey))
        .unwrap_or(hook::Modifier::Ctrl);

    hook::install(commands.clone());
    hook::arm(modifier);
}

fn modifier_of(hotkey: Hotkey) -> hook::Modifier {
    match hotkey {
        Hotkey::DoubleTapCtrl => hook::Modifier::Ctrl,
        Hotkey::DoubleTapAlt => hook::Modifier::Alt,
    }
}

/// The hotkey fired. Open the microphone and the pill if they are closed, close
/// them if they are open. Does nothing when nobody is signed in (record 0002
/// AC-16): the hook is disarmed in that case, and this is a second guard.
fn on_double_tap(app: &AppHandle, commands: &Sender<Command>) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return;
    };
    let setting = state.store.setting_for(&account_id).unwrap_or_default();

    let mut listening = state
        .listening
        .lock()
        .expect("dictate listening mutex poisoned");

    if let Some(mic) = listening.take() {
        // Shut the device before anything on screen changes, so the pill never
        // outlives the microphone in either direction.
        mic.stop();
        pill_window::close(app, "you_stopped_it");
        sound::play_close(setting.sounds_enabled);
        return;
    }

    // AC-15, and the "no silent listening" rule in AGENTS.md: the microphone is
    // opened first and the pill only goes up once it is genuinely capturing. A
    // pill that appeared first would be claiming the microphone was open before
    // anyone knew whether it was, and on a failure it would be a plain lie.
    match microphone::open(level_sink(app, commands)) {
        Ok(mic) => {
            *listening = Some(mic);
            pill_window::open(app, &setting);
            sound::play_open(setting.sounds_enabled);
        }
        Err(e) => {
            // No pill and no sound. Both of those mean "the microphone is on".
            report_microphone_error(app, e);
        }
    }
}

/// The callback the microphone calls every 60 ms with one loudness number.
///
/// It does two things and nothing else: send the number to the pill's level
/// meter, and check the caps. It never touches the listening state itself, so
/// it can never be holding a lock that the close it asks for needs.
fn level_sink(app: &AppHandle, commands: &Sender<Command>) -> Box<dyn Fn(f32) + Send> {
    let app = app.clone();
    let commands = commands.clone();
    let caps = limits::Deadlines::without_deepgram(Instant::now());
    // Ticks keep arriving until the close actually happens, so remember whether
    // one has already been asked for.
    let asked_to_close = AtomicBool::new(false);

    Box::new(move |level: f32| {
        let _ = app.emit_to(
            pill_window::LABEL,
            "dictation:level",
            json!({ "level": level }),
        );

        if let Some(expiry) = caps.expired(Instant::now()) {
            if !asked_to_close.swap(true, Ordering::SeqCst) {
                let _ = commands.send(Command::CloseBecause(expiry.reason()));
            }
        }
    })
}

/// Say why the microphone would not open (record 0002 AC-15).
///
/// Where a person reads this is not settled. The record wants the message to
/// carry a link straight to the Windows microphone privacy setting, and the
/// design system puts errors in the pill; but the pill deliberately takes no
/// mouse input outside its grip, so it has nothing that can be clicked. That
/// conflict is owed to `/architect`. Until it is settled the cause is named on
/// the event and printed, so nothing is silently swallowed.
fn report_microphone_error(app: &AppHandle, e: MicError) {
    eprintln!(
        "dictate: the microphone did not open: {} ({:?})",
        e.message(),
        e
    );
    let _ = app.emit(
        "dictation:error",
        json!({ "kind": e.kind(), "message": e.message() }),
    );
}

/// Close the microphone and the pill if they are open. Safe to call when they
/// are not: a second caller finds `None` and does nothing.
fn close_if_listening(app: &AppHandle, reason: &str) {
    let Some(state) = app.try_state::<Dictate>() else {
        return;
    };
    let mut listening = state
        .listening
        .lock()
        .expect("dictate listening mutex poisoned");
    let Some(mic) = listening.take() else {
        return;
    };

    mic.stop();
    pill_window::close(app, reason);

    let sounds_enabled = crate::sign_in::account_id_from(app)
        .and_then(|account_id| state.store.setting_for(&account_id).ok())
        .map(|setting| setting.sounds_enabled)
        .unwrap_or(true);
    sound::play_close(sounds_enabled);
}

/// What the current dictation is doing. Lets the pill recover after a reload.
#[tauri::command]
pub fn get_dictation_state(state: State<'_, Dictate>) -> DictationState {
    match *state
        .listening
        .lock()
        .expect("dictate listening mutex poisoned")
    {
        Some(_) => DictationState::Listening,
        None => DictationState::Idle,
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
