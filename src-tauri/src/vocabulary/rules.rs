//! The rules a custom word has to pass, and the budget arithmetic behind them
//! (record 0005 AC-5, AC-6, AC-7, AC-8, AC-14).
//!
//! Everything here is pure: no database, no app handle, no clock. That is
//! deliberate, because this is where the whole correctness of the feature
//! lives and it is the half that can be tested without a running app.
//!
//! **Why the budget is counted in characters.** Deepgram accepts 500 tokens of
//! keyterms per request and returns an error above that, and an error there
//! stops dictation, which is the whole product. This app cannot count
//! Deepgram's tokens: the tokeniser is theirs. So it counts the one thing that
//! is a proven upper bound on them. **No tokeniser ever emits more tokens than
//! there are characters**, because a token is one or more characters. Each
//! character of each term is therefore counted as one token, plus one per term
//! for the separator, and the total is capped at 400, which leaves 100 tokens
//! of headroom inside Deepgram's 500 whatever the script. A count of words
//! would have been safe in English and would have broken dictation in Chinese;
//! see record 0005's decision for the arithmetic.
//!
//! **The number a person is shown and the number an add is checked against are
//! the same number**, from [`room_for`] below, so what the screen says and what
//! the store will accept cannot disagree. Record 0005 promises that in as many
//! words, and it is the reason the screen does no arithmetic of its own.

/// The longest one word or phrase may be, in characters, after trimming
/// (record 0005 AC-5, AC-6). A judgement, not a measurement: long enough for a
/// full name or a drug name, short enough that one entry cannot eat the budget.
pub const MAX_TERM_CHARS: usize = 30;

/// The whole list's budget, counting each term's characters plus one per term.
/// Derived from Deepgram's documented 500 token limit by the upper bound in
/// this file's own notes. Fixed, not a setting.
pub const BUDGET: usize = 400;

/// The most terms there may ever be, whatever the budget allows. Deepgram's own
/// documented ceiling, read 2026-09-04. Practically unreachable: 100 terms
/// inside 400 characters means an average of three characters each. It is here
/// so this app can never send more than Deepgram accepts.
pub const MAX_TERMS: usize = 100;

/// Why a word was refused. Each maps to exactly one sentence a person reads,
/// held in `mod.rs` beside the others, and never to a sentence built here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// Nothing but spaces, or nothing at all. Unreachable from the screen,
    /// because the Add control is unusable while the field is empty, so this
    /// carries no sentence.
    Empty,
    /// Longer than [`MAX_TERM_CHARS`].
    TooLong,
    /// A line break, a tab, another control character, or a character that
    /// could reorder how the term is drawn. See [`is_forbidden`].
    NotOneLine,
    /// The list has no room for it.
    NoRoom,
    /// Already there, ignoring case.
    AlreadyAdded,
}

/// What one term costs against the budget: its characters, plus one for the
/// separator that joins it to the next.
///
/// Counted in `char`s and never in bytes. A byte count would make the budget
/// three times stricter for Cyrillic and Devanagari and four times for CJK,
/// which is the same script bias the character budget exists to avoid.
pub fn cost(term: &str) -> usize {
    term.chars().count() + 1
}

/// What a whole list costs.
pub fn total_cost<'a>(terms: impl IntoIterator<Item = &'a str>) -> usize {
    terms.into_iter().map(cost).sum()
}

/// The longest word that can still be added, which is the number the screen
/// shows and the number an add is checked against (record 0005 AC-1, AC-5).
///
/// It is the remaining budget less the one character the new term's own
/// separator will cost, floored at nothing. So a person told there is room for
/// 12 more characters can add a word of 12 and not one of 13, which is what
/// makes the line on screen a fact rather than an estimate.
pub fn room_for<'a>(terms: impl IntoIterator<Item = &'a str>) -> usize {
    BUDGET.saturating_sub(total_cost(terms)).saturating_sub(1)
}

/// Characters a term may never hold.
///
/// Two groups, and the second is the less obvious one.
///
/// Control characters, which is every line break, carriage return and tab.
/// A term is one word or phrase on one line: pasting a list is refused with a
/// sentence rather than quietly turned into one long term (AC-8).
///
/// And the characters that change how text is laid out without being visible:
/// the Unicode line and paragraph separators, which are line breaks that
/// `char::is_control` does not catch, and the bidirectional controls. Those
/// last ones matter here specifically. This project draws terms with
/// `dir="auto"` so that a person's own script reads correctly, and a bidi
/// override inside a stored term could make it render as something other than
/// what it is, on that very surface. AGENTS.md rule 7 says to treat anything
/// typed or pasted as hostile, and a character that lies about itself on screen
/// is exactly that.
pub fn is_forbidden(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            // Line and paragraph separators.
            '\u{2028}' | '\u{2029}'
            // Bidirectional marks and overrides.
            | '\u{200E}' | '\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
        )
}

/// Check one word against every rule, against the list already stored.
///
/// `existing` is that account's terms as they are now. Returns the term exactly
/// as it should be stored, which is the trimmed original with its capital
/// letters untouched: Deepgram's own guidance is to keep capitals for proper
/// nouns and lower case for common ones, so the casing a person typed is
/// information and is never normalised away. A duplicate is judged ignoring
/// case all the same, because two spellings of one word would take budget from
/// each other for nothing.
///
/// The order of the checks is the order a person would want to be told: what is
/// wrong with the word itself first, then whether it fits, then whether it is
/// already there. Only the last two need the stored list at all.
pub fn check(term: &str, existing: &[String]) -> Result<String, Refused> {
    let trimmed = term.trim();
    if trimmed.is_empty() {
        return Err(Refused::Empty);
    }
    if trimmed.chars().any(is_forbidden) {
        return Err(Refused::NotOneLine);
    }
    if trimmed.chars().count() > MAX_TERM_CHARS {
        return Err(Refused::TooLong);
    }
    if existing.len() >= MAX_TERMS {
        return Err(Refused::NoRoom);
    }
    // The same number the screen was shown, from the one place it comes from.
    if trimmed.chars().count() > room_for(existing.iter().map(String::as_str)) {
        return Err(Refused::NoRoom);
    }
    if existing.iter().any(|one| same_term(one, trimmed)) {
        return Err(Refused::AlreadyAdded);
    }
    Ok(trimmed.to_string())
}

/// Whether two terms are the same word for a person's purposes, which is
/// ignoring case.
///
/// Lowercases both wholly rather than comparing character by character,
/// because some letters do not fold one to one and a per-character comparison
/// gets them wrong: German ß lowercases from SS, which is two characters, so
/// the cheaper loop would call "STRASSE" and "Straße" different words. The cost
/// is two short allocations against a list of at most 100 short terms, which is
/// nothing, and it is the same folding SQLite's own `NOCASE` index cannot do,
/// which is why this check runs first and the index is the second guard.
pub fn same_term(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The terms that may be sent to Deepgram for one dictation, from what is
/// stored (record 0005 AC-3, AC-5).
///
/// **All of them, or none of them, and never some.** A list inside the budget
/// goes as it is. A list over the budget sends nothing at all: dictation
/// working with ordinary accuracy beats dictation not working, and a shortened
/// list would be a silent lie about which words are in force, which record 0005
/// refuses in as many words. The screen's own rules make an over-budget list
/// unreachable, so this is the guard for a database somebody edited by hand.
///
/// A row holding a forbidden character is dropped, on the rule record 0002
/// already applies to a stored hotkey it cannot understand: a stored value that
/// is not one of the ones this app writes is read as nonsense and never as an
/// instruction. Dropping one row rather than the whole list is right here
/// because the row was never legitimately writable, so it is not part of the
/// list a person believes is in force.
pub fn sendable(stored: Vec<String>) -> Vec<String> {
    let kept: Vec<String> = stored
        .into_iter()
        .filter(|term| !term.is_empty() && !term.chars().any(is_forbidden))
        .collect();
    if kept.len() > MAX_TERMS || total_cost(kept.iter().map(String::as_str)) > BUDGET {
        eprintln!(
            "vocabulary: the stored list is outside the budget, so no keyterms are sent for \
             this dictation"
        );
        return Vec::new();
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(terms: &[&str]) -> Vec<String> {
        terms.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn the_three_limits_are_the_ones_the_record_settled() {
        // covers: AC-5. All three are wording-free values a person's dictation
        // depends on, and the budget in particular is derived from Deepgram's
        // 500 token limit rather than chosen. A change here is a change to
        // record 0005 and to the argument that the request is provably inside
        // Deepgram's limit.
        assert_eq!(MAX_TERM_CHARS, 30);
        assert_eq!(BUDGET, 400);
        assert_eq!(MAX_TERMS, 100);
    }

    #[test]
    fn the_worst_possible_list_is_still_inside_deepgrams_limit() {
        // covers: AC-5, and it is the whole reason the budget is counted in
        // characters. The bound has to hold for the worst list the rules allow,
        // in any script, because Deepgram counts tokens and this app cannot.
        // One character is at least one token, so characters plus separators is
        // an upper bound on tokens, and that upper bound must stay under 500.
        for filler in ['a', 'ネ', 'م', '中'] {
            let one: String = std::iter::repeat_n(filler, MAX_TERM_CHARS).collect();
            let mut terms: Vec<String> = Vec::new();
            // Add until the rules refuse, which is the largest list reachable.
            while let Ok(kept) = check(&one_more(&one, terms.len()), &terms) {
                terms.push(kept);
            }
            let worst = total_cost(terms.iter().map(String::as_str));
            assert!(
                worst <= BUDGET,
                "the rules allowed a list costing {worst}, over the {BUDGET} budget"
            );
            assert!(
                worst < 500,
                "the worst list the rules allow costs {worst}, which is not provably inside \
                 Deepgram's 500 token limit"
            );
            assert!(
                terms.len() <= MAX_TERMS,
                "the rules allowed {} terms, over Deepgram's ceiling of {MAX_TERMS}",
                terms.len()
            );
        }
    }

    /// A term of the maximum length, made unique so the duplicate rule does not
    /// end the loop above before the budget does.
    fn one_more(base: &str, n: usize) -> String {
        let tail = n.to_string();
        let head: String = base.chars().take(MAX_TERM_CHARS - tail.len()).collect();
        format!("{head}{tail}")
    }

    #[test]
    fn the_room_shown_is_the_longest_word_that_will_be_accepted() {
        // covers: AC-1, AC-5. Record 0005 promises the screen's number and the
        // store's refusal cannot disagree, so the number has to be exact at the
        // boundary, which is the only place it matters. A word of exactly the
        // room fits; one character more does not.
        //
        // **The room shown can be larger than the longest word allowed**, and
        // that is not a contradiction: 30 characters is the rule about one
        // word, and the room is the rule about the list. A person with 300
        // characters of room still cannot add a 40 character phrase, and they
        // are told so by `TooLong` rather than by `NoRoom`, which is why the
        // two refusals have their own sentences. So the boundary of the budget
        // is only reachable when the room has fallen below the length limit,
        // and that is the case this builds. (An earlier version of this test
        // got this wrong and asserted a 38 character word would be accepted.)
        // A realistic nearly full list: terms of the longest allowed length,
        // added until the room left has fallen below that length. Built from
        // the constants rather than from a hand computed size, so it stays
        // correct if either number moves.
        let mut nearly: Vec<String> = Vec::new();
        while room_for(nearly.iter().map(String::as_str)) >= MAX_TERM_CHARS {
            nearly.push(one_more("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", nearly.len()));
        }
        let room = room_for(nearly.iter().map(String::as_str));
        assert!(
            room < MAX_TERM_CHARS,
            "this test needs the budget to bind before the length rule does, and the room \
             left is {room}"
        );

        let fits: String = "c".repeat(room);
        let one_too_many: String = "c".repeat(room + 1);
        assert!(
            check(&fits, &nearly).is_ok(),
            "a word of exactly the room shown was refused"
        );
        assert_eq!(check(&one_too_many, &nearly), Err(Refused::NoRoom));

        // And the ordinary case, where the length rule binds first: the room is
        // plentiful and the longest allowed word is still accepted.
        let terms = list(&["Fumnanya", "EchoScribe", "Deepgram"]);
        assert!(room_for(terms.iter().map(String::as_str)) > MAX_TERM_CHARS);
        let longest: String = "a".repeat(MAX_TERM_CHARS);
        assert!(check(&longest, &terms).is_ok());
        let over: String = "a".repeat(MAX_TERM_CHARS + 1);
        assert_eq!(
            check(&over, &terms),
            Err(Refused::TooLong),
            "a word over the length limit must be refused for being long, not for the room"
        );
    }

    #[test]
    fn a_full_list_shows_no_room_rather_than_going_negative() {
        // covers: AC-5. The screen turns a zero here into its "no room left"
        // wording and makes Add unusable, so an underflow would be a panic in
        // release and a wrong sentence in debug.
        let big: String = "a".repeat(BUDGET);
        let full = vec![big];
        assert_eq!(room_for(full.iter().map(String::as_str)), 0);
        assert_eq!(check("x", &full), Err(Refused::NoRoom));
    }

    #[test]
    fn the_ends_are_trimmed_and_nothing_else_is_altered() {
        // covers: AC-6, and the capitals half of AC-7. Deepgram's own guidance
        // is to keep capitals for proper nouns, so the casing a person typed is
        // information. Inner spaces are what makes a phrase a phrase.
        assert_eq!(check("  Fumnanya Nketa  ", &[]).unwrap(), "Fumnanya Nketa");
        assert_eq!(check("EchoScribe", &[]).unwrap(), "EchoScribe");
        assert_eq!(check("nova-3", &[]).unwrap(), "nova-3");
    }

    #[test]
    fn nothing_but_spaces_is_refused() {
        // covers: AC-6. Unreachable from the screen, because Add is unusable
        // while the field holds only spaces, and refused here all the same.
        for empty in ["", "   ", "\u{00a0}"] {
            assert_eq!(check(empty, &[]), Err(Refused::Empty));
        }
    }

    #[test]
    fn a_paste_of_several_lines_is_refused_whole() {
        // covers: AC-8. The alternative, taking the first line and dropping the
        // rest, is the silent alteration record 0005 refuses: a person pasting
        // twelve words would get one and no explanation.
        assert_eq!(check("one\ntwo", &[]), Err(Refused::NotOneLine));
        assert_eq!(check("one\r\ntwo", &[]), Err(Refused::NotOneLine));
        assert_eq!(check("one\ttwo", &[]), Err(Refused::NotOneLine));
        assert_eq!(check("one\u{2028}two", &[]), Err(Refused::NotOneLine));
    }

    #[test]
    fn a_character_that_could_reorder_the_term_on_screen_is_refused() {
        // covers: AC-8, AC-13. This surface draws terms with dir="auto" so a
        // person's own script reads correctly, which means a bidi override
        // inside a term could make a stored word render as something other than
        // what it is. AGENTS.md rule 7: anything typed or pasted is hostile.
        for sneaky in [
            "safe\u{202E}drowssap",
            "\u{200F}reversed",
            "wrapped\u{2066}inside\u{2069}",
        ] {
            assert_eq!(check(sneaky, &[]), Err(Refused::NotOneLine), "{sneaky:?}");
        }
    }

    #[test]
    fn a_word_longer_than_the_limit_is_refused_rather_than_shortened() {
        // covers: AC-5, AC-6. "Nothing is ever silently shortened or dropped."
        let long: String = "a".repeat(MAX_TERM_CHARS + 1);
        assert_eq!(check(&long, &[]), Err(Refused::TooLong));
        let exact: String = "a".repeat(MAX_TERM_CHARS);
        assert!(check(&exact, &[]).is_ok());
    }

    #[test]
    fn the_length_limit_counts_characters_and_not_bytes() {
        // covers: AC-5. A byte count would make this rule three times stricter
        // for Cyrillic and four times for CJK, which is exactly the script bias
        // the character budget exists to avoid. 30 Japanese characters are 90
        // bytes and are one legitimate term.
        let cjk: String = "ネ".repeat(MAX_TERM_CHARS);
        assert!(
            cjk.len() > MAX_TERM_CHARS,
            "the test string is not multi-byte"
        );
        assert!(
            check(&cjk, &[]).is_ok(),
            "a term of 30 CJK characters was refused"
        );
    }

    #[test]
    fn the_same_word_in_different_capitals_is_a_duplicate() {
        // covers: AC-7. Two spellings of one word would take budget from each
        // other for nothing, and a person adding "Deepgram" after "deepgram"
        // means the same word.
        let terms = list(&["Deepgram", "Fumnanya Nketa"]);
        for same in ["deepgram", "DEEPGRAM", "DeepGram"] {
            assert_eq!(check(same, &terms), Err(Refused::AlreadyAdded), "{same}");
        }
        // And the casing that is stored is the casing that was typed.
        assert_eq!(check("  NOVA  ", &terms).unwrap(), "NOVA");
    }

    #[test]
    fn case_folding_works_outside_ascii() {
        // covers: AC-7. An accented or Cyrillic name is exactly the kind of word
        // this feature exists for, so the duplicate rule has to hold there too.
        let terms = list(&["Café", "Привет"]);
        assert_eq!(check("café", &terms), Err(Refused::AlreadyAdded));
        assert_eq!(check("привет", &terms), Err(Refused::AlreadyAdded));
        assert!(check("Cafés", &terms).is_ok());
    }

    #[test]
    fn a_list_inside_the_budget_is_sent_exactly_as_stored() {
        // covers: AC-3. Order and content unchanged: what the screen shows is
        // what is sent.
        let stored = list(&["Fumnanya Nketa", "EchoScribe", "nova-3"]);
        assert_eq!(sendable(stored.clone()), stored);
    }

    #[test]
    fn an_over_budget_stored_list_sends_no_terms_rather_than_some() {
        // covers: AC-5. Unreachable from the screen and reachable with a hand
        // edited database. A shortened list would be a silent lie about which
        // words are in force; no list is honest and dictation still works,
        // which is AC-10's promise.
        let big: String = "a".repeat(MAX_TERM_CHARS);
        let too_many: Vec<String> = (0..MAX_TERMS + 5).map(|n| format!("{big}{n}")).collect();
        assert!(sendable(too_many).is_empty());

        let over: Vec<String> = (0..30).map(|n| format!("{big}{n}")).collect();
        assert!(total_cost(over.iter().map(String::as_str)) > BUDGET);
        assert!(sendable(over).is_empty());
    }

    #[test]
    fn a_stored_row_that_could_never_have_been_written_here_is_dropped() {
        // covers: AC-8, AC-14. The character rules make these unwritable, so a
        // row holding one came from outside this app and is read as nonsense
        // rather than as an instruction, which is the rule record 0002 already
        // applies to a stored hotkey. The rest of the list still goes: the bad
        // row was never part of what a person believes is in force.
        let stored = list(&["EchoScribe", "bad\nrow", "", "nova-3"]);
        assert_eq!(sendable(stored), list(&["EchoScribe", "nova-3"]));
    }

    #[test]
    fn an_empty_list_sends_no_terms_at_all() {
        // covers: AC-10. With nothing added, dictation asks Deepgram for
        // exactly what it asked before this feature existed.
        assert!(sendable(Vec::new()).is_empty());
    }
}
