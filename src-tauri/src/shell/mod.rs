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
    /// Held for the whole of `settle`, so two threads can never both ask "is
    /// the dashboard open", both hear no, and both build one. That happened
    /// live on 2026-10-01: `init` runs `settle` on the main thread while the
    /// sign-in renewal thread's first refresh emits `auth:signed_in`, whose
    /// listener runs `settle` on the renewal thread. Both builds succeed, the
    /// second takes the label in Tauri's window map, and the first becomes a
    /// window no `get_webview_window` can ever reach again: a ghost dashboard
    /// stacked pixel for pixel behind the real one, unmanageable for the life
    /// of the process. It is a race, so most launches look fine, which is
    /// exactly why the gate is a lock and not a boot-order promise.
    settle_gate: Mutex<()>,
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
        settle_gate: Mutex::new(()),
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
    // One settle at a time, across all threads. The ask-then-create below is
    // only safe when nothing else can be between the ask and the create; see
    // `settle_gate` on [`Shell`] for the night this was learned. A poisoned
    // lock means a settle panicked, and settling anyway is strictly better
    // than never settling again.
    let state = app.try_state::<Shell>();
    let _one_at_a_time = state
        .as_ref()
        .map(|shell| match shell.settle_gate.lock() {
            Ok(held) => held,
            Err(poisoned) => poisoned.into_inner(),
        });
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
            "the small window's close is refused only inside a branch. Whichever branch that is, the other one lets the window be destroyed, and with the pill alive that leaves EchoScribe running with nothing on screen"
        );
        assert!(
            handler.contains(".exit(0)"),
            "closing the small window no longer ends the app. With no dashboard behind it that window is the app, and nothing else will ever exit"
        );
    }

    #[test]
    fn settle_never_asks_about_a_dashboard_it_has_just_destroyed() {
        // covers: AC-7, as a source guard only. `window.destroy()` is always
        // posted to the event loop and never run inline, so for the rest of
        // that turn the dashboard is still gettable and asking whether it
        // exists gets the answer "still there" (AGENTS.md standing rule 14).
        // `settle` is the one function that has just ordered the destroy, so it
        // is the one function that must not ask: it works out `dashboard_up`
        // itself and hands the answer on. Asking hid the small window over a
        // dashboard that was already going, so signing out left EchoScribe
        // running with nothing on screen at all (found live 2026-09-03).
        // Proving the behaviour needs a real sign out with two real windows, so
        // /check verify owns that; this stops the asking version being put back
        // because it reads more simply.
        let flat = flattened(include_str!("mod.rs"));
        let start = flat
            .find("fnsettle(app:&AppHandle){")
            .expect("the shell no longer has one place that decides both windows");
        let end = flat[start..]
            .find("fnsettle_small_window(app:&AppHandle){")
            .expect("settle_small_window no longer follows settle in this file");
        let settle = &flat[start..start + end];
        assert!(
            !settle.contains("settle_small_window("),
            "`settle` calls the settle_small_window that asks the system whether the dashboard is open. It has just ordered that window destroyed, so the answer is still yes, and the small window is hidden over a dashboard that is going. It has to pass its own answer to settle_small_window_around instead"
        );
        let destroyed = settle
            .find("dashboard_window::close(")
            .expect("`settle` no longer destroys the dashboard when it is not due");
        assert!(
            !settle[destroyed..].contains("dashboard_window::is_open("),
            "`settle` asks whether the dashboard is open after ordering it destroyed. That answer is always still there, and acting on it leaves EchoScribe running with no window on screen"
        );
    }

    #[test]
    fn settle_takes_the_gate_before_asking_whether_the_dashboard_exists() {
        // covers: record 0004's one-dashboard invariant, as a source guard
        // only. `settle` runs on whichever thread calls it: the main thread at
        // init, and the sign-in renewal thread when its first refresh emits
        // `auth:signed_in`. Two threads that both ask "is the dashboard open"
        // before either has built one both hear no, and both build. The second
        // build takes the label in Tauri's window map and the first becomes a
        // ghost: a real window stacked behind the real dashboard that no
        // `get_webview_window` can ever reach, so nothing can hide, move or
        // close it for the life of the process (found live 2026-10-01, two
        // 'Tauri Window' handles at one rect under one pid). The cure is that
        // the whole decide-and-create is one turn of a lock, so the lock has
        // to be taken before the first ask. Proving the race needs two real
        // threads and two real windows, so /check verify owns that; this stops
        // the gate being dropped because the function reads fine without it.
        let flat = flattened(include_str!("mod.rs"));
        let start = flat
            .find("fnsettle(app:&AppHandle){")
            .expect("the shell no longer has one place that decides both windows");
        let end = flat[start..]
            .find("fnsettle_small_window(app:&AppHandle){")
            .expect("settle_small_window no longer follows settle in this file");
        let settle = &flat[start..start + end];
        let gate = settle
            .find("settle_gate.lock()")
            .expect("`settle` no longer takes the settle gate, so two threads can race to build two dashboards again");
        let first_ask = settle
            .find("dashboard_is_due(")
            .expect("`settle` no longer asks whether the dashboard is due");
        assert!(
            gate < first_ask,
            "`settle` asks whether the dashboard is due before taking the gate. The ask-then-create is only safe inside the lock; outside it, two threads can both hear \"no dashboard\" and both build one"
        );
    }

    #[test]
    fn the_small_window_is_hidden_in_exactly_one_place() {
        // covers: AC-6 and AC-7, as a source guard only, and it is the one
        // guard that would have caught both of 2026-09-03's bugs. EchoScribe
        // can be left with nothing on screen in exactly two ways: hide the
        // small window with no dashboard behind it, or destroy it without
        // ending the app. The second is guarded above. The first holds only
        // while there is a single place in this whole feature that hides that
        // window. A second hiding place anywhere is another door to the same
        // state, and it would not be found by reading this file.
        //
        // `flattened` keeps comments, on purpose, so the guards below it can
        // tell a call from a mention of one. That costs this test one false
        // positive: a comment writing `.hide()` with its brackets counts. If
        // that is why this failed, reword the comment, not the code.
        for (name, source) in [
            ("shell/mod.rs", include_str!("mod.rs")),
            (
                "shell/dashboard_window.rs",
                include_str!("dashboard_window.rs"),
            ),
            ("shell/geometry.rs", include_str!("geometry.rs")),
            ("shell/rail.rs", include_str!("rail.rs")),
            ("shell/store.rs", include_str!("store.rs")),
        ] {
            let expected = usize::from(name == "shell/mod.rs");
            let found = flattened(source).matches(".hide()").count();
            assert_eq!(
                found, expected,
                "{name} hides a window in {found} place(s), and should hide one in {expected}. The small window is hidden in one place only, and that place asks what is behind it first. A second hiding place is a second way to leave EchoScribe with nothing on screen"
            );
        }
    }

    #[test]
    fn the_one_place_that_hides_the_small_window_asks_what_is_behind_it() {
        // covers: AC-6 and AC-7, as a source guard only. The count above is
        // worth nothing if the one place stops consulting the dashboard, which
        // is the shape both of 2026-09-03's bugs had: the window went away and
        // nothing was revealed. Deliberately loose about how the condition is
        // written, because the promise is that the answer is consulted at all,
        // not the order the two halves are tested in.
        let flat = flattened(include_str!("mod.rs"));
        // Past the signature, not at it: the parameter is called `dashboard_up`
        // too, so a region that started at the signature would match itself and
        // pass while the condition was gone. It did, on this test's first run.
        let signature = "fnsettle_small_window_around(app:&AppHandle,dashboard_up:bool){";
        let start = flat
            .find(signature)
            .expect("the shell no longer has one function that shows or hides the small window")
            + signature.len();
        let hide = flat[start..]
            .find(".hide()")
            .expect("the one place that hides the small window has moved out of that function");
        assert!(
            flat[start..start + hide].contains("dashboard_up"),
            "the small window is hidden without consulting whether a dashboard is there to reveal. Hiding it with nothing behind it leaves EchoScribe running with no window on screen and the dictation hotkey still held"
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

    // ----- Guards for the 2026-10-02 to 2026-10-05 rail changes ------------
    //
    // The interface has no test runner (AGENTS.md), so what a Rust test can do
    // about a screen is read its source and fail the build when a promise
    // leaves it. These four hold the promises `/check verify` observed live on
    // 2026-10-06 (docs/evidence/night-shell-2026-10-06/report.md). They prove
    // a line is present, never that the screen behaves.

    #[test]
    fn the_account_row_is_a_button_and_the_only_way_to_sign_out() {
        // covers: design/registry.md "Account block", changed 2026-10-05 by
        // the user: the row is a button that opens a menu holding Sign out,
        // closed by Escape or a click elsewhere. And "Account card", retired
        // the same day: Sign out lives nowhere else.
        let block = flattened(include_str!("../../../src/shell/account-block.js"));
        assert!(
            block.contains("el(\"button\",\"account__identity\")"),
            "the account row is no longer a button, so pressing the name does nothing again"
        );
        for attr in ["aria-haspopup", "aria-expanded", "aria-controls"] {
            assert!(block.contains(attr), "the account row lost {attr}: a screen reader can no longer tell it opens a menu");
        }
        assert!(
            block.contains("event.key===\"Escape\""),
            "Escape no longer closes the account menu"
        );
        assert!(
            block.contains("\"pointerdown\""),
            "a click elsewhere no longer closes the account menu"
        );
        assert!(
            include_str!("../../../src/shell/account-block.js").contains("\"Sign out\""),
            "the account menu no longer holds Sign out"
        );
        for (name, source) in [
            ("shell/dashboard.js", include_str!("../../../src/shell/dashboard.js")),
            ("shell/rail.js", include_str!("../../../src/shell/rail.js")),
            (
                "dictate/transcription-settings.js",
                include_str!("../../../src/dictate/transcription-settings.js"),
            ),
        ] {
            assert!(
                !source.contains("\"Sign out\""),
                "{name} draws its own Sign out. The user put sign-out under the profile on 2026-10-05, and a second one is the scattered rail the 2026-10-02 patrol found"
            );
        }
    }

    #[test]
    fn nothing_in_the_interface_names_the_deleted_account_card() {
        // covers: design/registry.md "Account card", retired 2026-10-05.
        // `src/shell/account-card.js` was deleted in the same change. A file
        // that still names it is either importing something that is gone or
        // carrying a comment that sends the next reader to a file that does
        // not exist, which is how "two copies held together" drifts.
        let deleted = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../src/shell/account-card.js");
        assert!(
            !deleted.exists(),
            "src/shell/account-card.js is back. The account card was retired on 2026-10-05; its contents live in the account menu"
        );
        for (name, source) in [
            ("shell/dashboard.js", include_str!("../../../src/shell/dashboard.js")),
            ("shell/dashboard.css", include_str!("../../../src/shell/dashboard.css")),
            ("shell/rail.js", include_str!("../../../src/shell/rail.js")),
            (
                "dictate/transcription-settings.js",
                include_str!("../../../src/dictate/transcription-settings.js"),
            ),
        ] {
            // account-block.js is left out on purpose: its header says the
            // card was deleted, which is history and not a pointer.
            assert!(
                !source.contains("account-card"),
                "{name} still names src/shell/account-card.js, which was deleted on 2026-10-05. If it is a comment, point it at src/shell/account-block.js, where the surviving copy is"
            );
        }
    }

    #[test]
    fn a_hidden_sub_nav_and_a_hidden_account_menu_are_hidden_on_screen_too() {
        // covers: design/registry.md "Section sub-nav" (corrected 2026-10-02)
        // and "Account block" (2026-10-05). Both are hidden with the `hidden`
        // attribute, and the offline row's 2026-09-03 bug taught that a
        // display rule on the class silently beats the attribute. Each needs
        // its own `[hidden] { display: none }` rule.
        let css = flattened(include_str!("../../../src/shell/dashboard.css"));
        for selector in [".rail__sub[hidden]", ".account__menu[hidden]"] {
            let rule = format!("{selector}{{display:none");
            assert!(
                css.contains(&rule),
                "dashboard.css lost the rule `{selector} {{ display: none }}`. The element's own display rule beats the hidden attribute, so the folded sub-nav or the closed menu would stay on screen"
            );
        }
    }

    #[test]
    fn a_second_press_on_a_section_folds_its_sub_nav_and_changes_nothing_else() {
        // covers: design/registry.md "Section sub-nav", amended 2026-10-02 by
        // the user: a second press on the section a person is already inside
        // folds the sub-nav without changing the screen. The `return` is the
        // whole promise: nothing below it runs, so the screen stays.
        let dashboard = flattened(include_str!("../../../src/shell/dashboard.js"));
        assert!(
            dashboard.contains("toggleSub(rail,id);return;"),
            "dashboard.js no longer folds the sub-nav and stops. Either the fold is gone, or the press goes on to change the screen, which is the navigation AC-2 already covers and not the fold the user asked for"
        );
        let rail = flattened(include_str!("../../../src/shell/rail.js"));
        assert!(
            rail.contains("sub.hidden=!sub.hidden;"),
            "rail.js's toggleSub no longer flips the sub-nav"
        );
        assert!(
            rail.contains("item.setAttribute(\"aria-expanded\",sub.hidden?\"false\":\"true\")"),
            "toggleSub no longer tells the parent item its expanded state, so the fold is layout only and a screen reader hears nothing"
        );
    }
}
