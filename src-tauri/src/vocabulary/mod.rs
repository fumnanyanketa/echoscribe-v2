//! Teach it your words: the custom vocabulary sent to Deepgram as `keyterm`
//! (record 0005).
//!
//! Three commands and one door. As everywhere else in this app, the interface
//! asks and Rust decides: no command takes an account id, because Rust knows it
//! from the session, and every one of them refuses when nobody is signed in.
//!
//! **The seven sentences a person can read live in this file and nowhere
//! else.** That is record 0002's fourteenth amendment applied to a second
//! feature: the screen draws the code and the sentence it is handed and does
//! not know either one, so a wording change reaches a person without a second
//! file having to agree. A test at the bottom reads the screen's own source and
//! fails if a copy appears there.
//!
//! **The one door out of this feature is [`terms_for_dictation`], and it is not
//! a command.** The dictate feature calls it at the moment the microphone
//! opens, reads the terms once, and uses the same list for the whole dictation
//! including the one reconnect. That is why AC-3 is a simple promise rather
//! than a race, and why nothing here emits an event: there is nothing to push,
//! because the only moment the list matters is a moment the reader chooses.
//!
//! **Nothing in this feature ever logs a word.** Not on a refusal, not on a
//! failure. `set_hotkey` in record 0002 set that rule for a value arriving from
//! outside, and a list of somebody's custom vocabulary is more personal than a
//! hotkey: it can hold colleagues' names, a client list or a diagnosis.

mod rules;
mod store;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use rules::Refused;
use store::Store;

/// The same file every other feature opens. One SQLite file on the person's own
/// machine holds everything EchoScribe persists (AGENTS.md data rules).
const DB_FILE: &str = "echoscribe.db";

/// The mono code and the one sentence of `design/registry.md`'s
/// `Setting error line`, one pair per way a word can be refused (record 0005).
/// Fixed wording, and never carrying anything read off the word itself.
const FULL_CODE: &str = "VOCABULARY_FULL";
const FULL_MESSAGE: &str =
    "There is no room for this, so it was not added. Remove a word to make space.";
const TOO_LONG_CODE: &str = "TERM_TOO_LONG";
const TOO_LONG_MESSAGE: &str = "A word or phrase can be up to 30 characters.";
const ALREADY_CODE: &str = "TERM_ALREADY_ADDED";
const ALREADY_MESSAGE: &str = "This is already in your list.";
const NOT_ALLOWED_CODE: &str = "TERM_NOT_ALLOWED";
const NOT_ALLOWED_MESSAGE: &str = "This must be one word or phrase on a single line.";
const NOT_SAVED_CODE: &str = "TERM_NOT_SAVED";
const NOT_SAVED_MESSAGE: &str = "This could not be saved, so your list is unchanged.";
const NOT_REMOVED_CODE: &str = "TERM_NOT_REMOVED";
const NOT_REMOVED_MESSAGE: &str = "This could not be removed, so your list is unchanged.";
const NOT_READ_CODE: &str = "VOCABULARY_NOT_READ";
const NOT_READ_MESSAGE: &str = "Your list could not be read, so none of it is shown.";

/// Everything this feature keeps for the running app. Managed by Tauri.
pub struct Vocabulary {
    store: Store,
}

/// Open this feature's own connection to the shared database file.
///
/// Runs after the sign-in feature, which creates `account`, because this
/// feature's rows reference it. The same order and the same reason as the
/// shell's store and the dictate feature's.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(Vocabulary { store });
    Ok(())
}

/// Why one of the three commands refused, in the two parts the screen needs.
///
/// `reason` is for a log and for one branch the screen has to make. `code` and
/// `message` are the `Setting error line`, and are `None` for the three
/// refusals that have no line: nobody signed in, the core not up, and an empty
/// word. None of the three can happen on this screen, and a screen with nothing
/// to draw draws nothing rather than inventing a sentence for a state nobody
/// designed (record 0002's fourteenth amendment, applied here).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VocabularyError {
    reason: &'static str,
    code: Option<&'static str>,
    message: Option<&'static str>,
}

impl VocabularyError {
    fn plain(reason: &'static str) -> Self {
        Self {
            reason,
            code: None,
            message: None,
        }
    }

    fn shown(reason: &'static str, code: &'static str, message: &'static str) -> Self {
        Self {
            reason,
            code: Some(code),
            message: Some(message),
        }
    }

    fn not_signed_in() -> Self {
        Self::plain("not_signed_in")
    }

    fn not_ready() -> Self {
        Self::plain("not_ready")
    }

    fn not_read() -> Self {
        Self::shown("could_not_read", NOT_READ_CODE, NOT_READ_MESSAGE)
    }

    fn not_saved() -> Self {
        Self::shown("could_not_save", NOT_SAVED_CODE, NOT_SAVED_MESSAGE)
    }

    fn not_removed() -> Self {
        Self::shown("could_not_remove", NOT_REMOVED_CODE, NOT_REMOVED_MESSAGE)
    }

    /// The sentence for each way `rules::check` can refuse a word. One place,
    /// so the mapping cannot differ between the two commands that need it.
    fn refused(why: Refused) -> Self {
        match why {
            Refused::Empty => Self::plain("empty"),
            Refused::TooLong => Self::shown("too_long", TOO_LONG_CODE, TOO_LONG_MESSAGE),
            Refused::NotOneLine => {
                Self::shown("not_one_line", NOT_ALLOWED_CODE, NOT_ALLOWED_MESSAGE)
            }
            Refused::NoRoom => Self::shown("no_room", FULL_CODE, FULL_MESSAGE),
            Refused::AlreadyAdded => Self::shown("already_added", ALREADY_CODE, ALREADY_MESSAGE),
        }
    }
}

/// One word on the screen, with the identifier a remove needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TermView {
    id: i64,
    term: String,
}

/// What the interface needs to draw the whole screen: the words, newest first,
/// and the two limits (record 0005 AC-1, AC-5).
///
/// The limits are handed out rather than held by the screen, for the same
/// reason `get_hotkey` hands out the two hotkeys: what is allowed is Rust's
/// answer. `room_for_characters` is the longest word that will still be
/// accepted, from the one place that number comes from, so what the screen
/// shows and what an add will take cannot disagree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VocabularyView {
    terms: Vec<TermView>,
    room_for_characters: usize,
    max_term_characters: usize,
}

/// This account's words, and how much room is left.
#[tauri::command]
pub fn get_vocabulary(app: AppHandle) -> Result<VocabularyView, VocabularyError> {
    let (account_id, state) = signed_in(&app)?;
    read(&state, &account_id)
}

/// Add one word or phrase (record 0005 AC-2, and AC-5 to AC-8).
///
/// Treat `term` as hostile input: it is text from outside the core, headed for
/// a query parameter on a request to Deepgram. Everything done with it is in
/// `rules::check`, and nothing about it is ever printed.
///
/// Returns the whole list again rather than the one new word, because the room
/// left has changed too and one answer keeps the screen from drawing a list and
/// a budget that came from two different moments.
#[tauri::command]
pub fn add_vocabulary_term(
    app: AppHandle,
    term: String,
) -> Result<VocabularyView, VocabularyError> {
    let (account_id, state) = signed_in(&app)?;

    // The list as it is now, which is what the word is checked against. Read
    // here rather than trusting anything the screen holds.
    let existing = state.store.terms_for(&account_id).map_err(|e| {
        eprintln!("vocabulary: could not read the list before adding: {e}");
        VocabularyError::not_read()
    })?;
    let words: Vec<String> = existing.iter().map(|t| t.term.clone()).collect();

    let checked = rules::check(&term, &words).map_err(|why| {
        // Nothing from `term` is printed. It came from outside and it has no
        // business in a log; a person's custom vocabulary least of all.
        eprintln!("vocabulary: a word was refused, {why:?}");
        VocabularyError::refused(why)
    })?;

    let now = crate::sign_in::clock::now_iso8601();
    state
        .store
        .add_term(&account_id, &checked, &now)
        .map_err(|e| {
            eprintln!("vocabulary: could not save a word: {e}");
            VocabularyError::not_saved()
        })?;

    read(&state, &account_id)
}

/// Remove one word (record 0005 AC-4).
///
/// The store scopes the delete to the account as well as the row, so an
/// identifier belonging to somebody else removes nothing. Removing nothing is
/// not an error: there is nothing to tell a person and nothing went wrong.
#[tauri::command]
pub fn remove_vocabulary_term(app: AppHandle, id: i64) -> Result<VocabularyView, VocabularyError> {
    let (account_id, state) = signed_in(&app)?;
    state.store.remove_term(&account_id, id).map_err(|e| {
        eprintln!("vocabulary: could not remove a word: {e}");
        VocabularyError::not_removed()
    })?;
    read(&state, &account_id)
}

/// The account and this feature's state, or the refusal that carries no line.
fn signed_in(app: &AppHandle) -> Result<(String, tauri::State<'_, Vocabulary>), VocabularyError> {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return Err(VocabularyError::not_signed_in());
    };
    let Some(state) = app.try_state::<Vocabulary>() else {
        return Err(VocabularyError::not_ready());
    };
    Ok((account_id, state))
}

/// The whole screen's worth of answer, from one read, so the list and the room
/// left always come from the same moment.
fn read(state: &Vocabulary, account_id: &str) -> Result<VocabularyView, VocabularyError> {
    let terms = state.store.terms_for(account_id).map_err(|e| {
        eprintln!("vocabulary: could not read the list: {e}");
        VocabularyError::not_read()
    })?;
    let room = rules::room_for(terms.iter().map(|t| t.term.as_str()));
    Ok(VocabularyView {
        terms: terms
            .into_iter()
            .map(|t| TermView {
                id: t.id,
                term: t.term,
            })
            .collect(),
        room_for_characters: room,
        max_term_characters: rules::MAX_TERM_CHARS,
    })
}

/// The terms for one dictation, read at the moment the microphone opens
/// (record 0005 AC-3, AC-10).
///
/// **The one door between this feature and the dictate feature**, read only,
/// in one direction. Empty on anything at all going wrong, including nobody
/// being signed in, because a failure here must never stop a dictation:
/// accuracy falls back to what it was before this feature existed, which is
/// exactly AC-10's promise.
///
/// The list is filtered through `rules::sendable`, which sends all of it or
/// none of it and never some. See that function for why a shortened list would
/// be worse than no list at all.
pub fn terms_for_dictation(app: &AppHandle) -> Vec<String> {
    let Some(account_id) = crate::sign_in::account_id_from(app) else {
        return Vec::new();
    };
    let Some(state) = app.try_state::<Vocabulary>() else {
        return Vec::new();
    };
    let stored = match state.store.terms_for(&account_id) {
        Ok(terms) => terms,
        Err(e) => {
            eprintln!("vocabulary: could not read the list for this dictation: {e}");
            return Vec::new();
        }
    };
    rules::sendable(stored.into_iter().map(|t| t.term).collect())
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

    #[test]
    fn no_command_here_takes_an_account_id() {
        // covers: AC-9, and AGENTS.md's data rules. A command that accepted one
        // would let the interface name whose words to change.
        let flat = this_file_flattened();
        assert!(
            !flat.contains("account_id:String") && !flat.contains("accountid:String"),
            "a command in vocabulary/mod.rs now takes an account id. Whose words these are \
             is the session's answer, never the interface's"
        );
        // Every route to the account goes through the one helper, and there are
        // two callers: `signed_in`, which the three commands share, and
        // `terms_for_dictation`, which is not a command.
        assert_eq!(
            flat.matches("crate::sign_in::account_id_from").count(),
            2,
            "vocabulary/mod.rs no longer reads the account from the session in exactly the two \
             places it should: the commands' shared helper, and the dictation door"
        );
    }

    #[test]
    fn the_seven_sentences_are_word_for_word_the_records_own() {
        // covers: AC-5, AC-7, AC-8, AC-11, AC-12. They are wording a person
        // reads, so a build may not reword them and neither may a tidy-up.
        // Every character, including the full stop.
        assert_eq!(FULL_CODE, "VOCABULARY_FULL");
        assert_eq!(
            FULL_MESSAGE,
            "There is no room for this, so it was not added. Remove a word to make space."
        );
        assert_eq!(TOO_LONG_CODE, "TERM_TOO_LONG");
        assert_eq!(
            TOO_LONG_MESSAGE,
            "A word or phrase can be up to 30 characters."
        );
        assert_eq!(ALREADY_CODE, "TERM_ALREADY_ADDED");
        assert_eq!(ALREADY_MESSAGE, "This is already in your list.");
        assert_eq!(NOT_ALLOWED_CODE, "TERM_NOT_ALLOWED");
        assert_eq!(
            NOT_ALLOWED_MESSAGE,
            "This must be one word or phrase on a single line."
        );
        assert_eq!(NOT_SAVED_CODE, "TERM_NOT_SAVED");
        assert_eq!(
            NOT_SAVED_MESSAGE,
            "This could not be saved, so your list is unchanged."
        );
        assert_eq!(NOT_REMOVED_CODE, "TERM_NOT_REMOVED");
        assert_eq!(
            NOT_REMOVED_MESSAGE,
            "This could not be removed, so your list is unchanged."
        );
        assert_eq!(NOT_READ_CODE, "VOCABULARY_NOT_READ");
        assert_eq!(
            NOT_READ_MESSAGE,
            "Your list could not be read, so none of it is shown."
        );
    }

    #[test]
    fn the_too_long_sentence_names_the_number_the_rules_enforce() {
        // covers: AC-5. The sentence says 30 characters and `rules` is what
        // refuses at 30. If one moves without the other, a person is told a
        // limit that is not the limit, which is worse than no sentence.
        assert!(
            TOO_LONG_MESSAGE.contains(&rules::MAX_TERM_CHARS.to_string()),
            "the too-long sentence no longer names the limit the rules enforce, \
             which is {}",
            rules::MAX_TERM_CHARS
        );
    }

    #[test]
    fn no_sentence_a_person_reads_lives_outside_this_file() {
        // covers: record 0002's fourteenth amendment, applied here. A second
        // copy is how two truths start: one gets reworded and the other does
        // not. Same shape as the guard in dictate/settings.rs.
        const SCREEN: &str = include_str!("../../../src/vocabulary/vocabulary.js");
        for sentence in [
            FULL_MESSAGE,
            TOO_LONG_MESSAGE,
            ALREADY_MESSAGE,
            NOT_ALLOWED_MESSAGE,
            NOT_SAVED_MESSAGE,
            NOT_REMOVED_MESSAGE,
            NOT_READ_MESSAGE,
        ] {
            assert!(
                !SCREEN.contains(sentence),
                "src/vocabulary/vocabulary.js now holds a sentence this file owns. Every \
                 refusal sentence lives in Rust and nowhere else (record 0005)"
            );
        }
        for code in [
            FULL_CODE,
            TOO_LONG_CODE,
            ALREADY_CODE,
            NOT_ALLOWED_CODE,
            NOT_SAVED_CODE,
            NOT_REMOVED_CODE,
            NOT_READ_CODE,
        ] {
            assert!(
                !SCREEN.contains(code),
                "src/vocabulary/vocabulary.js now holds an error code this file owns. The \
                 screen shows the code it is handed"
            );
        }
    }

    #[test]
    fn the_screen_holds_no_limit_of_its_own() {
        // covers: AC-5. Both limits are handed out on `get_vocabulary`, for the
        // same reason `get_hotkey` hands out the two hotkeys: a screen holding
        // its own copy of 30 or 400 would eventually disagree with the rules,
        // and the disagreement would show as a refusal a person was not warned
        // about.
        const SCREEN: &str = include_str!("../../../src/vocabulary/vocabulary.js");
        for number in [rules::MAX_TERM_CHARS.to_string(), rules::BUDGET.to_string()] {
            assert!(
                !SCREEN.contains(&number),
                "src/vocabulary/vocabulary.js holds the number {number}, which is one of \
                 this feature's limits. The screen draws the limits it is handed"
            );
        }
    }

    #[test]
    fn the_three_refusals_that_cannot_happen_here_carry_no_line() {
        // covers: AC-11, AC-12. Nobody signed in and the core not being up can
        // only happen while Rust is already closing the window this screen
        // lives in, and an empty word cannot arrive because the Add control is
        // unusable while the field is empty. Handing any of them a sentence
        // would put a line on screen for a state nobody designed.
        for e in [
            VocabularyError::not_signed_in(),
            VocabularyError::not_ready(),
            VocabularyError::refused(Refused::Empty),
        ] {
            assert_eq!(e.code, None);
            assert_eq!(e.message, None);
        }
        assert_eq!(VocabularyError::not_signed_in().reason, "not_signed_in");
        assert_eq!(VocabularyError::not_ready().reason, "not_ready");
        assert_eq!(VocabularyError::refused(Refused::Empty).reason, "empty");
    }

    #[test]
    fn every_way_a_word_can_be_refused_has_exactly_one_sentence() {
        // covers: AC-5, AC-7, AC-8. Four of the five kinds are things a person
        // can do something about, so each needs its own line rather than one
        // shared "that did not work". The fifth is the unreachable empty.
        let mapped = [
            (Refused::TooLong, TOO_LONG_CODE),
            (Refused::NotOneLine, NOT_ALLOWED_CODE),
            (Refused::NoRoom, FULL_CODE),
            (Refused::AlreadyAdded, ALREADY_CODE),
        ];
        for (why, code) in mapped {
            let e = VocabularyError::refused(why);
            assert_eq!(e.code, Some(code), "{why:?}");
            assert!(e.message.is_some(), "{why:?} has a code and no sentence");
        }
        // And no two share a sentence, which would make two different problems
        // read as one.
        let mut seen: Vec<&'static str> = mapped
            .iter()
            .map(|(why, _)| VocabularyError::refused(*why).message.expect("a sentence"))
            .collect();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(before, seen.len(), "two refusals share one sentence");
    }

    #[test]
    fn nothing_here_prints_a_word_a_person_typed() {
        // covers: AC-13, and AGENTS.md's rule that these words go to two places
        // only. A log is a file. The only value interpolated into any log line
        // in this file is a machine error or a refusal kind, never `term` and
        // never a stored word.
        let flat = this_file_flattened();
        for forbidden in [
            "{term}",
            "{term:?}",
            "{checked}",
            "{words:?}",
            "{existing:?}",
        ] {
            assert!(
                !flat.contains(forbidden),
                "vocabulary/mod.rs now logs {forbidden}, which puts a person's own word in a \
                 file (record 0005's risk section)"
            );
        }
    }

    #[test]
    fn the_dictation_door_is_the_only_way_out_of_this_feature() {
        // covers: AC-13, and record 0005's interface surface, "It is the one
        // door between the two features and there is deliberately no second."
        // A second public function returning terms would be a second place the
        // words could reach, and the whole point of one door is that there is
        // one thing to audit.
        let flat = this_file_flattened();
        assert_eq!(
            flat.matches("pubfnterms_for_dictation").count(),
            1,
            "terms_for_dictation is no longer declared exactly once"
        );
        // Nothing else public hands out a term outside a command's own view.
        assert!(
            !flat.contains("pubfnterms_for(") && !flat.contains("pubfnall_terms"),
            "vocabulary/mod.rs now has a second public way to read the words. There is one \
             door on purpose (record 0005)"
        );
    }
}
