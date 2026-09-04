//! The dictate feature's settings surface: the hotkey choice and the sound
//! switch (record 0002 AC-19, AC-21, AC-22).
//!
//! Four commands, and they are the whole of what the interface may do with
//! either setting. As everywhere else on this surface, the interface asks and
//! Rust decides: no command takes an account id, because Rust already knows it
//! from the session, and every one of them refuses when nobody is signed in.
//!
//! **The two hotkeys are this file's to hand out and nobody's to widen.**
//! `get_hotkey` returns both allowed values along with the chosen one, so the
//! interface renders a list rather than deciding for itself what is allowed,
//! and `set_hotkey` refuses anything that is not one of the two outright. That
//! is AC-22, and it is not a convenience: record 0002's Risk section allows
//! only two modifier double taps because any wider set would mean the keyboard
//! hook inspecting ordinary keystrokes all day. A third choice arriving here
//! would be a security change wearing a settings hat.
//!
//! **Both settings take effect immediately.** The sound switch does so on its
//! own, because `sound.rs` is handed the stored value at each open and close.
//! The hotkey needs one more step: the hook is armed with the chosen modifier,
//! so changing it re-arms the hook there and then, which is AC-19's "works
//! immediately, with no restart".
//!
//! **The two sentences a person can read here live in this file and nowhere
//! else.** Record 0002's fourteenth amendment fixed both, and fixed that they
//! are held in one place in Rust like every other sentence on this surface, so
//! no screen invents its own. The screen draws the code and the sentence it is
//! handed; it does not know either one. The screen itself is
//! `src/dictate/dictation-settings.js`, on the dashboard's white surface.
//!
//! **The surface has a second section from 2026-09-04, and it reads the same
//! two sentences.** The fourteenth amendment's Value sourcing rows name AC-12
//! alongside AC-19 and AC-21, so the saved Deepgram key on Settings,
//! Transcription says a refused write and a failed read in exactly these
//! words. Its two commands live in `deepgram_key.rs`, because everything about
//! the key does, and they return the `SettingError` below rather than a second
//! shape of their own. That is why the constructors are visible to the rest of
//! this feature: one error line, one place, two sections.

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::store::Hotkey;
use super::{hook, modifier_of, Dictate};

/// The mono code and the one sentence of `design/registry.md`'s
/// `Setting error line` when a write is refused (record 0002, fourteenth
/// amendment). It says both things the person needs: the change did not
/// happen, and what is still on screen is therefore true, which is honest only
/// because the screen never moves the shown choice until the write comes back.
const NOT_SAVED_CODE: &str = "SETTING_NOT_SAVED";
const NOT_SAVED_MESSAGE: &str = "This setting could not be saved, so it is unchanged.";

/// The same line when the settings cannot be read as the screen opens (record
/// 0002, fourteenth amendment). In this state **no control is drawn at all**:
/// a control drawn without a chosen value would be showing a setting the app
/// cannot read, and a person could leave believing a hotkey is in force that
/// is not.
const NOT_READ_CODE: &str = "SETTINGS_NOT_READ";
const NOT_READ_MESSAGE: &str = "These settings could not be read, so none is shown.";

/// Why one of the four commands below refused, in the two parts the screen
/// needs: the machine cause, and the line a person reads.
///
/// `reason` is for a log and for one branch the screen has to make. `code` and
/// `message` are the `Setting error line` and are `None` for the two refusals
/// that have no line, because they happen only while Rust is already closing
/// the window this screen lives in. A screen with nothing to draw draws
/// nothing rather than inventing a sentence for a state nobody designed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingError {
    reason: &'static str,
    code: Option<&'static str>,
    message: Option<&'static str>,
}

impl SettingError {
    /// Nobody is signed in. Unreachable from this screen, which only exists
    /// while somebody is, so it carries no line (record 0002, fourteenth
    /// amendment).
    pub(super) fn not_signed_in() -> Self {
        Self {
            reason: "not_signed_in",
            code: None,
            message: None,
        }
    }

    /// The dictate feature is not up yet. Unreachable for the same reason, and
    /// it carries no line for the same reason.
    pub(super) fn not_ready() -> Self {
        Self {
            reason: "not_ready",
            code: None,
            message: None,
        }
    }

    /// A read failed as the screen opened. The one cause the read half can
    /// reach, so the one code it can show.
    pub(super) fn not_read() -> Self {
        Self {
            reason: "could_not_read",
            code: Some(NOT_READ_CODE),
            message: Some(NOT_READ_MESSAGE),
        }
    }

    /// A write did not happen. Every way a write can fail ends here, because
    /// every one of them leaves the setting unchanged, which is exactly what
    /// the sentence says. `reason` keeps the true cause for the log.
    pub(super) fn not_saved(reason: &'static str) -> Self {
        Self {
            reason,
            code: Some(NOT_SAVED_CODE),
            message: Some(NOT_SAVED_MESSAGE),
        }
    }
}

/// What the interface needs to draw the hotkey setting: which one is in force,
/// and the complete list it may be chosen from (record 0002 AC-19, AC-22).
///
/// The values are the stored ones, not wording. What a person reads for each is
/// the screen's business and is not decided here.
#[derive(Debug, Clone, Serialize)]
pub struct HotkeyChoice {
    chosen: &'static str,
    choices: Vec<&'static str>,
}

/// The chosen hotkey and the two it may be chosen from.
#[tauri::command]
pub fn get_hotkey(app: AppHandle) -> Result<HotkeyChoice, SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };
    let setting = state.store.setting_for(&account_id).map_err(|e| {
        eprintln!("dictate: could not read the chosen hotkey: {e}");
        SettingError::not_read()
    })?;

    Ok(HotkeyChoice {
        chosen: setting.hotkey.as_stored(),
        choices: Hotkey::CHOICES.iter().map(|h| h.as_stored()).collect(),
    })
}

/// Choose one of the two hotkeys (record 0002 AC-19, AC-22).
///
/// Refusing a value that is neither is the only refusal this command has left
/// to make, and it makes it before anything is written or armed. Treat the
/// incoming string as hostile input: it is text from outside the core, and the
/// only thing done with it is matching it against the two known values.
///
/// The hook is re-armed after the write, not before, so a database that would
/// not take the change cannot leave the hook watching a key the settings screen
/// will not show as chosen.
#[tauri::command]
pub fn set_hotkey(app: AppHandle, binding: String) -> Result<(), SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };
    let Some(hotkey) = Hotkey::from_chosen(&binding) else {
        // Nothing from `binding` is printed. It came from outside and it has no
        // business in a log; which of the two it was not is not information.
        eprintln!("dictate: a hotkey that is not one of the two was refused");
        // A write that did not happen, so the shown choice is still true and
        // the not-saved line is the honest one. The record says this cannot
        // arrive from the screen at all, because its two rows come from
        // `get_hotkey`; `reason` keeps it apart from a store failure in a log.
        return Err(SettingError::not_saved("unknown_hotkey"));
    };

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .save_hotkey(&account_id, hotkey, &now)
        .map_err(|e| {
            eprintln!("dictate: could not save the chosen hotkey: {e}");
            SettingError::not_saved("could_not_save")
        })?;

    // AC-19: in force from now, with no restart. The hook is already installed
    // and armed for this account, and this points it at the other modifier.
    hook::arm(modifier_of(hotkey));
    Ok(())
}

/// Whether the opening and closing sounds play (record 0002 AC-21). On for a
/// new account.
#[tauri::command]
pub fn get_dictation_sounds(app: AppHandle) -> Result<bool, SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };
    state
        .store
        .setting_for(&account_id)
        .map(|setting| setting.sounds_enabled)
        .map_err(|e| {
            eprintln!("dictate: could not read the sound setting: {e}");
            SettingError::not_read()
        })
}

/// The one sound switch (record 0002 AC-21).
///
/// Switched off, both sounds are silent and nothing else about dictation
/// changes: the pill still appears, because it is the one visible sign that the
/// microphone is open and AGENTS.md forbids opening it without one. Nothing
/// here touches the pill, and nothing may be added that does.
#[tauri::command]
pub fn set_dictation_sounds(app: AppHandle, enabled: bool) -> Result<(), SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .save_sounds_enabled(&account_id, enabled, &now)
        .map_err(|e| {
            eprintln!("dictate: could not save the sound setting: {e}");
            SettingError::not_saved("could_not_save")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This file's own source, minus its tests, flattened so a guard survives a
    /// formatter moving a call across lines.
    fn this_file_flattened() -> String {
        include_str!("settings.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    #[test]
    fn changing_the_hotkey_re_arms_the_hook() {
        // covers: AC-19, as a source guard only. "Picking the other one works
        // immediately, with no restart" is not the database write: the hook is
        // armed with one modifier and keeps watching it until something says
        // otherwise. Without this line the new choice is stored, the settings
        // screen shows it, and the old key is still the one that dictates until
        // the app restarts. Proving it for real needs a running app and a real
        // double tap, which is `/check verify`'s; this guard only stops the
        // line being tidied away as a duplicate of the write above it.
        assert!(
            this_file_flattened().contains("hook::arm(modifier_of(hotkey))"),
            "set_hotkey no longer re-arms the keyboard hook. The new hotkey \
             would then be stored and shown but not in force until a restart \
             (record 0002 AC-19)"
        );
    }

    #[test]
    fn no_command_here_takes_an_account_id() {
        // covers: AGENTS.md data rules, and record 0002's interface surface.
        // No command on this surface takes an account id, because Rust knows
        // it from the session. One that accepted one would let the interface
        // name whose settings to change.
        let flat = this_file_flattened();
        assert!(
            !flat.contains("account_id:String") && !flat.contains("accountid:String"),
            "a command in settings.rs now takes an account id. Whose settings \
             these are is the session's answer, never the interface's"
        );
        // Every command reads it from the session instead, and there are four.
        assert_eq!(
            flat.matches("crate::sign_in::account_id_from(&app)")
                .count(),
            4,
            "settings.rs no longer reads the account from the session in all \
             four of its commands. Each one must refuse when nobody is signed \
             in (record 0002, interface surface)"
        );
    }

    #[test]
    fn the_two_sentences_are_word_for_word_the_records_own() {
        // covers: AC-12, AC-19, AC-21, through record 0002's fourteenth
        // amendment, which fixed both codes and both sentences. They are
        // wording a person reads, so a build may not reword them and neither
        // may a tidy-up. Every character, including the full stop.
        assert_eq!(NOT_SAVED_CODE, "SETTING_NOT_SAVED");
        assert_eq!(
            NOT_SAVED_MESSAGE,
            "This setting could not be saved, so it is unchanged."
        );
        assert_eq!(NOT_READ_CODE, "SETTINGS_NOT_READ");
        assert_eq!(
            NOT_READ_MESSAGE,
            "These settings could not be read, so none is shown."
        );
    }

    #[test]
    fn a_write_that_did_not_happen_always_reads_as_unchanged() {
        // covers: AC-19, AC-21. Both ways a write can fail leave the setting
        // exactly as it was, which is what the sentence claims, so both carry
        // the same line. The true cause survives in `reason` for a log, and
        // that is the only thing that differs.
        for reason in ["could_not_save", "unknown_hotkey"] {
            let e = SettingError::not_saved(reason);
            assert_eq!(e.reason, reason);
            assert_eq!(e.code, Some(NOT_SAVED_CODE));
            assert_eq!(e.message, Some(NOT_SAVED_MESSAGE));
        }
    }

    #[test]
    fn a_failed_read_carries_the_line_that_draws_no_control() {
        // covers: AC-19, AC-21. The screen draws this line and no control at
        // all, so the line has to arrive for it to have anything to draw.
        let e = SettingError::not_read();
        assert_eq!(e.reason, "could_not_read");
        assert_eq!(e.code, Some(NOT_READ_CODE));
        assert_eq!(e.message, Some(NOT_READ_MESSAGE));
    }

    #[test]
    fn the_two_unreachable_refusals_carry_no_line_at_all() {
        // covers: record 0002's fourteenth amendment. Neither can happen on a
        // screen only reachable while signed in, and both happen only while
        // Rust is already closing the window the screen lives in. Handing one
        // a sentence would put a line on screen for a state nobody designed,
        // and the screen's own branch on this is what keeps it blank.
        for e in [SettingError::not_signed_in(), SettingError::not_ready()] {
            assert_eq!(e.code, None);
            assert_eq!(e.message, None);
        }
        assert_eq!(SettingError::not_signed_in().reason, "not_signed_in");
        assert_eq!(SettingError::not_ready().reason, "not_ready");
    }

    #[test]
    fn no_sentence_a_person_reads_lives_outside_this_file() {
        // covers: record 0002's fourteenth amendment, "held in one place in
        // Rust like every other sentence on this surface, so no screen invents
        // its own". A second copy is how two truths start: one gets reworded
        // and the other does not.
        let flat = this_file_flattened();
        assert_eq!(
            flat.matches("Thissettingcouldnotbesaved").count(),
            1,
            "the not-saved sentence appears more than once in settings.rs"
        );
        assert_eq!(
            flat.matches("Thesesettingscouldnotberead").count(),
            1,
            "the not-read sentence appears more than once in settings.rs"
        );

        // And neither screen on this surface holds either. Each draws the code
        // and the sentence it is handed on the command's error, and knows
        // neither one, so a change here reaches a person without a second file
        // having to agree. Same shape as the two guards in mod.rs and
        // error_screen.rs, which read src/main.js for the same reason.
        //
        // **Both screens, from 2026-09-04.** The surface gained a second
        // section that day, the saved Deepgram key on Settings, Transcription,
        // and the fourteenth amendment's Value sourcing rows name AC-12
        // alongside AC-19 and AC-21. Guarding only the first screen would have
        // left the second free to grow the copy this test exists to stop.
        for (name, screen) in [
            (
                "src/dictate/dictation-settings.js",
                include_str!("../../../src/dictate/dictation-settings.js"),
            ),
            (
                "src/dictate/transcription-settings.js",
                include_str!("../../../src/dictate/transcription-settings.js"),
            ),
        ] {
            for sentence in [NOT_SAVED_MESSAGE, NOT_READ_MESSAGE] {
                assert!(
                    !screen.contains(sentence),
                    "{name} now holds a sentence this file owns. Both settings sentences live in Rust and nowhere else (record 0002, fourteenth amendment)"
                );
            }
            for code in [NOT_SAVED_CODE, NOT_READ_CODE] {
                assert!(
                    !screen.contains(code),
                    "{name} now holds an error code this file owns. A screen shows the code it is handed"
                );
            }
        }
    }
}
