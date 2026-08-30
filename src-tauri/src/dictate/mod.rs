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
//! Milestone 3: the Deepgram key (AC-9 to AC-13). The hotkey now checks that a
//! key is saved before it touches the microphone. With no key the microphone
//! does not open, no pill appears, neither sound plays, and the EchoScribe
//! window comes forward with the guided setup screen instead. Checking a pasted
//! key against Deepgram, storing it in Windows Credential Manager and the fixed
//! wording for every key error all live in `deepgram_key.rs`.
//!
//! Step 3a: clearing an error the person has already fixed (AC-32, settled by
//! the sixth amendment of 2026-08-30). An error screen is a claim that
//! dictation cannot happen, and the person can make that claim untrue without
//! touching EchoScribe at all. So each error clears on its own proof that the
//! thing it named now works: the four microphone kinds on `dictation:opened`,
//! which is why that event is broadcast rather than sent to the pill alone. It
//! clears quietly, so nothing on this path brings the window forward.
//!
//! Milestone 4: transcription and typing at the cursor (AC-3, AC-4, AC-7,
//! AC-8's silence half, AC-20). The hotkey now reads the account's Deepgram key,
//! opens the microphone, and streams it to Deepgram over `transcribe.rs`.
//! Wording Deepgram marks as final is typed at whatever cursor has focus, by
//! `typing.rs`, which refuses a password field. Nothing unfinished is ever
//! typed, so no text is ever taken back. AC-8's silence cap is armed here,
//! because Deepgram's final results are the first thing on this machine that can
//! honestly say a person spoke.
//!
//! **Milestone 4 is not finished.** Everything a person can read when Deepgram
//! or the microphone ends a dictation mid flow is owed a drawn state that
//! `design/registry.md` does not hold, so it is not built and was not invented:
//! the pill's words, the pill's `MIC STOPPED`, and the EchoScribe window's
//! screen with its one action. The kinds, codes, sentences and actions all exist
//! and ride on `dictation:error` already. See
//! `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`.
//!
//! Still to come: history and settings (milestone 5).

mod consent;
pub mod deepgram_key;
mod hook;
mod key_vault;
mod limits;
mod machine;
mod microphone;
mod pill_mouse;
mod pill_window;
mod sound;
pub mod store;
mod transcribe;
mod typing;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Listener, Manager, State};

use limits::Deadlines;
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
    /// Deepgram or a password field ended the dictation while the microphone
    /// was open (record 0002 AC-13, AC-14, AC-20, AC-30).
    EndedBy(transcribe::Ended),
}

/// One dictation in progress: the open microphone and the live connection to
/// Deepgram. Holding one is what "listening" means.
struct Live {
    mic: Microphone,
    session: transcribe::Session,
}

impl Live {
    /// Close both, in the one order that works.
    ///
    /// The microphone first, always. It owns the only feed into the stream, so
    /// closing it is what tells Deepgram no more audio is coming, which is what
    /// lets the last words of a sentence come back instead of being cut off.
    fn close(self) {
        self.mic.stop();
        self.session.stop();
    }
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
/// `listening` holds the open microphone and its Deepgram connection, and
/// holding it is what "listening" means: there is no separate flag that could
/// disagree with the device. `None` is idle, and the microphone is shut.
pub struct Dictate {
    store: Store,
    listening: Mutex<Option<Live>>,
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
                    Command::EndedBy(ended) => on_ended(&consumer_handle, ended),
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

        if let Some(live) = listening.take() {
            // Shut the device before anything on screen changes, so the pill
            // never outlives the microphone in either direction.
            live.close();
            pill_window::close(app, "you_stopped_it");
            sound::play_close(setting.sounds_enabled);
            return;
        }
    }

    if let Err(e) = try_start(app, commands) {
        report_start_error(app, e);
    }
}

/// Everything that can stop dictation before it begins.
///
/// Two shapes, because they are read in two different places. A missing key is
/// answered by the guided setup screen (AC-9); a microphone failure is answered
/// by the error screen (AC-28). Both bring the EchoScribe window forward, both
/// leave the microphone shut, and neither ever shows a pill.
#[derive(Debug, Clone, Copy)]
enum StartError {
    /// No Deepgram key is saved for this account (record 0002 AC-9).
    NoDeepgramKey,
    /// The microphone would not open (record 0002 AC-15, AC-28).
    Microphone(MicError),
}

impl From<MicError> for StartError {
    fn from(e: MicError) -> Self {
        StartError::Microphone(e)
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
fn try_start(app: &AppHandle, commands: &Sender<Command>) -> Result<(), StartError> {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return Ok(());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Ok(());
    };
    let setting = state.store.setting_for(&account_id).unwrap_or_default();

    // AC-9: with no key saved the microphone does not open. Checked before the
    // device is touched, so there is no moment in which it was open.
    //
    // Milestone 4 needs the key itself and not merely the row, because there is
    // now something to stream with it. Anything that stops us getting it, a
    // missing row, a vault that will not answer, or a credential somebody
    // deleted from Windows behind our back, is treated as no key. That sends
    // the person to a screen they can act on, and it errs towards not opening
    // the microphone, which is the only safe direction to err in.
    let Some(key) = read_key(&state, &account_id) else {
        return Err(StartError::NoDeepgramKey);
    };

    let mut listening = state
        .listening
        .lock()
        .expect("dictate listening mutex poisoned");
    if listening.is_some() {
        return Ok(());
    }

    // AC-8: the silence cap is armed from here on. Milestone 2 built it unarmed
    // because nothing could honestly report speech; Deepgram's final results
    // can, and `transcribe` calls `speech_heard` on every one of them. Shared
    // between the level thread, which reads the caps, and the transcription
    // thread, which restarts the silence clock.
    let deadlines = Arc::new(Mutex::new(Deadlines::watching_for_silence(Instant::now())));

    // Made before the microphone, because the microphone needs somewhere to put
    // its samples and Deepgram needs the microphone's sample rate.
    let (feed, intake) = transcribe::channel();

    // `consent::refine` is step 2b: a failure with no named cause asks the
    // Windows consent switches, read only, and becomes blocked-by-Windows if
    // any of the three says deny. It sits here so the hotkey and Try again
    // share it, the same as they share everything else on this path.
    let mic = microphone::open(
        level_sink(app, commands, Arc::clone(&deadlines)),
        Box::new(move |samples| feed.push(samples)),
    )
    .map_err(consent::refine)?;

    let ended_commands = commands.clone();
    let session = match transcribe::start(
        app.clone(),
        key,
        mic.sample_rate(),
        deadlines,
        Box::new(move |ended| {
            // Everything that changes what this feature is doing goes through
            // the one channel, so a failure arriving from the network thread
            // cannot race the hotkey.
            let _ = ended_commands.send(Command::EndedBy(ended));
        }),
        intake,
    ) {
        Ok(session) => session,
        Err(e) => {
            eprintln!("dictate: could not start the transcription thread: {e}");
            // The device opened but nothing can transcribe. Close it again
            // rather than leave a microphone open that does nothing.
            mic.stop();
            return Err(StartError::Microphone(MicError::Unavailable));
        }
    };

    *listening = Some(Live { mic, session });
    pill_window::open(app, &setting);
    sound::play_open(setting.sounds_enabled);
    Ok(())
}

/// The account's Deepgram key, ready to stream with.
///
/// Returns `None` for every reason there might not be one, because they all
/// have the same answer: the guided setup screen. The key is never logged and
/// never returned anywhere but here.
fn read_key(state: &Dictate, account_id: &str) -> Option<String> {
    let saved = match state.store.deepgram_key_for(account_id) {
        Ok(Some(saved)) => saved,
        Ok(None) => return None,
        Err(e) => {
            eprintln!("dictate: could not tell whether a Deepgram key is saved: {e}");
            return None;
        }
    };
    match key_vault::load(&saved.credential_target) {
        Ok(Some(key)) => Some(key),
        Ok(None) => {
            eprintln!(
                "dictate: this account has a saved Deepgram key on record, but the credential is \
                 no longer in Windows Credential Manager"
            );
            None
        }
        Err(e) => {
            eprintln!("dictate: could not read the saved Deepgram key: {e}");
            None
        }
    }
}

/// The callback the microphone calls every 60 ms with one loudness number.
///
/// It does two things and nothing else: send the number to the pill's level
/// meter, and check the caps. It never touches the listening state itself, so
/// it can never be holding a lock that the close it asks for needs.
fn level_sink(
    app: &AppHandle,
    commands: &Sender<Command>,
    caps: Arc<Mutex<Deadlines>>,
) -> Box<dyn Fn(f32) + Send> {
    let app = app.clone();
    let commands = commands.clone();
    // Ticks keep arriving until the close actually happens, so remember whether
    // one has already been asked for.
    let asked_to_close = AtomicBool::new(false);

    Box::new(move |level: f32| {
        let _ = app.emit_to(
            pill_window::LABEL,
            "dictation:level",
            json!({ "level": level }),
        );

        // Read the caps and let the lock go before doing anything with the
        // answer, so this never holds it across a send.
        let expiry = caps
            .lock()
            .ok()
            .and_then(|caps| caps.expired(Instant::now()));
        if let Some(expiry) = expiry {
            if !asked_to_close.swap(true, Ordering::SeqCst) {
                let _ = commands.send(Command::CloseBecause(expiry.reason()));
            }
        }
    })
}

/// Say why dictation did not start, where a person can actually read it
/// (record 0002 AC-9, AC-15, AC-28).
///
/// Both reasons take the same shape, which the record settled on 2026-08-30: a
/// hotkey press that could not start dictation, and the EchoScribe window
/// brought forward saying why, carrying exactly one thing to do. The pill is a
/// sign, never a control, and it never appears when the microphone did not
/// open, so neither of these is ever read on a pill.
///
/// Bringing the window forward moves focus out of whatever the person was in.
/// The record accepts that cost because this only ever answers a hotkey they
/// just pressed and got nothing from.
fn report_start_error(app: &AppHandle, e: StartError) {
    match e {
        // AC-9. No code and no sentence: nothing has gone wrong, there is just
        // a setup step outstanding, and the guided screen explains it. The
        // event carries no detail because the screen needs none.
        StartError::NoDeepgramKey => {
            eprintln!("dictate: no Deepgram key is saved, so the microphone was not opened");
            let _ = app.emit("dictation:needs_key", json!({}));
        }
        StartError::Microphone(mic) => {
            eprintln!(
                "dictate: the microphone did not open: {} ({:?})",
                mic.message(),
                mic
            );
            // The event goes to every window, so the main window receives it on
            // the capability it already has. Emitted before the window comes
            // forward so the screen is mounting as it arrives.
            let _ = app.emit(
                "dictation:error",
                json!({ "kind": mic.kind(), "message": mic.message() }),
            );
        }
    }
    bring_window_forward(app);
}

/// Put the EchoScribe window in front of whatever the person was in. The one
/// place that does this, so every reason to interrupt somebody goes through the
/// same three calls.
fn bring_window_forward(app: &AppHandle) {
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

impl From<StartError> for DictationErrorPayload {
    fn from(e: StartError) -> Self {
        match e {
            // The setup screen is already on its way from the
            // `dictation:needs_key` event, so this only tells the caller why
            // Try again did not start anything. It carries no sentence because
            // no error line is ever drawn for it.
            StartError::NoDeepgramKey => Self {
                kind: "no_deepgram_key",
                message: "",
            },
            StartError::Microphone(mic) => Self {
                kind: mic.kind(),
                message: mic.message(),
            },
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
            return Err(DictationErrorPayload::from(StartError::Microphone(
                MicError::Unavailable,
            )));
        };
        let sender = state
            .commands
            .lock()
            .expect("dictate commands mutex poisoned")
            .clone();
        sender
    };
    try_start(&app, &commands).map_err(|e| {
        // A key that was cleared between the failure and the retry lands here.
        // It needs the setup screen, not an error line, so it goes through the
        // same reporting path a hotkey press would.
        if matches!(e, StartError::NoDeepgramKey) {
            report_start_error(&app, e);
        }
        DictationErrorPayload::from(e)
    })
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
    let Some(live) = listening.take() else {
        return;
    };

    live.close();
    pill_window::close(app, reason);

    let sounds_enabled = crate::sign_in::account_id_from(app)
        .and_then(|account_id| state.store.setting_for(&account_id).ok())
        .map(|setting| setting.sounds_enabled)
        .unwrap_or(true);
    sound::play_close(sounds_enabled);
}

/// Deepgram or a password field ended the dictation while the microphone was
/// open (record 0002 AC-13, AC-14, AC-20, AC-30).
///
/// The ending the record fixes for every mid dictation failure, in its order:
/// the microphone shuts at once, the pill says what happened in words, and then
/// the pill closes and the closing sound plays. The pill carries no action,
/// ever; it is a sign and never a control.
///
/// **Two halves of this are deliberately not here yet, and both are `/canvas`
/// work.** The record has the pill say what happened in words, and brings the
/// EchoScribe window forward carrying the one action. `design/registry.md`
/// draws neither for a mid dictation Deepgram error: its "Error pill" row keeps
/// the working pill's 232x44, which the decided sentences do not fit in, and it
/// has no screen for this at all. Record 0002 says twice that `/canvas` goes
/// first and that nothing here may be invented during a build, so this stops at
/// closing cleanly and emitting the event. The event already carries the kind,
/// the code, the sentence and the action, so the two screens are all that is
/// owed.
fn on_ended(app: &AppHandle, ended: transcribe::Ended) {
    let Some(state) = app.try_state::<Dictate>() else {
        return;
    };

    // The device goes first, and immediately. Whatever went wrong, the
    // microphone has no business staying open while somebody reads about it.
    {
        let mut listening = state
            .listening
            .lock()
            .expect("dictate listening mutex poisoned");
        let Some(live) = listening.take() else {
            // Something else already closed it. Nothing to say twice.
            return;
        };
        live.close();
    }

    match ended {
        transcribe::Ended::Deepgram(cause) => {
            eprintln!(
                "dictate: dictation stopped: {} ({})",
                cause.code(),
                cause.message()
            );
            // Broadcast, the same way every other error on this feature is, so
            // the window receives it on the capability it already has and
            // nothing in `src-tauri/capabilities/` widens.
            let _ = app.emit(
                "dictation:error",
                json!({
                    "kind": cause.kind(),
                    "code": cause.code(),
                    "message": cause.message(),
                    "action": cause.action(),
                }),
            );
        }
        // The pill was told at the moment of the refusal, from the thread that
        // refused, so the person saw it before the words stopped arriving.
        transcribe::Ended::BlockedPasswordField => {
            eprintln!("dictate: dictation stopped: BLOCKED_PASSWORD_FIELD");
        }
    }

    // The pill goes at once, rather than holding for a beat so the words can be
    // read. That is not what record 0002 asks for, and it is deliberate: the
    // pill has no error state built, because the one `design/registry.md` draws
    // is the working pill's own geometry and the decided sentences do not fit
    // in it. Holding the pill here would leave it saying MIC OPEN over a
    // microphone that just closed, which is exactly the lie on screen this
    // record refuses everywhere else. So it closes honestly and says nothing,
    // and the pause comes back with the drawn state. See the milestone 4
    // decisions file, "What is still blocked".
    pill_window::close(app, "an_error");

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

#[cfg(test)]
mod tests {
    /// Whitespace removed and anything non-ASCII dropped, so a guard survives a
    /// formatter moving a call across lines.
    fn flattened(source: &str) -> String {
        source
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    /// The interface shell, which owns which screen is on the EchoScribe window
    /// and so owns the clearing half of AC-32.
    const SHELL: &str = include_str!("../../../src/main.js");

    /// The pill window's source, minus its own tests, so a guard cannot be
    /// satisfied by a test that quotes the thing it is checking for.
    fn the_pill_window_file() -> &'static str {
        include_str!("pill_window.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn the_microphone_opening_reaches_the_echoscribe_window_and_not_only_the_pill() {
        // covers: AC-32, as a source guard only. An error screen clears itself
        // the moment the thing it complained about is shown to work, and for
        // the four microphone errors that proof is `dictation:opened`. Sent to
        // the pill alone it never reaches the window holding the error, and the
        // screen sits there saying dictation cannot start while it is running.
        let flat = flattened(the_pill_window_file());
        assert!(
            flat.contains(r#"app.emit("dictation:opened""#),
            "`dictation:opened` is no longer broadcast to every window. The \
             EchoScribe window then never learns the microphone opened and a \
             microphone error stays on screen after the person has fixed it \
             (record 0002 AC-32)"
        );
    }

    #[test]
    fn the_allowance_error_is_not_cleared_by_the_microphone_opening() {
        // covers: AC-32. The clearing table gives each error its own proof. An
        // open microphone is not proof that a Deepgram allowance is back, so
        // wiring `deepgram_no_allowance` to it would take the message away
        // while the problem was still there. That kind clears on the first
        // finalised words, in milestone 4, and nowhere in the shell before it.
        assert!(
            !flattened(SHELL).contains("deepgram_no_allowance"),
            "src/main.js now knows about `deepgram_no_allowance`. Its only \
             clearing trigger is the first finalised words from Deepgram \
             (record 0002, the clearing table), never the microphone opening"
        );
    }

    #[test]
    fn clearing_an_error_never_moves_the_echoscribe_window() {
        // covers: AC-32. It clears quietly: the window does not come to the
        // front, does not hide itself and does not move. This record brings a
        // window forward for one reason only, a hotkey press that produced
        // nothing, and that call lives in `bring_window_forward` here in Rust.
        // The shell has no business reaching for a window at all.
        let flat = flattened(SHELL);
        for forbidden in [
            "getCurrentWindow",
            "getCurrentWebviewWindow",
            "setFocus",
            ".hide(",
            ".setPosition(",
        ] {
            assert!(
                !flat.contains(forbidden),
                "src/main.js now mentions `{forbidden}`. Clearing an error is \
                 quiet: the person is dictating into another app, and the \
                 window must not come forward, hide or move (record 0002 AC-32)"
            );
        }
    }
}
