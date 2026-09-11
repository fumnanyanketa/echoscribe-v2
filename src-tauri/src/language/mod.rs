//! Speak in your language: the transcription language sent to Deepgram
//! (record 0006).
//!
//! Two commands and one door. As everywhere else in this app, the interface
//! asks and Rust decides: neither command takes an account id, because Rust
//! knows it from the session, and both refuse when nobody is signed in.
//!
//! **The two sentences a person can read live in this file and nowhere else**,
//! the same division record 0002's fourteenth amendment fixed. A test at the
//! bottom reads the screen's own source and fails if a copy appears there.
//!
//! **The one door out of this feature is [`language_for_dictation`], and it is
//! not a command.** The dictate feature calls it at the moment the microphone
//! opens and uses the same value for the whole dictation, including the one
//! reconnect. That is AC-4: a change applies to the next dictation and never to
//! one already running, and it is a promise rather than a race because the
//! value is read once.
//!
//! **Nothing here ever sends Deepgram a string that came from the interface.**
//! `catalog` holds 64 literals and `catalog::from_chosen` is what an incoming
//! value has to match before anything is written. See record 0006's risk
//! section: the language ends up in the query of the streaming address, so
//! anything less than a literal match would be a way to add a parameter to
//! every request.

mod catalog;
mod store;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use store::Store;

/// The same file every other feature opens. One SQLite file on the person's own
/// machine holds everything EchoScribe persists (AGENTS.md data rules).
const DB_FILE: &str = "echoscribe.sqlite3";

/// The mono code and the one sentence of `design/registry.md`'s
/// `Setting error line` when a write is refused (record 0006).
const NOT_SAVED_CODE: &str = "LANGUAGE_NOT_SAVED";
const NOT_SAVED_MESSAGE: &str = "This language could not be saved, so it is unchanged.";

/// The same line when the choice cannot be read as the screen opens. In this
/// state **no control is drawn at all**: a list drawn without a chosen value
/// would be showing a person a setting the app cannot read.
const NOT_READ_CODE: &str = "LANGUAGE_NOT_READ";
const NOT_READ_MESSAGE: &str = "Your language could not be read, so none is shown.";

/// Everything this feature keeps for the running app. Managed by Tauri.
pub struct Language {
    store: Store,
}

/// Open this feature's own connection to the shared database file.
///
/// Runs after the sign-in feature, which creates `account`, because this
/// feature's row references it.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(Language { store });
    Ok(())
}

/// Why one of the two commands refused, in the two parts the screen needs.
///
/// `code` and `message` are `None` for the three refusals that have no line:
/// nobody signed in, the core not up, and a language this app does not offer.
/// None of the three can happen on this screen, and a screen with nothing to
/// draw draws nothing rather than inventing a sentence for a state nobody
/// designed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageError {
    reason: &'static str,
    code: Option<&'static str>,
    message: Option<&'static str>,
}

impl LanguageError {
    fn not_signed_in() -> Self {
        Self {
            reason: "not_signed_in",
            code: None,
            message: None,
        }
    }

    fn not_ready() -> Self {
        Self {
            reason: "not_ready",
            code: None,
            message: None,
        }
    }

    fn unknown_language() -> Self {
        Self {
            reason: "unknown_language",
            code: None,
            message: None,
        }
    }

    fn not_read() -> Self {
        Self {
            reason: "could_not_read",
            code: Some(NOT_READ_CODE),
            message: Some(NOT_READ_MESSAGE),
        }
    }

    fn not_saved() -> Self {
        Self {
            reason: "could_not_save",
            code: Some(NOT_SAVED_CODE),
            message: Some(NOT_SAVED_MESSAGE),
        }
    }
}

/// What the interface needs to draw the language setting: which one is in
/// force, and the complete list it may be chosen from (record 0006 AC-1, AC-6).
///
/// The values are codes, not wording. What a person reads for each is the
/// screen's business and is not decided here, the same division `get_hotkey` is
/// on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageChoice {
    chosen: &'static str,
    choices: Vec<&'static str>,
}

/// The chosen language and the 64 it may be chosen from.
#[tauri::command]
pub fn get_transcription_language(app: AppHandle) -> Result<LanguageChoice, LanguageError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(LanguageError::not_signed_in());
    };
    let Some(state) = app.try_state::<Language>() else {
        return Err(LanguageError::not_ready());
    };
    let chosen = state.store.language_for(&account_id).map_err(|e| {
        eprintln!("language: could not read the chosen language: {e}");
        LanguageError::not_read()
    })?;
    Ok(LanguageChoice {
        chosen,
        choices: catalog::OFFERED.to_vec(),
    })
}

/// Choose one of the offered languages (record 0006 AC-2).
///
/// Refusing anything that is not one of the 64 is the only refusal this command
/// has left to make, and it makes it before anything is written. Treat the
/// incoming string as hostile input: it is text from outside the core headed
/// for a query parameter, and the only thing done with it is matching it
/// against the known literals.
#[tauri::command]
pub fn set_transcription_language(app: AppHandle, language: String) -> Result<(), LanguageError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(LanguageError::not_signed_in());
    };
    let Some(state) = app.try_state::<Language>() else {
        return Err(LanguageError::not_ready());
    };
    let Some(chosen) = catalog::from_chosen(&language) else {
        // Nothing from `language` is printed. It came from outside and it has
        // no business in a log; which of the 64 it was not is not information.
        eprintln!("language: a language this app does not offer was refused");
        return Err(LanguageError::unknown_language());
    };

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .save_language(&account_id, chosen, &now)
        .map_err(|e| {
            eprintln!("language: could not save the chosen language: {e}");
            LanguageError::not_saved()
        })?;
    // Nothing else to do. AC-2's "in force from my next dictation" needs no
    // re-arming, because the language is read at the moment the microphone
    // opens rather than held anywhere, which is also what makes AC-4 true.
    Ok(())
}

/// The language for one dictation, read at the moment the microphone opens
/// (record 0006 AC-2, AC-3, AC-4).
///
/// **The one door between this feature and the dictate feature**, read only,
/// in one direction. English on anything at all going wrong, including nobody
/// being signed in, because a failure here must never stop a dictation: it
/// falls back to exactly what record 0002 had fixed in code.
pub fn language_for_dictation(app: &AppHandle) -> &'static str {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return catalog::DEFAULT;
    };
    let Some(state) = app.try_state::<Language>() else {
        return catalog::DEFAULT;
    };
    match state.store.language_for(&account_id) {
        Ok(language) => language,
        Err(e) => {
            eprintln!("language: could not read the language for this dictation: {e}");
            catalog::DEFAULT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This file's own source, minus its tests, flattened so a guard survives a
    /// formatter moving a call across lines.
    fn this_file_flattened() -> String {
        include_str!("mod.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    /// The screen, read for the guards that keep the two sides' jobs apart.
    const SCREEN: &str = include_str!("../../../src/language/language.js");

    #[test]
    fn no_command_here_takes_an_account_id() {
        // covers: AC-8, and AGENTS.md's data rules. A command that accepted one
        // would let the interface name whose language to change.
        let flat = this_file_flattened();
        assert!(
            !flat.contains("account_id:String") && !flat.contains("accountid:String"),
            "a command in language/mod.rs now takes an account id. Whose language this is \
             is the session's answer, never the interface's"
        );
        // The two commands and the dictation door, each reading the session.
        assert_eq!(
            flat.matches("crate::sign_in::account_id_from").count(),
            3,
            "language/mod.rs no longer reads the account from the session in all three \
             places: the two commands, and the dictation door"
        );
    }

    #[test]
    fn the_two_sentences_are_word_for_word_the_records_own() {
        // covers: AC-9, AC-10. They are wording a person reads, so a build may
        // not reword them and neither may a tidy-up.
        assert_eq!(NOT_SAVED_CODE, "LANGUAGE_NOT_SAVED");
        assert_eq!(
            NOT_SAVED_MESSAGE,
            "This language could not be saved, so it is unchanged."
        );
        assert_eq!(NOT_READ_CODE, "LANGUAGE_NOT_READ");
        assert_eq!(
            NOT_READ_MESSAGE,
            "Your language could not be read, so none is shown."
        );
    }

    #[test]
    fn no_sentence_a_person_reads_lives_outside_this_file() {
        // covers: record 0002's fourteenth amendment, applied here. A second
        // copy is how two truths start.
        for sentence in [NOT_SAVED_MESSAGE, NOT_READ_MESSAGE] {
            assert!(
                !SCREEN.contains(sentence),
                "src/language/language.js now holds a sentence this file owns. Both live in \
                 Rust and nowhere else (record 0006)"
            );
        }
        for code in [NOT_SAVED_CODE, NOT_READ_CODE] {
            assert!(
                !SCREEN.contains(code),
                "src/language/language.js now holds an error code this file owns. The screen \
                 shows the code it is handed"
            );
        }
    }

    #[test]
    fn the_screen_has_wording_for_every_language_rust_can_hand_out() {
        // covers: AC-1, AC-6. Rust hands out codes and the screen holds the
        // names, which is this project's division everywhere. The cost of that
        // division is exactly this: two lists that must change together. A code
        // with no name is not drawn at all, so without this guard adding a
        // language to `catalog` would quietly give a person one fewer row than
        // the record promises.
        for code in catalog::OFFERED {
            let key = format!("\"{code}\":");
            assert!(
                SCREEN.contains(&key),
                "src/language/language.js has no wording for the language {code}, which \
                 `catalog::OFFERED` can hand out. A row with no name is not drawn, so this \
                 language would silently not exist for a person (record 0006 AC-1)"
            );
        }
    }

    #[test]
    fn the_screen_offers_no_language_rust_does_not() {
        // covers: AC-6, "the list only ever holds languages EchoScribe can
        // really transcribe in". The other direction of the guard above: a name
        // in the screen with no code behind it would be a row Rust refuses the
        // moment it is pressed, so a person would be shown a language that
        // cannot be chosen.
        let rows = wording_rows();
        assert_eq!(
            rows.len(),
            catalog::OFFERED.len(),
            "src/language/language.js holds {} wording rows and `catalog::OFFERED` holds {}. \
             The two lists are meant to change together",
            rows.len(),
            catalog::OFFERED.len()
        );
        for code in rows {
            assert!(
                catalog::is_offered(&code),
                "src/language/language.js offers the language {code:?}, which \
                 `catalog::OFFERED` does not hold. Rust would refuse it the moment it was \
                 pressed (record 0006 AC-6)"
            );
        }
    }

    /// Every code in the screen's wording table, read off its one shape:
    /// `  "code": "Name",` at exactly two spaces of indent. Deliberately strict
    /// rather than clever, so that a sentence in this file that happens to
    /// start with a quotation mark is not mistaken for a language.
    #[cfg(test)]
    fn wording_rows() -> Vec<String> {
        SCREEN
            .lines()
            .filter_map(|line| {
                let rest = line.strip_prefix("  \"")?;
                let (code, tail) = rest.split_once('"')?;
                // A language tag, and then a name. Anything else is prose.
                if !tail.starts_with(": \"") {
                    return None;
                }
                if code.is_empty()
                    || code.len() > 8
                    || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                {
                    return None;
                }
                Some(code.to_string())
            })
            .collect()
    }

    #[test]
    fn the_three_refusals_that_cannot_happen_here_carry_no_line() {
        // covers: AC-9, AC-10. Nobody signed in and the core not being up can
        // only happen while Rust is already closing the window this screen
        // lives in, and an unoffered language cannot arrive while the rows come
        // from what `get_transcription_language` handed out.
        for e in [
            LanguageError::not_signed_in(),
            LanguageError::not_ready(),
            LanguageError::unknown_language(),
        ] {
            assert_eq!(e.code, None);
            assert_eq!(e.message, None);
        }
        assert_eq!(LanguageError::unknown_language().reason, "unknown_language");
    }

    #[test]
    fn a_write_that_did_not_happen_reads_as_unchanged() {
        // covers: AC-10. The screen does not move the marked language until a
        // write comes back, so the sentence "so it is unchanged" is true rather
        // than hopeful.
        let e = LanguageError::not_saved();
        assert_eq!(e.reason, "could_not_save");
        assert_eq!(e.code, Some(NOT_SAVED_CODE));
        assert_eq!(e.message, Some(NOT_SAVED_MESSAGE));
    }

    #[test]
    fn a_failed_read_carries_the_line_that_draws_no_control() {
        // covers: AC-9. The screen draws this line and no control at all, so
        // the line has to arrive for it to have anything to draw.
        let e = LanguageError::not_read();
        assert_eq!(e.reason, "could_not_read");
        assert_eq!(e.code, Some(NOT_READ_CODE));
        assert_eq!(e.message, Some(NOT_READ_MESSAGE));
    }

    #[test]
    fn nothing_here_prints_the_value_the_interface_asked_for() {
        // covers: record 0006's risk section. `set_hotkey` set this rule: what
        // the value was not is not information, and a log is a file.
        let flat = this_file_flattened();
        for forbidden in ["{language}", "{language:?}", "{chosen}"] {
            assert!(
                !flat.contains(forbidden),
                "language/mod.rs now logs {forbidden}, a value that arrived from outside \
                 the core"
            );
        }
    }

    #[test]
    fn the_dictation_door_is_the_only_way_out_of_this_feature() {
        // covers: record 0006's interface surface, "It is the one door between
        // the two features and there is deliberately no second."
        let flat = this_file_flattened();
        assert_eq!(
            flat.matches("pubfnlanguage_for_dictation").count(),
            1,
            "language_for_dictation is no longer declared exactly once"
        );
    }

    #[test]
    fn choosing_a_language_needs_nothing_re_armed() {
        // covers: AC-2, AC-4, as a source guard. `set_hotkey` has to re-arm the
        // keyboard hook, and it would be natural to copy that shape here. It
        // would be wrong: the language is read at the moment the microphone
        // opens, so there is nothing holding a stale copy, and anything that
        // cached it would break AC-4 by letting a change reach a dictation
        // already running.
        let flat = this_file_flattened();
        assert!(
            !flat.contains("hook::arm") && !flat.contains("Mutex<"),
            "language/mod.rs now caches or re-arms something. The language is read at the \
             moment the microphone opens, and a cached copy would let a change reach a \
             dictation already running (record 0006 AC-4)"
        );
    }
}
