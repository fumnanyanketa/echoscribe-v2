//! The app shell: which of EchoScribe's two windows is the window at rest.
//!
//! There are two, and each keeps the job it is shaped for (record 0004).
//!
//!   * `main`, the 760x540 non-resizable dark window, holds everything with one
//!     way forward: sign in, first run, the Deepgram key setup screen, the
//!     microphone error screen and the mid-dictation Deepgram error screen.
//!   * `dashboard`, the 1200x800 two-tone window, holds the nav rail and the
//!     white reading surface.
//!
//! The invariant everything here leans on: **while a person is signed in with a
//! key saved, the dashboard window exists.** So the small window is shown when,
//! and only when, there is a pre-shell state, meaning nobody is signed in or no
//! key is saved yet, or there is an interruption record 0002 or record 0003
//! names. Otherwise it is hidden and the dashboard is what is on screen.
//!
//! Rust decides all of that. The interface asks and draws; it has no command
//! through which to show, hide, move or resize anything, and the dashboard's
//! capability grants it nothing that could.

pub mod dashboard_window;
pub mod geometry;
pub mod rail;
pub mod store;

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Listener, Manager, State};

use crate::dictate::error_screen::{screen_for, ErrorScreen};
use rail::RailView;
use store::Store;

/// The one SQLite file that holds everything EchoScribe persists (AGENTS.md
/// data rules). The sign-in feature opens it first, the dictate feature opens
/// its own connection to the same file, and so does this one. The name is
/// shared infrastructure and will move to a shared module once a third feature
/// needs it; three now do, so that move is owed and is `/sync`'s to record.
const DB_FILE: &str = "echoscribe.sqlite3";

/// The label of the small dark window, the one `tauri.conf.json` declares.
const SMALL_WINDOW: &str = "main";

/// How long a move or a resize has to stop for before it counts as finished.
/// The row is written when the person has finished, not on every pixel of a
/// drag (record 0004 data rules).
const SETTLED: Duration = Duration::from_millis(400);

/// Everything this feature keeps for the running app. Managed by Tauri.
pub struct Shell {
    store: Store,
    /// Which error screen is on the small window, when one is. `None` means
    /// nothing has gone wrong, so the small window has no reason to be up.
    ///
    /// This is record 0002's clearing table read for two windows, and it is the
    /// second reader of it: the interface decides what is drawn, and Rust
    /// decides whether the window is on screen at all, which AGENTS.md puts in
    /// Rust rather than leaving to a screen choosing not to ask. Neither reader
    /// holds a copy of the table any more. Which screen a kind belongs on is
    /// classified once, in the dictate feature where the kinds are minted, and
    /// read here through `screen_for` (record 0004's second amendment of
    /// 2026-09-03, and record 0002's fifteenth). What stays on both sides is
    /// each family's pairing with its own proof, below, because each side
    /// clears the thing it is itself holding.
    interruption: Mutex<Option<ErrorScreen>>,
    /// A nudge that the dashboard has been moved or resized. One thread reads
    /// these, waits for them to stop, and writes the row once.
    moves: Mutex<Sender<()>>,
}

/// Open this feature's database connection, wire up everything that changes
/// which window is at rest, and settle the windows for the state the app is
/// already in.
///
/// Called from the Tauri setup hook **after** `sign_in::init` and
/// `dictate::init`, and the order matters: `account` has to exist before this
/// feature's table can reference it, and `deepgram_credential` has to exist
/// before this feature can count rows in it.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    let (moves, move_rx) = mpsc::channel::<()>();
    app.manage(Shell {
        store,
        interruption: Mutex::new(None),
        moves: Mutex::new(moves),
    });
    start_geometry_thread(handle.clone(), move_rx)?;

    // Closing the small window is how a person dismisses whatever it is
    // carrying. With the dashboard up it must not destroy the window, because
    // every error screen in the app lives on it and there would be nowhere for
    // the next one to go, so the close is refused and the window is hidden
    // instead. With no dashboard behind it this window is the app, and closing
    // it ends the app, exactly as closing the dashboard does (record 0004,
    // AC-6).
    //
    // The close is refused in both cases, and the second one is the reason.
    // The pill window is created at startup and never destroyed, so Tauri's
    // own "the last window has gone" exit can never fire while EchoScribe is
    // running: letting this close through destroyed the only window a person
    // had and left the process alive on a pill nobody can see, still holding
    // the global hotkey. Asking to exit is a request the event loop carries
    // out on a later turn, not the exit itself (AGENTS.md standing rule 14),
    // so the window is kept until that lands. A window still on screen is the
    // harmless way for the exit to fail; destroying it first is not.
    if let Some(small) = app.get_webview_window(SMALL_WINDOW) {
        let dismiss_handle = handle.clone();
        small.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if dashboard_window::is_open(&dismiss_handle) {
                    clear_interruption(&dismiss_handle, None);
                    settle_small_window(&dismiss_handle);
                } else {
                    // The same one way out of the app that the dashboard's own
                    // close uses. There is nothing to write down first: this
                    // window's size and place are not remembered, and the
                    // pill's spot is written by the dictate feature when a
                    // drag settles.
                    dismiss_handle.exit(0);
                }
            }
        });
    }

    // The four things that change whether the dashboard may exist at all.
    for signal in [
        "auth:signed_in",
        "auth:signed_out",
        "dictation:key_saved",
        "dictation:key_cleared",
    ] {
        let signal_handle = handle.clone();
        app.listen(signal, move |_| {
            if signal == "dictation:key_saved" {
                clear_interruption(&signal_handle, Some(ErrorScreen::KeySetup));
            }
            settle(&signal_handle);
        });
    }

    // An interruption arriving. The dictate feature has already brought the
    // small window forward; this only records that it is up, so that the window
    // is not hidden again until the thing it complains about is shown to work.
    //
    // Which screen the kind belongs on is the dictate feature's answer, not one
    // worked out again here. A kind with no screen is not an interruption: the
    // password refusal is that case, and its event goes to the pill alone and
    // the window deliberately stays where it is (record 0002 AC-20).
    let error_handle = handle.clone();
    app.listen("dictation:error", move |event| {
        let kind = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|payload| payload.get("kind")?.as_str().map(str::to_string));
        if let Some(screen) = kind.as_deref().and_then(screen_for) {
            note_interruption(&error_handle, screen);
        }
    });
    let needs_key_handle = handle.clone();
    app.listen("dictation:needs_key", move |_| {
        note_interruption(&needs_key_handle, ErrorScreen::KeySetup);
    });

    // The two proofs Rust already emits, each clearing the one interruption it
    // disproves. An open microphone is no proof that a Deepgram allowance is
    // back, and words coming back are no proof that a microphone will open, so
    // neither clears the other's screen (record 0002 AC-32 and its clearing
    // table).
    let opened_handle = handle.clone();
    app.listen("dictation:opened", move |_| {
        clear_interruption(&opened_handle, Some(ErrorScreen::Microphone));
        settle_small_window(&opened_handle);
    });
    let text_handle = handle.clone();
    app.listen("dictation:text", move |_| {
        clear_interruption(&text_handle, Some(ErrorScreen::Deepgram));
        settle_small_window(&text_handle);
    });

    settle(&handle);
    Ok(())
}

fn note_interruption(app: &AppHandle, kind: ErrorScreen) {
    if let Some(shell) = app.try_state::<Shell>() {
        if let Ok(mut held) = shell.interruption.lock() {
            *held = Some(kind);
        }
    }
}

/// Forget the interruption on the small window. `only` clears it just when it
/// is that screen, which is what stops one proof clearing another's screen;
/// `None` clears whatever is there, which is what a person dismissing the
/// window does.
fn clear_interruption(app: &AppHandle, only: Option<ErrorScreen>) {
    if let Some(shell) = app.try_state::<Shell>() {
        if let Ok(mut held) = shell.interruption.lock() {
            match only {
                Some(kind) if *held != Some(kind) => {}
                _ => *held = None,
            }
        }
    }
}

/// Whether the dashboard may exist: somebody is signed in, and a Deepgram key
/// is saved for them. Both are things Rust already knows, and neither is ever
/// passed in from the interface (record 0004, value sourcing).
fn dashboard_is_due(app: &AppHandle) -> bool {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return false;
    };
    let Some(shell) = app.try_state::<Shell>() else {
        return false;
    };
    shell
        .store
        .deepgram_key_exists(&account_id)
        .unwrap_or_else(|e| {
            // Unreadable is not the same as absent, but it is the same answer:
            // do not put a person in front of a dashboard we cannot vouch for.
            eprintln!("shell: could not tell whether a Deepgram key is saved: {e}");
            false
        })
}

/// The one place that decides which window is at rest. Creates the dashboard
/// when it is due and destroys it when it is not, then settles the small
/// window around it.
fn settle(app: &AppHandle) {
    let dashboard_up = if dashboard_is_due(app) {
        if dashboard_window::is_open(app) {
            true
        } else {
            let remembered = remembered_placement(app);
            match dashboard_window::open(app, remembered) {
                Ok(()) => true,
                Err(e) => {
                    // Say so loudly and leave the small window up: a person
                    // with no window at all has no way back into the app.
                    eprintln!("shell: the dashboard window could not be created: {e}");
                    false
                }
            }
        }
    } else {
        dashboard_window::close(app);
        // Destroying a window is a request the event loop carries out on a
        // later turn, so the dashboard is still gettable at this instant even
        // though it is on its way out, and asking would get the answer "still
        // there" (AGENTS.md standing rule 14: the call returning says the
        // destroy was accepted, not that the window has gone). This branch is
        // the only thing that knows there will be no dashboard, so it says so
        // rather than leaving the next step to ask. Getting this wrong hid the
        // small window over a dashboard that was already going and left the
        // app running with nothing on screen, which is AC-7 and the state the
        // record's second amendment refused for the close button.
        false
    };
    settle_small_window_around(app, dashboard_up);
}

/// Show or hide the small window, without touching the dashboard.
///
/// It is hidden only when there is a dashboard to reveal and nothing has gone
/// wrong. That is AC-6: the small window goes and the dashboard is there
/// exactly as it was. Nothing here shows, focuses, moves or resizes the
/// dashboard, so it is never brought to the front by an error clearing, and
/// bringing the small window forward for a reason stays the dictate feature's
/// one place that does it.
fn settle_small_window(app: &AppHandle) {
    // Nothing here has just ordered the dashboard open or closed, so what the
    // system reports is the truth. `settle` is the one caller that cannot ask,
    // and it passes its own answer in instead.
    settle_small_window_around(app, dashboard_window::is_open(app));
}

/// The body of the above, told whether there is a dashboard rather than asking.
fn settle_small_window_around(app: &AppHandle, dashboard_up: bool) {
    let Some(small) = app.get_webview_window(SMALL_WINDOW) else {
        return;
    };
    let interrupted = app
        .try_state::<Shell>()
        .and_then(|shell| shell.interruption.lock().ok().map(|held| held.is_some()))
        .unwrap_or(true);

    if dashboard_up && !interrupted {
        let _ = small.hide();
    } else {
        // Deliberately not `set_focus`: this shows the window where the state
        // of the app says it belongs, and an error that wants attention is
        // brought forward by the dictate feature instead.
        let _ = small.show();
    }
}

/// What was remembered for the signed-in account, or `None` for a first ever
/// open or an unreadable row.
fn remembered_placement(app: &AppHandle) -> Option<store::RememberedWindow> {
    let account_id = crate::sign_in::account_id_from(app)?;
    let shell = app.try_state::<Shell>()?;
    shell
        .store
        .read(&account_id, geometry::FLOOR_W, geometry::FLOOR_H)
        .unwrap_or_else(|e| {
            eprintln!("shell: could not read the remembered window: {e}");
            None
        })
}

/// One thread, waiting for a move or a resize to stop and then writing the row
/// once. A drag sends dozens of these; the row is written after the last one.
fn start_geometry_thread(
    app: AppHandle,
    rx: Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::Builder::new()
        .name("echoscribe-shell-geometry".into())
        .spawn(move || {
            while rx.recv().is_ok() {
                // Swallow everything that arrives while the person is still
                // dragging or resizing.
                while rx.recv_timeout(SETTLED).is_ok() {}
                remember_placement(&app);
            }
        })?;
    Ok(())
}

/// The dashboard has been moved or resized. Called from the window's own event
/// handler, which runs on the window's thread, so it does no more than nudge
/// the thread that does the work.
fn moved_or_resized(app: &AppHandle) {
    if let Some(shell) = app.try_state::<Shell>() {
        if let Ok(moves) = shell.moves.lock() {
            let _ = moves.send(());
        }
    }
}

/// Read where the dashboard is now and remember it for the signed-in account.
fn remember_placement(app: &AppHandle) {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return;
    };
    let Some(shell) = app.try_state::<Shell>() else {
        return;
    };
    let Some(placement) = dashboard_window::placement_now(app) else {
        return;
    };
    let now = crate::sign_in::clock::now_iso8601();
    if let Err(e) = shell.store.save(&account_id, &placement, &now) {
        eprintln!("shell: could not remember where the dashboard was left: {e}");
    }
}

/// The person closed the dashboard. Remember where it was, then leave: while
/// somebody is signed in with a key saved this window is the app, and AC-4 is
/// written as "close the app, reopen it".
fn dashboard_closed(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        remember_placement(&app);
        app.exit(0);
    });
}

/// The sections the rail holds, as identifiers, and which of them the dashboard
/// opens on. Refuses when nobody is signed in, like every command on this
/// surface.
#[tauri::command]
pub fn get_rail(app: AppHandle, _state: State<'_, Shell>) -> Result<RailView, &'static str> {
    if crate::sign_in::account_id_from(&app).is_none() {
        return Err("not_signed_in");
    }
    Ok(rail::view())
}

#[cfg(test)]
mod tests {
    /// This feature's own source, minus its tests, flattened, so a guard
    /// survives a formatter moving a call across lines.
    fn flattened(source: &str) -> String {
        source
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    /// Which screen the listener for `signal` clears, read out of this file's
    /// own source. The last mention of the signal is the one inside a listener:
    /// the earlier one, when there is one, is the array of signal names.
    fn screen_cleared_on(flat: &str, signal: &str) -> String {
        let quoted = format!("\"{signal}\"");
        let at = flat
            .rfind(&quoted)
            .unwrap_or_else(|| panic!("the shell no longer listens for {signal} at all"));
        let call = flat[at..]
            .find("clear_interruption(")
            .unwrap_or_else(|| panic!("{signal} no longer clears anything"));
        let from = at + call;
        let marker = flat[from..]
            .find("ErrorScreen::")
            .unwrap_or_else(|| panic!("{signal} clears without naming a screen"));
        let start = from + marker + "ErrorScreen::".len();
        let end = flat[start..]
            .find(')')
            .expect("a named screen is inside a call");
        flat[start..start + end].to_string()
    }

    #[test]
    fn each_proof_clears_only_its_own_screen() {
        // covers: AC-6, and record 0002 AC-32, as a source guard only. This is
        // the Rust half of the residue the fifteenth amendment left on both
        // sides: which kind belongs on which screen is now classified once, in
        // the dictate feature, but each side still pairs a family with its own
        // proof, because each side clears the thing it is itself holding. Get a
        // pairing wrong here and a person's error window is taken away by
        // something that did not disprove it: an open microphone says nothing
        // about a Deepgram allowance, and words coming back say nothing about a
        // microphone that will not open. The interface half of the same residue
        // is guarded in dictate/mod.rs, over src/main.js.
        let flat = flattened(include_str!("mod.rs"));
        assert_eq!(
            screen_cleared_on(&flat, "dictation:opened"),
            "Microphone",
            "the microphone opening must clear the microphone screen and nothing else"
        );
        assert_eq!(
            screen_cleared_on(&flat, "dictation:text"),
            "Deepgram",
            "the first finalised words must clear the Deepgram screen and nothing else"
        );
        assert_eq!(
            screen_cleared_on(&flat, "dictation:key_saved"),
            "KeySetup",
            "a key being accepted must clear the key setup screen and nothing else"
        );
    }

    #[test]
    fn closing_the_small_window_never_leaves_the_app_with_no_window() {
        // covers: AC-6, as a source guard only. The pill window is created at
        // startup and never destroyed, so Tauri never sees a last window go and
        // so never exits by itself. That makes both halves of this one handler
        // load-bearing: the close is refused whatever is behind the window, and
        // when there is no dashboard behind it the app is ended on purpose.
        // With the refusal inside the dashboard branch, closing the sign in or
        // key setup screen destroyed the only window a person had and left
        // echoscribe.exe running on a hidden pill, still holding the global
        // hotkey, with no way back in (found live 2026-09-03). Proving the
        // behaviour needs a real click on a real title bar, so /check verify
        // owns that; this stops either line drifting back inside the branch.
        let flat = flattened(include_str!("mod.rs"));
        let at = flat
            .find("CloseRequested")
            .expect("the shell no longer watches the small window closing");
        // Bounded at the next thing in `init`, so a match here can only come
        // from this handler and not from the dashboard's own exit further down.
        let end = flat[at..]
            .find("\"auth:signed_in\"")
            .expect("the shell no longer listens for the sign-in signals");
        let handler = &flat[at..at + end];
        let branch = handler
            .find("dashboard_window::is_open(")
            .expect("the close handler no longer asks whether the dashboard is up");
        assert!(
            handler[..branch].contains("api.prevent_close();"),
            "the small window's close is refused only inside a branch. Whichever              branch that is, the other one lets the window be destroyed, and with              the pill alive that leaves EchoScribe running with nothing on screen"
        );
        assert!(
            handler.contains(".exit(0)"),
            "closing the small window no longer ends the app. With no dashboard              behind it that window is the app, and nothing else will ever exit"
        );
    }

    #[test]
    fn clearing_an_error_never_raises_the_dashboard() {
        // covers: AC-6, and record 0002 AC-32, as a source guard only. The
        // whole of AC-6's second half is that the dashboard is revealed rather
        // than raised: the person is working in another app and a window taking
        // focus costs them their place. Only the dictate feature brings a
        // window forward, for one reason, so nothing in this file may focus or
        // raise one. The behaviour itself needs two real windows and a real
        // failure, so /check verify owns proving it; this stops the line being
        // added back as a convenience.
        // The call, not the name: this file's own comments say which calls are
        // deliberately absent, and a guard that could not tell a comment from a
        // call would make writing that down impossible.
        let flat = flattened(include_str!("mod.rs"));
        for forbidden in ["set_focus(", "unminimize(", "set_always_on_top("] {
            assert!(
                !flat.contains(forbidden),
                "the shell calls {forbidden}. Clearing an error would then take \
                 the person's typing cursor, which record 0002 AC-32 and record \
                 0004 AC-6 both refuse"
            );
        }
    }
}
