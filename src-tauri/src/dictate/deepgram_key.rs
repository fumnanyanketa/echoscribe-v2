//! The Deepgram key (record 0002 AC-9 to AC-13, milestone 3).
//!
//! One place holds everything about the person's own Deepgram key: what is
//! asked of Deepgram to check one, what each answer means, the fixed wording
//! for every way it can go wrong, and the two outside addresses this feature is
//! allowed to open. Keeping it in one file is what stops a screen inventing its
//! own sentence for a cause.
//!
//! **The key is a secret from the first keystroke.** It arrives from the
//! interface, goes straight into an `Authorization` header and then into
//! Windows Credential Manager, and never comes back out. It is never logged,
//! never put on an event, never returned by a command, and never written to the
//! database. Only its last four characters are stored, for the masked display
//! AC-12 asks for.
//!
//! **The pasted text is hostile input** (AGENTS.md standing rule 7). It is
//! going into an HTTP header, where a stray newline would be header injection,
//! so it is checked for shape before it is used and refused here if it cannot
//! be one.
//!
//! The five things this file embodies were owed decisions that the record did
//! not name. They were answered by the user on 2026-08-30 and written up in
//! `docs/evidence/dictate-with-a-hotkey/milestone-3-decisions-owed.md`.
//! `/architect` owes record 0002 an amendment carrying them into it.

use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use super::key_vault;
use super::store::SavedKey;
use super::Dictate;

/// Deepgram's own documented way to test a key: it answers with details of the
/// key used, and refuses an invalid one. It sends no audio and costs no
/// allowance. Fixed literal, held here; the interface never supplies or sees
/// it, the same rule the Windows privacy page follows.
const DEEPGRAM_AUTH_CHECK: &str = "https://api.deepgram.com/v1/auth/token";

/// Where AC-9's "linking out to get one" goes. Deepgram's signup page, jumping
/// straight to the keys screen, so the person lands on the thing they were sent
/// for. Fixed literal, opened through the system browser.
const DEEPGRAM_SIGNUP_PAGE: &str = "https://console.deepgram.com/signup?jump=keys";

/// AC-13's next step when the allowance has run out: the person's own Deepgram
/// console, where they top up or make a new key. Fixed literal. Pasting the
/// same key again gets them nowhere, which is why this is a different action
/// from the rejected key's.
const DEEPGRAM_CONSOLE_PAGE: &str = "https://console.deepgram.com";

/// How long to wait for Deepgram before calling it unreachable. The person is
/// watching a busy button, so this is short enough not to feel hung and long
/// enough to survive a slow connection.
const CHECK_TIMEOUT: Duration = Duration::from_secs(10);

/// The longest pasted text that will even be looked at. A Deepgram key is far
/// shorter than this; the cap is here so nothing unbounded reaches a header.
const MAX_KEY_LENGTH: usize = 512;

/// Every way Deepgram can stop dictation, and nothing else. Six kinds, six
/// codes, six sentences. The first four were fixed by the answers of
/// 2026-08-30 and two of their codes were already drawn in
/// `design/registry.md`. The last two were added for milestone 4 and are
/// recorded in
/// `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`, points
/// 3 and 4, until `/architect` carries them into record 0002.
///
/// **The name is now narrower than the type.** Four of these really are ways a
/// key can be wrong. `ConnectionLost` is not about the key at all. They live
/// together anyway because record 0002 asks for one thing in particular: the
/// sentences are "held in one place in Rust so no screen can invent its own",
/// and splitting them by cause would give two places to look and two places to
/// drift.
///
/// **Which of these can reach which screen is not decided here.** The setup
/// screen can only ever show what `from_status` produces from
/// `GET /v1/auth/token`, which is the first four. The last two arise only on a
/// live stream and cannot reach it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// Deepgram refused the key. 401.
    Rejected,
    /// The key is real but its project has no credit left. 402.
    ///
    /// Deepgram only ever returns this for a transcription request, so it can
    /// never come back from the check below. It is the live-stream half of
    /// AC-13 and milestone 4 raises it.
    NoAllowance,
    /// Deepgram did not answer at all: no connection, no DNS, or a timeout. The
    /// key is neither accepted nor rejected, and the sentence never blames it.
    Unreachable,
    /// Deepgram answered with something that is none of the above, or the
    /// pasted text could not be used at all. The honest catch-all: it admits it
    /// does not know rather than dressing itself up as one of the others.
    ///
    /// Same shape and same reasoning as the fourth microphone error. Telling
    /// somebody their key is bad when Deepgram was rate limiting them sends
    /// them to replace a key that was fine.
    CheckFailed,
    /// The key is real, and Deepgram accepted it at the setup screen, but it is
    /// not allowed to open a transcription stream.
    ///
    /// This is the gap record 0002 names in its Still open section and hands to
    /// milestone 4 in as many words: `GET /v1/auth/token` proves a key exists,
    /// never that it has the scope to stream. It is a kind of its own rather
    /// than a fold into `Rejected` for the reason this file already applies
    /// twice: Deepgram did accept this key, so saying it did not sends the
    /// person to replace something that was never the problem.
    ///
    /// **Its wording is settled. Its trigger is not.** See `from_stream_status`.
    KeyNotAllowed,
    /// The connection to Deepgram dropped mid dictation and the one reconnect
    /// attempt AC-14 allows did not get it back. Nothing to do with the key,
    /// and its sentence never mentions one.
    ConnectionLost,
}

impl KeyError {
    /// The machine-readable kind that crosses to the interface.
    pub fn kind(self) -> &'static str {
        match self {
            KeyError::Rejected => "deepgram_key_rejected",
            KeyError::NoAllowance => "deepgram_no_allowance",
            KeyError::Unreachable => "deepgram_unreachable",
            KeyError::CheckFailed => "deepgram_check_failed",
            KeyError::KeyNotAllowed => "deepgram_key_not_allowed",
            KeyError::ConnectionLost => "deepgram_connection_lost",
        }
    }

    /// The mono code the screen shows, so a person can quote it. Fixed, not
    /// derived from the kind: two of these were drawn in the registry before
    /// the kinds existed and the drawn wording wins.
    pub fn code(self) -> &'static str {
        match self {
            KeyError::Rejected => "DEEPGRAM_KEY_REJECTED",
            KeyError::NoAllowance => "DEEPGRAM_NO_ALLOWANCE",
            KeyError::Unreachable => "DEEPGRAM_UNREACHABLE",
            KeyError::CheckFailed => "DEEPGRAM_CHECK_FAILED",
            KeyError::KeyNotAllowed => "DEEPGRAM_KEY_NOT_ALLOWED",
            KeyError::ConnectionLost => "DEEPGRAM_CONNECTION_LOST",
        }
    }

    /// One plain sentence of cause. Safe to show: none of them carries anything
    /// read off the pasted key, and none of them blames the key when the
    /// network was at fault.
    pub fn message(self) -> &'static str {
        match self {
            KeyError::Rejected => "Deepgram did not accept this key. Nothing was saved.",
            KeyError::NoAllowance => "This key's Deepgram allowance has run out.",
            KeyError::Unreachable => {
                "EchoScribe could not reach Deepgram, so the key was not checked. Nothing was saved."
            }
            KeyError::CheckFailed => {
                "The check did not succeed and Deepgram did not say why. Nothing was saved."
            }
            KeyError::KeyNotAllowed => "This key is not allowed to transcribe live audio.",
            KeyError::ConnectionLost => {
                "Dictation stopped because the connection to Deepgram was lost."
            }
        }
    }

    /// Which one action goes with this cause (AC-13: "the matching next step
    /// for each"). The interface renders the action; this names it so the
    /// pairing lives beside the wording rather than in a screen.
    pub fn action(self) -> &'static str {
        match self {
            // Pasting the same key again gets a rejected key nowhere new, but a
            // typo is the likeliest cause at the setup screen, so the setup
            // screen keeps the text and offers the check again. Mid dictation,
            // where there is no field, this is Replace key.
            KeyError::Rejected => "replace_key",
            // The only step that helps: top up, or make a new key.
            KeyError::NoAllowance => "open_deepgram_console",
            KeyError::Unreachable | KeyError::CheckFailed => "try_again",
            // A key's permissions are changed in Deepgram's console and
            // nowhere else. Replace key opens a paste field, which is the
            // right door only for somebody making a whole new key; ticking a
            // permission on the key they already have is shorter and starts in
            // the same place.
            KeyError::KeyNotAllowed => "open_deepgram_console",
            // Nothing about the key is wrong, so the step is simply to go
            // again. It goes through `try_start`, like every other way in.
            KeyError::ConnectionLost => "try_again",
        }
    }

    /// What Deepgram's answer means. Split out from the request so every arm is
    /// testable without a network.
    ///
    /// Anything not named here is the catch-all, on purpose: 403 (the key
    /// cannot see the model), 429 (rate limited) and every 5xx are all cases
    /// where saying "your key is bad" would send the person to the wrong place.
    fn from_status(status: u16) -> Option<KeyError> {
        match status {
            200..=299 => None,
            401 => Some(KeyError::Rejected),
            402 => Some(KeyError::NoAllowance),
            _ => Some(KeyError::CheckFailed),
        }
    }

    /// What Deepgram refusing a *streaming* connection means. A different
    /// mapping from `from_status` above, because the two requests do not fail
    /// in the same ways: the check cannot produce 402 at all, and only the
    /// stream can produce a scope failure.
    ///
    /// **One arm of this is decided but unproven, and must not be read as
    /// settled.** Deepgram uses 401 for two different things, an invalid key
    /// and a key whose permissions do not cover the request, and their error
    /// reference documents no websocket behaviour whatsoever (checked
    /// 2026-08-30). So 403 is mapped to `KeyNotAllowed`, on their documented
    /// meaning for it, "project does not have access to the requested model",
    /// and 401 stays `Rejected`. If a scope-limited key turns out to produce
    /// 401 instead, this arm is wrong and a person with a working key is told
    /// it was refused.
    ///
    /// Spike 1 in
    /// `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`
    /// settles it, by pointing a deliberately narrow key at the stream and
    /// recording what came back. Milestone 4 is not done until it has run.
    pub fn from_stream_status(status: u16) -> KeyError {
        match status {
            401 => KeyError::Rejected,
            402 => KeyError::NoAllowance,
            403 => KeyError::KeyNotAllowed,
            _ => KeyError::CheckFailed,
        }
    }
}

/// What a failed `save_deepgram_key` hands back: the named kind, the fixed
/// code, the fixed sentence and the one action. Nothing else, and nothing read
/// off the key.
#[derive(Debug, Clone, Serialize)]
pub struct KeyErrorPayload {
    kind: &'static str,
    code: &'static str,
    message: &'static str,
    action: &'static str,
}

impl From<KeyError> for KeyErrorPayload {
    fn from(e: KeyError) -> Self {
        Self {
            kind: e.kind(),
            code: e.code(),
            message: e.message(),
            action: e.action(),
        }
    }
}

/// Make sure the pasted text can be used as a header value at all, and hand
/// back the trimmed key.
///
/// This is the hostile-input guard. Surrounding whitespace is forgiven, because
/// a paste routinely carries a trailing newline and refusing that would be
/// user-hostile for no safety gain. Anything else that is not printable ASCII
/// is refused without leaving this machine: a key cannot contain it, and a
/// carriage return in a header value is an injection.
fn usable_key(pasted: &str) -> Option<&str> {
    let key = pasted.trim();
    if key.is_empty() || key.len() > MAX_KEY_LENGTH {
        return None;
    }
    // Printable ASCII only. Excludes every control character, so CR and LF
    // cannot reach the header, and excludes anything non-ASCII, which a
    // Deepgram key never contains.
    if !key.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
        return None;
    }
    Some(key)
}

/// The last four characters of the key, for the masked display AC-12 asks for.
/// Counts characters rather than bytes so a key is never cut through the middle
/// of one; `usable_key` has already ruled out non-ASCII, so this is belt and
/// braces against a future format.
fn last_four(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    let start = chars.len().saturating_sub(4);
    chars[start..].iter().collect()
}

/// Ask Deepgram whether this key is good. Sends no audio and costs no
/// allowance.
///
/// The key goes into the header and nowhere else. On every path out of here,
/// including every error path, nothing derived from the key is returned,
/// logged or stored.
fn check_with_deepgram(key: &str) -> Result<(), KeyError> {
    // The same blocking client the sign-in feature builds, reached through the
    // `oauth2` crate that already vendors it. No new dependency: record 0002
    // approves the Deepgram SDK for streaming in milestone 4, and this check is
    // one plain GET that does not need it.
    let http = oauth2::reqwest::blocking::Client::builder()
        .timeout(CHECK_TIMEOUT)
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| KeyError::CheckFailed)?;

    let response = http
        .get(DEEPGRAM_AUTH_CHECK)
        .header("Authorization", format!("Token {key}"))
        .send()
        // No answer at all: no connection, no DNS, or the timeout ran out. The
        // key is neither accepted nor rejected.
        .map_err(|_| KeyError::Unreachable)?;

    match KeyError::from_status(response.status().as_u16()) {
        None => Ok(()),
        Some(e) => Err(e),
    }
}

/// What `get_deepgram_key_info` hands back. Never the key.
#[derive(Debug, Clone, Serialize)]
pub struct KeyInfo {
    /// The last four characters, and only those (AC-12).
    last_four: String,
    saved_at: String,
    last_validated_at: Option<String>,
}

impl From<SavedKey> for KeyInfo {
    fn from(saved: SavedKey) -> Self {
        Self {
            last_four: saved.key_last_four,
            saved_at: saved.saved_at,
            last_validated_at: saved.last_validated_at,
        }
    }
}

/// Check a pasted key against Deepgram and, only if Deepgram accepts it, save
/// it (record 0002 AC-10, AC-11).
///
/// Checking is the only way a key is ever stored: on every failing path nothing
/// is written to the credential store and nothing is written to the database.
/// Returns the last four characters on success, which is the only thing about
/// the key that ever crosses back to the interface.
///
/// Declared `fn`, not `async fn`, so Tauri runs it on a worker thread and the
/// blocking request never sits on the async runtime. The same shape as
/// `start_sign_in`.
#[tauri::command]
pub fn save_deepgram_key(app: AppHandle, key: String) -> Result<String, KeyErrorPayload> {
    // Every command on this surface refuses when nobody is signed in, and
    // nothing is written without knowing whose it is (AGENTS.md data rules).
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(KeyErrorPayload::from(KeyError::CheckFailed));
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(KeyErrorPayload::from(KeyError::CheckFailed));
    };

    // Hostile input, refused before it leaves the machine. The catch-all rather
    // than "rejected", because Deepgram never saw it and saying it did would be
    // a small lie; "the check did not succeed" is true and Try again is right.
    let Some(key) = usable_key(&key) else {
        return Err(KeyErrorPayload::from(KeyError::CheckFailed));
    };

    check_with_deepgram(key)?;

    // Accepted. The key goes to the credential store and the masked remainder
    // to the database. The key itself is never a parameter to the store.
    let target = key_vault::target_for(&account_id);
    if let Err(e) = key_vault::store(&target, key) {
        // The message is generic by construction and carries no fragment of the
        // key. Nothing was saved, so Try again is the honest next step.
        eprintln!("dictate: the Deepgram key could not be stored: {e}");
        return Err(KeyErrorPayload::from(KeyError::CheckFailed));
    }

    let now = crate::sign_in::clock::now_iso8601();
    let masked = last_four(key);
    if let Err(e) = state
        .store
        .save_deepgram_key(&account_id, &masked, &target, &now)
    {
        // Leave nothing half-saved: without the row there is no key as far as
        // AC-9 is concerned, so the vault entry would be an orphan.
        let _ = key_vault::delete(&target);
        eprintln!("dictate: the Deepgram key row could not be written: {e}");
        return Err(KeyErrorPayload::from(KeyError::CheckFailed));
    }

    // A key is now saved, which is the moment record 0004's invariant changes:
    // a person who was in a pre-shell state is now signed in with a key, so the
    // dashboard is due. The shell listens for this. It carries nothing about
    // the key, not even its last four characters, because nothing that listens
    // needs them.
    let _ = app.emit("dictation:key_saved", json!({}));

    Ok(masked)
}

/// The last four characters of the saved key, when it was saved and when it was
/// last checked, or nothing if no key is saved. Never returns the key
/// (record 0002 AC-12).
#[tauri::command]
pub fn get_deepgram_key_info(app: AppHandle) -> Result<Option<KeyInfo>, &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };
    state
        .store
        .deepgram_key_for(&account_id)
        .map(|saved| saved.map(KeyInfo::from))
        .map_err(|e| {
            eprintln!("dictate: could not read the saved key info: {e}");
            "could_not_read"
        })
}

/// Remove both the credential entry and the row. After this, AC-9 holds again:
/// the next hotkey press opens the setup screen instead of the microphone.
#[tauri::command]
pub fn clear_deepgram_key(app: AppHandle) -> Result<(), &'static str> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err("not_signed_in");
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err("not_ready");
    };

    // The entry name comes from the row when there is one, so an entry saved
    // under an older naming scheme is still the one that gets removed.
    let target = state
        .store
        .deepgram_key_for(&account_id)
        .ok()
        .flatten()
        .map(|saved| saved.credential_target)
        .unwrap_or_else(|| key_vault::target_for(&account_id));

    let vault = key_vault::delete(&target);
    // The row goes whatever the vault did, so the app never believes in a key
    // it cannot reach. A vault entry left behind is overwritten by the next
    // save under the same name.
    let row = state.store.clear_deepgram_key(&account_id);

    if let Err(e) = vault {
        eprintln!("dictate: could not clear the Deepgram key entry: {e}");
        return Err("could_not_clear");
    }
    if let Err(e) = row {
        eprintln!("dictate: could not clear the Deepgram key row: {e}");
        return Err("could_not_clear");
    }

    // The other side of `dictation:key_saved`: after this AC-9 holds again, so
    // the app is back in a pre-shell state and record 0004's dashboard is no
    // longer due. The shell listens for this.
    let _ = app.emit("dictation:key_cleared", json!({}));
    Ok(())
}

/// Open Deepgram's signup page, jumping to the keys screen. AC-9's "linking out
/// to get one". Takes nothing and returns nothing: the address is a fixed
/// literal here, so the interface can ask for that one page and no other.
#[tauri::command]
pub fn open_deepgram_signup(app: AppHandle) -> Result<(), &'static str> {
    open_fixed_page(&app, DEEPGRAM_SIGNUP_PAGE)
}

/// Open the person's Deepgram console. AC-13's one action when the allowance
/// has run out. Takes nothing and returns nothing, same as above.
#[tauri::command]
pub fn open_deepgram_console(app: AppHandle) -> Result<(), &'static str> {
    open_fixed_page(&app, DEEPGRAM_CONSOLE_PAGE)
}

/// The one way this feature opens a web address. Both callers pass one of the
/// two literals above and there is no path by which anything else arrives here.
fn open_fixed_page(app: &AppHandle, page: &'static str) -> Result<(), &'static str> {
    if crate::sign_in::account_id_from(app).is_none() {
        return Err("not_signed_in");
    }
    open::that_detached(page).map_err(|e| {
        eprintln!("dictate: could not open the browser: {e}");
        "could_not_open_browser"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_good_answer_is_no_error() {
        assert_eq!(KeyError::from_status(200), None);
        assert_eq!(KeyError::from_status(204), None);
    }

    /// AC-11: an invalid key is rejected, and AC-13's first cause.
    #[test]
    fn unauthorised_is_a_rejected_key() {
        assert_eq!(KeyError::from_status(401), Some(KeyError::Rejected));
    }

    /// AC-13's second cause. Deepgram only returns 402 for a transcription
    /// request, so this arm is milestone 4's; it is mapped and tested now so
    /// milestone 4 wires a trigger rather than writing a new state.
    #[test]
    fn payment_required_is_the_allowance_running_out() {
        assert_eq!(KeyError::from_status(402), Some(KeyError::NoAllowance));
    }

    /// The honest catch-all. None of these may read as a bad key: 403 is a
    /// scope problem, 429 is rate limiting, 5xx is Deepgram's own trouble, and
    /// all three would send somebody to replace a key that was fine.
    #[test]
    fn everything_else_is_the_catch_all_and_never_a_rejected_key() {
        for status in [400, 403, 404, 418, 429, 500, 502, 503] {
            assert_eq!(
                KeyError::from_status(status),
                Some(KeyError::CheckFailed),
                "status {status} must not be dressed up as another cause"
            );
        }
    }

    /// AC-13: the two causes must read differently and must not share a next
    /// step, or the criterion's "matching next step for each" is not met.
    #[test]
    fn the_two_ac13_causes_differ_in_every_part() {
        let rejected = KeyError::Rejected;
        let allowance = KeyError::NoAllowance;
        assert_ne!(rejected.kind(), allowance.kind());
        assert_ne!(rejected.code(), allowance.code());
        assert_ne!(rejected.message(), allowance.message());
        assert_ne!(rejected.action(), allowance.action());
    }

    /// Every kind reads differently from every other, which is what the record
    /// means by "errors that matter and must each read differently".
    #[test]
    fn all_four_kinds_read_differently() {
        let all = [
            KeyError::Rejected,
            KeyError::NoAllowance,
            KeyError::Unreachable,
            KeyError::CheckFailed,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.kind(), b.kind());
                assert_ne!(a.code(), b.code());
                assert_ne!(a.message(), b.message());
            }
        }
    }

    /// The network error must never blame the key, and the key errors must
    /// never blame the network. The registry says so in both directions.
    #[test]
    fn no_sentence_blames_the_wrong_thing() {
        assert!(!KeyError::Unreachable.message().contains("did not accept"));
        assert!(KeyError::Unreachable.message().contains("not checked"));
        assert!(!KeyError::Rejected.message().contains("reach"));
    }

    /// Nothing was saved has to be said out loud on every path where nothing
    /// was saved, because that is the reassurance AC-11 turns on.
    #[test]
    fn every_setup_time_failure_says_nothing_was_saved() {
        for e in [
            KeyError::Rejected,
            KeyError::Unreachable,
            KeyError::CheckFailed,
        ] {
            assert!(
                e.message().contains("Nothing was saved."),
                "{} must say nothing was saved",
                e.code()
            );
        }
    }

    #[test]
    fn surrounding_whitespace_is_forgiven() {
        assert_eq!(usable_key("  abc123  "), Some("abc123"));
        assert_eq!(usable_key("abc123\r\n"), Some("abc123"));
        assert_eq!(usable_key("\tabc123\n"), Some("abc123"));
    }

    /// Hostile input. A newline inside the value is header injection, so it is
    /// refused here and never leaves the machine.
    #[test]
    fn a_key_that_could_inject_a_header_is_refused() {
        assert_eq!(usable_key("abc\r\nX-Evil: 1"), None);
        assert_eq!(usable_key("abc\nX-Evil: 1"), None);
        assert_eq!(usable_key("abc\0def"), None);
        assert_eq!(usable_key("abc def"), None);
    }

    #[test]
    fn empty_and_oversized_pastes_are_refused() {
        assert_eq!(usable_key(""), None);
        assert_eq!(usable_key("   "), None);
        assert_eq!(usable_key("\n\t "), None);
        assert_eq!(usable_key(&"a".repeat(MAX_KEY_LENGTH + 1)), None);
        assert!(usable_key(&"a".repeat(MAX_KEY_LENGTH)).is_some());
    }

    #[test]
    fn non_ascii_is_refused() {
        assert_eq!(usable_key("abc\u{00e9}def"), None);
        assert_eq!(usable_key("\u{1f511}"), None);
    }

    /// AC-12: only the last four are ever kept.
    #[test]
    fn the_masked_remainder_is_exactly_four_characters() {
        assert_eq!(last_four("0123456789abcdef"), "cdef");
        assert_eq!(last_four("abcd"), "abcd");
    }

    /// A key shorter than four characters cannot be a real one, but the masking
    /// must not panic if one ever reaches here.
    #[test]
    fn masking_a_short_key_does_not_panic() {
        assert_eq!(last_four("ab"), "ab");
        assert_eq!(last_four("a"), "a");
        assert_eq!(last_four(""), "");
    }

    /// The masked remainder is never most of the key. Four characters of a
    /// 40-character key is what AC-12 permits and nothing wider.
    #[test]
    fn masking_never_leaks_more_than_four_characters() {
        let key = "a".repeat(40) + "wxyz";
        assert_eq!(last_four(&key).chars().count(), 4);
        assert_eq!(last_four(&key), "wxyz");
    }

    /// The two addresses that leave the machine are fixed literals and are the
    /// only ones this feature can open. If either changes, that is a decision,
    /// not an edit.
    #[test]
    fn the_outside_addresses_are_the_approved_ones() {
        assert_eq!(
            DEEPGRAM_SIGNUP_PAGE,
            "https://console.deepgram.com/signup?jump=keys"
        );
        assert_eq!(DEEPGRAM_CONSOLE_PAGE, "https://console.deepgram.com");
        assert_eq!(
            DEEPGRAM_AUTH_CHECK,
            "https://api.deepgram.com/v1/auth/token"
        );
        for address in [
            DEEPGRAM_SIGNUP_PAGE,
            DEEPGRAM_CONSOLE_PAGE,
            DEEPGRAM_AUTH_CHECK,
        ] {
            assert!(address.starts_with("https://"), "{address} must be https");
        }
    }
}
