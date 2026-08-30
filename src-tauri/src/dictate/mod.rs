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
//! Step 2a: somewhere to read a microphone error (AC-28, AC-29, AC-31, settled
//! by the record's fourth amendment of 2026-08-30). When the microphone will
//! not open, the EchoScribe window is brought to the front carrying a short
//! code, one sentence of cause, and exactly one action. The pill is a sign,
//! never a control: it never appears when the microphone did not open, and no
//! error ever asks for a click on it.
//!
//! Step 2b: telling blocked-by-Windows apart for real (settled by the fifth
//! amendment of 2026-08-30). cpal never reports a privacy block on Windows, so
//! a failure with no named cause asks the Windows consent switches, read only,
//! in `consent.rs`.
//!
//! Still to come: Deepgram, transcription and typing at the cursor (milestones
//! 3 and 4), history and settings (milestone 5). AC-8's 30 second silence cap
//! is built and deliberately unarmed until milestone 4 arms it, because its
//! only named source is Deepgram's final results.

mod consent;
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
    /// A handle onto the command channel, so `retry_dictation` can build the
    /// same level sink the hotkey path builds. Behind a mutex only because a
    /// channel sender cannot be shared between threads without one.
    commands: Mutex<Sender<Command>>,
}

/// Open this feature's database connection, create the pill window, start the
/// keyboard-hook plumbing, and arm it if somebody is already signed in.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // One consumer thread handles every command, in the order they arrive.
    let (commands, command_rx) = mpsc::channel::<Command>();

    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(Dictate {
        store,
        listening: Mutex::new(None),
        commands: Mutex::new(commands.clone()),
    });

    pill_window::create(&handle)?;

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

    {
        let mut listening = state
            .listening
            .lock()
            .expect("dictate listening mutex poisoned");

        if let Some(mic) = listening.take() {
            // Shut the device before anything on screen changes, so the pill
            // never outlives the microphone in either direction.
            mic.stop();
            pill_window::close(app, "you_stopped_it");
            sound::play_close(setting.sounds_enabled);
            return;
        }
    }

    if let Err(e) = try_start(app, commands) {
        report_microphone_error(app, e);
    }
}

/// Open the microphone and put the pill up. The one way into dictation: the
/// hotkey lands here, and so does `retry_dictation` (record 0002 AC-29), so
/// they can never drift apart. Does nothing when already listening or when
/// nobody is signed in; both callers have their own guard for the latter.
///
/// AC-15, and the "no silent listening" rule in AGENTS.md: the microphone is
/// opened first and the pill only goes up once it is genuinely capturing. A
/// pill that appeared first would be claiming the microphone was open before
/// anyone knew whether it was, and on a failure it would be a plain lie. On
/// an error nothing is shown here at all: no pill and no sound, because both
/// of those mean "the microphone is on".
fn try_start(app: &AppHandle, commands: &Sender<Command>) -> Result<(), MicError> {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return Ok(());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Ok(());
    };
    let setting = state.store.setting_for(&account_id).unwrap_or_default();

    let mut listening = state
        .listening
        .lock()
        .expect("dictate listening mutex poisoned");
    if listening.is_some() {
        return Ok(());
    }

    // `consent::refine` is step 2b: a failure with no named cause asks the
    // Windows consent switches, read only, and becomes blocked-by-Windows if
    // any of the three says deny. It sits here so the hotkey and Try again
    // share it, the same as they share everything else on this path.
    let mic = microphone::open(level_sink(app, commands)).map_err(consent::refine)?;
    *listening = Some(mic);
    pill_window::open(app, &setting);
    sound::play_open(setting.sounds_enabled);
    Ok(())
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

/// Say why the microphone would not open, where a person can actually read it
/// (record 0002 AC-15, AC-28, settled by the fourth amendment of 2026-08-30).
///
/// The pill is a sign, never a control, and it never appears when the
/// microphone did not open. So the message is read in the EchoScribe window,
/// which is brought to the front carrying the code, the sentence and the one
/// action. Bringing it forward moves focus out of whatever the person was in;
/// the record accepts that cost because this only ever answers a hotkey they
/// just pressed that produced nothing.
fn report_microphone_error(app: &AppHandle, e: MicError) {
    eprintln!(
        "dictate: the microphone did not open: {} ({:?})",
        e.message(),
        e
    );
    // The event goes to every window, so the main window receives it on the
    // capability it already has. Emitted before the window comes forward so
    // the screen is mounting as it arrives.
    let _ = app.emit(
        "dictation:error",
        json!({ "kind": e.kind(), "message": e.message() }),
    );
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// The Windows microphone privacy page, as a fixed literal. The interface can
/// ask for this one page and no other; nothing is ever built from anything
/// (record 0002, value sourcing).
const MIC_PRIVACY_PAGE: windows::core::PCWSTR = windows::core::w!("ms-settings:privacy-microphone");

/// What a failed `retry_dictation` hands back to the interface: the same named
/// kind and fixed sentence that ride on `dictation:error`, and nothing else.
#[derive(Debug, Clone, Serialize)]
pub struct DictationErrorPayload {
    kind: &'static str,
    message: &'static str,
}

impl From<MicError> for DictationErrorPayload {
    fn from(e: MicError) -> Self {
        Self {
            kind: e.kind(),
            message: e.message(),
        }
    }
}

/// Open the Windows microphone privacy page. The one action on a
/// `microphone_blocked_by_windows` error (record 0002 AC-29). Takes nothing,
/// returns nothing, and refuses when nobody is signed in, like every command
/// on this surface.
#[tauri::command]
pub async fn open_microphone_privacy_settings(app: AppHandle) -> Result<(), &'static str> {
    if crate::sign_in::account_id_from(&app).is_none() {
        return Err("not_signed_in");
    }
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let launched = unsafe {
        ShellExecuteW(
            None,
            windows::core::w!("open"),
            MIC_PRIVACY_PAGE,
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW reports success as a value above 32.
    if launched.0 as isize <= 32 {
        eprintln!("dictate: Windows would not open the microphone privacy page");
        return Err("could_not_open_settings");
    }
    Ok(())
}

/// Try again: the one action on the other three microphone errors (record 0002
/// AC-29). Goes through the same path the hotkey does, so there is only ever
/// one way into dictation. Starts dictation there and then if the microphone
/// now opens, or comes back with the same named error if it does not.
#[tauri::command]
pub async fn retry_dictation(app: AppHandle) -> Result<(), DictationErrorPayload> {
    if crate::sign_in::account_id_from(&app).is_none() {
        return Err(DictationErrorPayload {
            kind: "not_signed_in",
            message: "",
        });
    }
    let commands = {
        let Some(state) = app.try_state::<Dictate>() else {
            return Err(DictationErrorPayload::from(MicError::Unavailable));
        };
        let sender = state
            .commands
            .lock()
            .expect("dictate commands mutex poisoned")
            .clone();
        sender
    };
    try_start(&app, &commands).map_err(DictationErrorPayload::from)
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
