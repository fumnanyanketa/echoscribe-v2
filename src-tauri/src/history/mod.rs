//! See what you've said before: reading back what record 0002 has been writing
//! (record 0007).
//!
//! One command, and no door. As everywhere else in this app, the interface asks
//! and Rust decides: the command takes no account id, because Rust knows it
//! from the session, and it refuses when nobody is signed in.
//!
//! **This feature stores nothing.** No table, no column, no migration, no
//! write. It is the first in this project of which that is true. What it reads
//! is `dictation`, which the dictate feature owns and created, and
//! `store.rs`'s own comment carries the reasoning and the guard.
//!
//! **The one sentence a person can read lives in this file and nowhere else.**
//! That is record 0002's fourteenth amendment applied to a fourth feature: the
//! screen draws the code and the sentence it is handed and does not know
//! either one, so a wording change reaches a person without a second file
//! having to agree. A test at the bottom reads the screen's own source and
//! fails if a copy appears there.
//!
//! **Nothing here ever logs a transcript.** Not on a failure, not in a
//! diagnostic. Record 0005 set that rule for a person's custom words; these are
//! the words they actually said, so it holds more strongly and not less. A
//! failure logs its machine reason and never a row.

mod store;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use store::{Cursor, Store};

/// The same file every other feature opens. One SQLite file on the person's own
/// machine holds everything EchoScribe persists (AGENTS.md data rules).
const DB_FILE: &str = "echoscribe.db";

/// The mono code and the one sentence of `design/registry.md`'s
/// `Setting error line`, which `History, could not be read` reuses unchanged
/// (record 0007). Fixed wording, and never carrying anything read off a row.
const NOT_READ_CODE: &str = "HISTORY_NOT_READ";
const NOT_READ_MESSAGE: &str = "Your history could not be read, so none of it is shown.";

/// Everything this feature keeps for the running app. Managed by Tauri.
pub struct History {
    store: Store,
}

/// Open this feature's own connection to the shared database file.
///
/// **Runs after `dictate::init`**, because the dictate feature owns `dictation`
/// and creates it, and this feature creates nothing. That ordering is in
/// `lib.rs` with a comment saying so.
pub fn init(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    let store = Store::open(&dir.join(DB_FILE))?;
    app.manage(History { store });
    Ok(())
}

/// Why the one command refused, in the two parts the screen needs.
///
/// `reason` is for a log. `code` and `message` are the `Setting error line`,
/// and are `None` for the two refusals that have no line: nobody signed in and
/// the core not up. Neither can happen on a screen that only exists while
/// somebody is signed in, and a screen with nothing to draw draws nothing
/// rather than inventing a sentence for a state nobody designed (record 0002's
/// fourteenth amendment, applied here).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryError {
    reason: &'static str,
    code: Option<&'static str>,
    message: Option<&'static str>,
}

impl HistoryError {
    fn plain(reason: &'static str) -> Self {
        Self {
            reason,
            code: None,
            message: None,
        }
    }

    fn not_signed_in() -> Self {
        Self::plain("not_signed_in")
    }

    fn not_ready() -> Self {
        Self::plain("not_ready")
    }

    fn not_read() -> Self {
        Self {
            reason: "could_not_read",
            code: Some(NOT_READ_CODE),
            message: Some(NOT_READ_MESSAGE),
        }
    }
}

/// One dictation on the screen (record 0007 AC-2, AC-14).
///
/// Six values and no seventh. There is still no source application, because
/// nothing stores one and record 0007 refuses to start.
///
/// The sixth is `characters`, added 2026-09-04 by record 0007's first
/// amendment. It is a count of characters and not of words: no counting rule
/// for words is true in every language record 0006 offers, and splitting on
/// spaces reports one word for a paragraph of Chinese. Record 0005 counts its
/// vocabulary budget in characters for that same reason and this is its
/// precedent applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DictationView {
    id: i64,
    text: String,
    started_at: String,
    duration_ms: i64,
    language: String,
    /// How many characters were typed, `chars().count()` over `text`, the same
    /// call `vocabulary::rules` counts with, so this project has one meaning of
    /// "a character" and not two.
    ///
    /// Counted here and never in the interface. JavaScript's `String.length`
    /// counts UTF-16 code units, so an emoji or anything outside the basic
    /// range would come out as two there and as one here, and the same
    /// transcript would carry two different numbers depending on which side of
    /// the boundary asked. The pill's chip counts the same way off the same
    /// string while a dictation is running, which is what makes AC-14's "the
    /// same figure" a fact rather than a hope.
    characters: i64,
}

/// What the interface needs to draw the whole screen: one page of dictations,
/// how many match, and whether there are older ones.
///
/// One answer for all three, for the same reason `get_vocabulary` returns the
/// list and the room left together: the rows and the count must come from one
/// moment, or a person reads "12 dictations match" above eleven of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryView {
    rows: Vec<DictationView>,
    /// How many match in total, which is more than this page holds when there
    /// are older ones. The screen shows it only while something is typed.
    total: i64,
    has_older: bool,
}

/// One page of this account's history (record 0007 AC-2, AC-3, AC-5, AC-7,
/// AC-9).
///
/// Every argument is optional and all three are hostile input: `query` is text
/// a person typed, and the two cursor values come back from a screen. The query
/// is bound as a parameter with its wildcards escaped, and the cursor narrows
/// the read and can never widen it, because the account scope is a separate
/// clause that is always there.
///
/// A query of nothing but spaces is no query at all (record 0007, answered at
/// the gate). Anything else is used exactly as it was typed, because a person
/// searching for `the ` may well mean the trailing space.
#[tauri::command]
pub fn get_history(
    app: AppHandle,
    query: Option<String>,
    before_started_at: Option<String>,
    before_id: Option<i64>,
) -> Result<HistoryView, HistoryError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(HistoryError::not_signed_in());
    };
    let Some(state) = app.try_state::<History>() else {
        return Err(HistoryError::not_ready());
    };

    let query = query.filter(|q| !q.trim().is_empty());
    // Both halves of the cursor or neither. A half cursor is a screen sending
    // something nobody designed, and reading it as "no cursor" shows the newest
    // page, which is the state this screen opens in anyway.
    let cursor = match (before_started_at, before_id) {
        (Some(started_at), Some(id)) => Some(Cursor { started_at, id }),
        _ => None,
    };

    let page = state
        .store
        .page(&account_id, query.as_deref(), cursor.as_ref())
        .map_err(|e| {
            // The machine reason only. Nothing from a row and nothing from the
            // query is printed: one is what a person said and the other is what
            // they were looking for.
            eprintln!("history: could not read a page: {e}");
            HistoryError::not_read()
        })?;

    let total = state
        .store
        .count(&account_id, query.as_deref())
        .map_err(|e| {
            eprintln!("history: could not count: {e}");
            HistoryError::not_read()
        })?;

    Ok(HistoryView {
        rows: page
            .rows
            .into_iter()
            .map(|d| DictationView {
                id: d.id,
                // Counted before `text` moves, over the whole stored string:
                // nothing trimmed, no space collapsed, and the joining spaces
                // record 0002 types between phrases counted like any other
                // character. The number answers "how much of my document is
                // this", so it counts what went into the document.
                characters: d.text.chars().count() as i64,
                text: d.text,
                started_at: d.started_at,
                duration_ms: d.duration_ms,
                language: d.language,
            })
            .collect(),
        total,
        has_older: page.has_older,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The screen's own source.
    const SCREEN: &str = include_str!("../../../src/history/history.js");
    /// And its stylesheet, for the one promise that lives there.
    const SCREEN_CSS: &str = include_str!("../../../src/history/history.css");

    #[test]
    fn a_transcript_can_be_selected_so_a_person_can_copy_it_themselves() {
        // covers: AC-13. This is the whole of how somebody gets a transcript
        // out of EchoScribe today, because the Copy action is refused until the
        // user settles it. It is stated rather than left to the browser's
        // default, because a later `user-select: none` on a parent would take
        // it away silently and nothing on screen would say so.
        assert!(
            SCREEN_CSS.contains("user-select: text"),
            "src/history/history.css no longer states that a transcript is \
             selectable. With no Copy action, selecting it is the only way a \
             person can copy what they said (record 0007 AC-13)"
        );
    }

    #[test]
    fn the_screen_holds_no_copy_of_the_one_sentence() {
        // covers: AC-10. Record 0002's fourteenth amendment: the sentence lives
        // in one place in Rust, and the screen draws what it is handed. A copy
        // there means a wording change reaches a person only if two files
        // agree.
        assert!(
            !SCREEN.contains(NOT_READ_MESSAGE),
            "src/history/history.js holds a copy of the read failure sentence. \
             It belongs in this file alone"
        );
        assert!(
            !SCREEN.contains(NOT_READ_CODE),
            "src/history/history.js holds a copy of the read failure code. It \
             belongs in this file alone"
        );
    }

    #[test]
    fn the_screen_never_sets_a_dictation_as_markup() {
        // covers: AC-11, AC-12, and record 0007's Risk, finding 1. A
        // transcription is text from outside the program (AGENTS.md rule 7),
        // and this is the first screen that renders a whole one. Set as HTML it
        // would run script inside the dashboard's web view, which can invoke
        // every command this app has. This guard is not decoration: the screen
        // will be edited again by somebody who wants one bold word.
        for forbidden in [
            "innerHTML",
            "outerHTML",
            "insertAdjacentHTML",
            "document.write",
        ] {
            assert!(
                !SCREEN.contains(forbidden),
                "src/history/history.js uses {forbidden}. Every value on that \
                 screen is a person's own transcript or their search, both of \
                 which are text from outside the program, and both reach the \
                 page through textContent and nothing else"
            );
        }
    }

    #[test]
    fn a_transcript_reads_the_way_it_was_said_whatever_the_script() {
        // covers: AC-11. `dir="auto"` on the element holding a dictation is the
        // whole mechanism: the browser lays each string out from the text it
        // actually holds, so a dictation in Arabic reads right to left and one
        // in English does not. Without it a person is shown their own sentence
        // backwards, and nothing else in this suite would notice.
        assert!(
            SCREEN.contains("text.dir = \"auto\""),
            "src/history/history.js no longer sets dir=\"auto\" on a \
             dictation's text. Record 0006's AC-11 cannot hold without it"
        );
        assert!(
            SCREEN.contains("field.dir = \"auto\""),
            "src/history/history.js no longer sets dir=\"auto\" on the search \
             field, so a person searching in their own script sees what they \
             type laid out the wrong way round"
        );
    }

    #[test]
    fn the_screen_holds_no_list_of_right_to_left_languages() {
        // covers: AC-11. `dir="auto"` is the mechanism and a list beside it is
        // the mistake: a direction guessed from a language would be wrong the
        // moment somebody dictates an English product name in Hebrew. Record
        // 0006 says no such list may exist anywhere in this project.
        for forbidden in ["rtl", "direction:", "\"he\"", "\"ar\"", "\"fa\"", "\"ur\""] {
            assert!(
                !SCREEN.contains(forbidden),
                "src/history/history.js mentions {forbidden}. Direction is the \
                 browser's answer from dir=\"auto\", and no list of right to \
                 left languages may exist anywhere in this project"
            );
        }
    }

    #[test]
    fn both_surfaces_word_the_count_the_same_way() {
        // covers: AC-14, and record 0002's AC-36. One figure reads one way
        // wherever a person meets it: on a history row, and on the pill's chip
        // while the dictation that made that row was running.
        //
        // There are deliberately two copies of the wording and not a shared
        // module. AGENTS.md: feature folders do not import from each other, and
        // something becomes shared only when three features need it, because
        // two is a coincidence. So the two copies are held together by this
        // guard instead, which fails the build the day they drift.
        const PILL: &str = include_str!("../../../src/dictate/pill.js");
        for wording in ["\"1 character\"", "\" characters\""] {
            assert!(
                SCREEN.contains(wording),
                "src/history/history.js no longer holds {wording}. The count's \
                 wording is record 0007's, in its first amendment: the plain \
                 word in full, singular at one"
            );
            assert!(
                PILL.contains(wording),
                "src/dictate/pill.js no longer holds {wording}. The pill's chip \
                 and the history row must word the count the same way, because \
                 they are the same figure about the same dictation"
            );
        }
        // Neither surface counts a word, and neither splits a string to try.
        // The rule is characters, on record 0005's precedent, and the reason is
        // that splitting on spaces reports one word for a paragraph of Chinese.
        // These are the shapes a word count is written in, not the word
        // "words", which is all over the prose in both files.
        for forbidden in ["1 word\"", " words\"", ".split(", "\\s+", "\\W+"] {
            for (name, source) in [("history/history.js", SCREEN), ("dictate/pill.js", PILL)] {
                assert!(
                    !source.contains(forbidden),
                    "src/{name} reaches for a word count ({forbidden}). The \
                     count is characters, counted in Rust: no rule for \
                     counting words is true in every language record 0006 \
                     offers"
                );
            }
        }
    }

    #[test]
    fn neither_surface_counts_the_characters_itself() {
        // covers: AC-14, and record 0002's AC-36. The number is Rust's, from
        // `chars().count()`. `String.length` in the web view counts UTF-16 code
        // units, so an emoji would come out as two there and as one here, and
        // the same transcript would carry two different numbers depending on
        // which side of the boundary asked. `Intl.Segmenter` would be a third
        // answer again, counting graphemes, which record 0007's Still open
        // holds as the user's decision and not a build's.
        const PILL: &str = include_str!("../../../src/dictate/pill.js");
        for forbidden in [
            "text.length",
            "transcript.length",
            "settled.length",
            "Intl.Segmenter",
            "Array.from(",
        ] {
            for (name, source) in [("history/history.js", SCREEN), ("dictate/pill.js", PILL)] {
                assert!(
                    !source.contains(forbidden),
                    "src/{name} measures a transcript itself ({forbidden}). \
                     The count comes from Rust: on the row for history, and on \
                     dictation:text for the pill's chip"
                );
            }
        }
    }

    #[test]
    fn the_screen_writes_the_clipboard_in_exactly_one_place() {
        // covers: AC-13, AC-15. This guard used to forbid the clipboard here
        // outright. The user lifted that on 2026-09-04, and record 0007's first
        // amendment writes the reading into AGENTS.md's data rules as a
        // distinction rather than an exception: the app never routes
        // transcribed text through a third place of its own accord, and a
        // person taking their own text at their own request is not the app
        // routing it. So the ban becomes a count, which is the part that still
        // has to hold: exactly one write, under one press, and nothing else.
        //
        // AC-13 is still what this protects. It promises EchoScribe never puts
        // a transcript on the clipboard "on its own", and one write behind one
        // button is not the app acting on its own. A second write anywhere
        // would be, which is why a second one fails the build.
        assert_eq!(
            SCREEN.matches("navigator.clipboard.writeText").count(),
            1,
            "src/history/history.js must write the clipboard in exactly one \
             place, the Copy action. A second one is a data rule question and \
             not a convenience: AGENTS.md holds transcribed text to the cursor, \
             the local history, and where the person themselves asks for it"
        );
        // Every other route to the clipboard stays shut. These are the ways to
        // reach it without saying `writeText`, so the count above cannot be
        // satisfied while a second, quieter write hides behind one of them.
        for forbidden in [
            "execCommand",
            "ClipboardItem",
            "clipboard.write(",
            "clipboard.readText",
            "oncopy",
            "\"copy\"",
        ] {
            assert!(
                !SCREEN.contains(forbidden),
                "src/history/history.js reaches for the clipboard another way \
                 ({forbidden}). There is one route and it is \
                 navigator.clipboard.writeText behind the Copy action"
            );
        }
    }

    #[test]
    fn no_other_screen_in_this_app_touches_the_clipboard() {
        // covers: AC-15's second sentence, "Nothing else in EchoScribe ever
        // writes to the clipboard". The Copy action is the one place, and the
        // one place is on this screen. This walks every other interface file
        // rather than naming them, so a screen added later is covered the day
        // it arrives and not the day somebody remembers this test.
        let mut checked = 0;
        let mut stack = vec![std::path::PathBuf::from("../src")];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
            for entry in entries {
                let path = entry.expect("cannot read an entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let is_interface = path
                    .extension()
                    .is_some_and(|ext| ext == "js" || ext == "html");
                // The one file allowed to, guarded by the test above.
                let is_the_history_screen = path.ends_with("history.js");
                if !is_interface || is_the_history_screen {
                    continue;
                }
                let source = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
                checked += 1;
                assert!(
                    !source.contains("clipboard") && !source.contains("execCommand"),
                    "{} reaches for the clipboard. EchoScribe writes it in \
                     exactly one place, the Copy action on a history row, \
                     where a person asked for their own text. Everywhere \
                     else, transcribed text goes to the cursor and the local \
                     history and nowhere else (AGENTS.md's data rules; record \
                     0002's clipboard refusal of 2026-08-31, which stands)",
                    path.display()
                );
            }
        }
        // A walk that found nothing would pass silently forever.
        assert!(
            checked > 5,
            "expected to walk the interface files and found {checked}"
        );
    }

    #[test]
    fn nothing_here_prints_a_transcript_or_a_search() {
        // covers: AC-12. Record 0005 set this rule for a person's custom words;
        // these are the words they actually said, so it holds more strongly and
        // not less. Every log line in this file is a machine reason and a
        // rusqlite error, and neither carries a row. The shape to catch is a
        // `{err}` or `{e}` line that has grown a value beside it.
        let body = include_str!("mod.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the file has a non-test half");
        for line in body.lines() {
            let line = line.trim();
            if !line.starts_with("eprintln!") && !line.starts_with("println!") {
                continue;
            }
            for forbidden in ["query", "text", "row", "term", "{account"] {
                assert!(
                    !line.contains(forbidden),
                    "a log line in src-tauri/src/history/mod.rs names \
                     {forbidden}: {line}. Nothing about what a person said, or \
                     what they were looking for, is worth a log line, and a log \
                     is a file"
                );
            }
        }
    }

    #[test]
    fn the_command_takes_no_account_id() {
        // covers: AC-9. Rust knows whose history it is from the session, and a
        // command that accepted an account id would let a screen ask for
        // somebody else's dictations. Every command in this project is on these
        // terms and this is the one that matters most.
        let body = include_str!("mod.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the file has a non-test half");
        assert!(
            !body.contains("account_id: String") && !body.contains("account_id: &str,\n    app"),
            "get_history takes an account id. It must read the account from \
             the session, never from the interface"
        );
        assert!(
            body.contains("crate::sign_in::account_id_from(&app)"),
            "get_history no longer reads the account from the session"
        );
    }

    #[test]
    fn the_two_refusals_that_cannot_happen_here_carry_no_line() {
        // covers: AC-10. Nobody signed in and the core not being up both happen
        // only while Rust is already closing the window this screen lives in. A
        // screen with nothing to draw draws nothing rather than inventing a
        // sentence for a state nobody designed (record 0002's fourteenth
        // amendment).
        for err in [HistoryError::not_signed_in(), HistoryError::not_ready()] {
            assert!(err.code.is_none());
            assert!(err.message.is_none());
        }
        let read = HistoryError::not_read();
        assert_eq!(read.code, Some(NOT_READ_CODE));
        assert_eq!(read.message, Some(NOT_READ_MESSAGE));
    }

    #[test]
    fn the_one_sentence_is_word_for_word_the_records_own() {
        // covers: AC-10. Record 0007 fixes both, and a screen reads them from
        // here. If the wording is ever changed it is changed in the record
        // first, and this is what makes that true rather than hoped for.
        assert_eq!(NOT_READ_CODE, "HISTORY_NOT_READ");
        assert_eq!(
            NOT_READ_MESSAGE,
            "Your history could not be read, so none of it is shown."
        );
    }

    #[test]
    fn the_screen_holds_no_copy_of_the_page_size() {
        // covers: AC-7. How many rows a page holds is Rust's answer, and
        // whether there are older ones arrives on every read. A copy in the
        // screen is a second place the number can be wrong, the same reasoning
        // record 0005 applies to its two limits.
        assert!(
            !SCREEN.contains(&store::PAGE_SIZE.to_string()),
            "src/history/history.js holds the page size. `has_older` is Rust's \
             answer and the screen never counts"
        );
    }
}
