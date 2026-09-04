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
use super::settings::SettingError;
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
///
/// It refuses with the settings surface's own error line, `SettingError`,
/// because from 2026-09-04 one of its two readers is that surface: the saved
/// key row on Settings, Transcription draws `SETTINGS_NOT_READ` and no control
/// at all when this fails, the same state and the same words as the Dictation
/// section beside it. Its other reader is the small dark window's router, which
/// only ever asks whether there is a key at all and reads the reason for a log.
#[tauri::command]
pub fn get_deepgram_key_info(app: AppHandle) -> Result<Option<KeyInfo>, SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };
    state
        .store
        .deepgram_key_for(&account_id)
        .map(|saved| saved.map(KeyInfo::from))
        .map_err(|e| {
            eprintln!("dictate: could not read the saved key info: {e}");
            SettingError::not_read()
        })
}

/// Remove both the row and the credential entry. After this, AC-9 holds again:
/// the next hotkey press opens the setup screen instead of the microphone.
///
/// **The row goes first, and that ordering is the whole correctness of this
/// function.** It is Remove on the Settings, Transcription surface from
/// 2026-09-04, and a refused write there says "This setting could not be saved,
/// so it is unchanged.", so every failure has to leave that sentence true.
///
///   * The row will not delete: nothing has been removed anywhere, the account
///     still has its key, and the sentence is exactly true.
///   * The row is gone: the app has no key from this instant, whatever happens
///     next, so `dictation:key_cleared` is emitted here and not after the
///     vault. Record 0004's shell must not be left holding a dashboard for an
///     account with no key.
///   * The row is gone and the entry will not delete: an orphaned entry nobody
///     can reach, which the next save under the same name overwrites. It is
///     said on stderr and it is not a failure a person is shown, because from
///     where they are standing the key is removed and there is nothing they
///     could do about the leftover.
///
/// The old order did the opposite, and both of its failures were dishonest: a
/// vault error returned before the event, leaving the row deleted and the
/// dashboard up over an account with no key, and either error claimed the
/// setting was unchanged when the row had already gone.
#[tauri::command]
pub fn clear_deepgram_key(app: AppHandle) -> Result<(), SettingError> {
    let Some(account_id) = crate::sign_in::account_id_from(&app) else {
        return Err(SettingError::not_signed_in());
    };
    let Some(state) = app.try_state::<Dictate>() else {
        return Err(SettingError::not_ready());
    };

    // The entry name comes from the row when there is one, so an entry saved
    // under an older naming scheme is still the one that gets removed. Read
    // before the row is deleted, because after that there is nothing to read.
    let target = state
        .store
        .deepgram_key_for(&account_id)
        .ok()
        .flatten()
        .map(|saved| saved.credential_target)
        .unwrap_or_else(|| key_vault::target_for(&account_id));

    if let Err(e) = state.store.clear_deepgram_key(&account_id) {
        eprintln!("dictate: could not clear the Deepgram key row: {e}");
        return Err(SettingError::not_saved("could_not_clear"));
    }

    // The other side of `dictation:key_saved`, and it is emitted the moment the
    // row is gone: AC-9 holds again from here, the app is back in a pre-shell
    // state, and record 0004's dashboard is no longer due.
    let _ = app.emit("dictation:key_cleared", json!({}));

    if let Err(e) = key_vault::delete(&target) {
        eprintln!("dictate: could not clear the Deepgram key entry: {e}");
    }
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

    /* ---- The saved key row on Settings, Transcription ------------------
     *
     * Record 0002 AC-12, AC-34 and AC-35, built 2026-09-04 against that
     * record's sixteenth and seventeenth amendments.
     *
     * Several of these are source guards, and each says so. They exist where
     * the promise lives in an ordering or in a call being absent, which is
     * exactly what a running app cannot be asked about cheaply and what a
     * later tidy-up is most likely to undo. What a person actually sees is
     * `/check verify`'s, and the two things needing a real Deepgram key are
     * named in this build's report as permanently manual.
     */

    /// One file's source, minus its Rust tests, flattened, so a guard survives
    /// a formatter or a prettier moving a call across lines. The same helper as
    /// `settings.rs`'s and `shell/mod.rs`'s, and it keeps comments on purpose,
    /// so a guard can tell a call from a comment saying one is absent.
    fn flattened(source: &str) -> String {
        source
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    /// The screen that draws the saved key. Read raw as well as flattened,
    /// because one guard below is about a character that is not ASCII.
    const SCREEN: &str = include_str!("../../../src/dictate/transcription-settings.js");

    #[test]
    fn the_saved_key_row_is_handed_only_what_ac12_allows() {
        // covers: AC-12. `KeyInfo` is the whole of what the interface is ever
        // told about a saved key, so what it carries is what AC-12's "only the
        // last four characters" comes down to. A field added here is a field on
        // a screen, and the entry name in particular must never cross: it is
        // the address of the real key in Windows Credential Manager.
        let info = KeyInfo::from(SavedKey {
            key_last_four: "5f2a".to_string(),
            credential_target: "deepgram:acct_test".to_string(),
            saved_at: "2026-09-04T09:00:00Z".to_string(),
            last_validated_at: Some("2026-09-04T09:00:00Z".to_string()),
        });

        let value = serde_json::to_value(&info).expect("KeyInfo serialises");
        let object = value.as_object().expect("KeyInfo is an object");
        let mut names: Vec<&str> = object.keys().map(String::as_str).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            ["last_four", "last_validated_at", "saved_at"],
            "get_deepgram_key_info now hands the interface a different set of fields. Only these three may ever cross, and none of them is the key (record 0002 AC-12)"
        );
        assert_eq!(object["last_four"], "5f2a");

        let json = serde_json::to_string(&info).expect("KeyInfo serialises");
        assert!(
            !json.contains("deepgram:acct_test"),
            "the credential entry name is on its way to a screen. It is the address of the real key in Windows Credential Manager and it never leaves Rust"
        );
    }

    #[test]
    fn the_row_is_never_told_how_long_the_key_was() {
        // covers: AC-12, as a source guard only. The mask is a fixed run of
        // bullets and record 0002's sixteenth amendment is explicit that it
        // carries no information: a length narrows a secret, and the length is
        // not stored for exactly that reason. The tempting change is to draw
        // one bullet per character, which looks more honest and is the one
        // thing this must never do.
        assert!(
            SCREEN.contains("repeat(20)"),
            "the mask on the saved key row is no longer a fixed run. If it is now drawn from the key's own length, it is telling a person something record 0002 refuses to store (AC-12)"
        );
        let flat = flattened(SCREEN);
        assert!(
            !flat.contains("last_four.length"),
            "the saved key row reads the length of the fragment it was given. Nothing on this surface may be derived from how long the key was"
        );
    }

    #[test]
    fn the_verified_date_is_never_the_date_the_key_was_saved() {
        // covers: AC-12. The caption says "Verified <date>", and the date is
        // `last_validated_at`, the moment Deepgram last accepted the key. When
        // there is none the sentence is dropped. Falling back to `saved_at`
        // would put the word Verified in front of a date nothing had checked,
        // which is a claim rather than a gap.
        // The read, not the word. `flattened` keeps comments on purpose, and
        // this screen's own comment says in as many words that `saved_at` is
        // deliberately not used, which a guard looking for the bare name would
        // count. Both ways the field could actually be read are covered.
        let flat = flattened(SCREEN);
        for read in [".saved_at", "[\"saved_at\"]"] {
            assert!(
                !flat.contains(read),
                "the saved key row reads saved_at. The caption in front of that date says Verified, and being saved is not being verified (record 0002, sixteenth amendment)"
            );
        }
    }

    #[test]
    fn the_pasted_key_is_a_password_field_from_the_first_keystroke() {
        // covers: AC-12, AC-34, as a source guard only. Replace puts a paste
        // field on a light reading surface, which is the first time in this app
        // a key is typed anywhere but the dark window. AC-12's "never shown
        // again" covers the new one being pasted too, and the masking is the
        // field's own type.
        let flat = flattened(SCREEN);
        assert!(
            flat.contains("input.type=\"password\""),
            "the paste field on the saved key row is no longer a password field. The key would then be on screen in plain text, which record 0002 AC-12 refuses"
        );
        assert!(
            !flat.contains("input.type=\"text\""),
            "the paste field on the saved key row is a plain text field"
        );
    }

    #[test]
    fn both_key_commands_hand_out_the_settings_surfaces_one_error_line() {
        // covers: AC-12, AC-19, AC-21, as a source guard only, and it is record
        // 0002's fourteenth amendment held across two files. The Transcription
        // section and the Dictation section beside it say a refused write and a
        // failed read in the same words, from the one place that holds them. A
        // second error shape here is how two truths start: one gets reworded
        // and the other does not.
        let flat = flattened(include_str!("deepgram_key.rs"));
        for signature in [
            "pubfnget_deepgram_key_info(app:AppHandle)->Result<Option<KeyInfo>,SettingError>",
            "pubfnclear_deepgram_key(app:AppHandle)->Result<(),SettingError>",
        ] {
            assert!(
                flat.contains(signature),
                "a command on the saved key row no longer returns SettingError. Its section of Settings would then say a failure in words nothing else on that surface uses (record 0002, fourteenth amendment)"
            );
        }
    }

    #[test]
    fn removing_a_key_deletes_the_row_before_the_entry_and_says_so_in_between() {
        // covers: AC-35, as a source guard only, and this ordering is the whole
        // correctness of Remove. Three things in one order, and each swap is a
        // different lie:
        //
        //   row, then event, then entry.
        //
        // Entry before row was the old order, and a failed entry delete then
        // returned "this setting could not be saved, so it is unchanged" over
        // an account whose row had already gone, with no event emitted, leaving
        // record 0004's shell holding a dashboard for an account with no key.
        // Event after the entry has the same hole one step along. Proving it for
        // real needs a credential store that refuses a delete, which is not
        // something a test can arrange on this machine, so the order is guarded
        // here instead of being re-derived by the next person who reads the
        // function and finds the entry delete more natural first.
        let flat = flattened(include_str!("deepgram_key.rs"));
        let start = flat
            .find("pubfnclear_deepgram_key(app:AppHandle)")
            .expect("clear_deepgram_key is no longer a command in this file");
        let body = &flat[start..];
        let row = body
            .find("state.store.clear_deepgram_key(&account_id)")
            .expect("clear_deepgram_key no longer deletes the row");
        let event = body
            .find("\"dictation:key_cleared\"")
            .expect("clear_deepgram_key no longer says the key is gone");
        let entry = body
            .find("key_vault::delete(&target)")
            .expect("clear_deepgram_key no longer removes the credential entry");

        assert!(
            row < event,
            "clear_deepgram_key says the key is gone before the row is deleted. If the delete then fails, record 0004's shell has already closed the dashboard for an account that still has a key"
        );
        assert!(
            event < entry,
            "clear_deepgram_key removes the credential entry before it says the key is gone. An entry that will not delete then returns a failure and no event, leaving the shell holding a dashboard for an account whose row has already gone"
        );
    }

    #[test]
    fn the_key_can_never_be_removed_without_the_question_being_asked() {
        // covers: AC-35, as a source guard only. Remove sits a few pixels from
        // Replace, and AC-12 means this app can never show the key again, so a
        // misclick destroys something no screen here can give back. The
        // question is the whole of the protection, and the way it gets lost is
        // somebody wiring the row's Remove straight to the command because the
        // extra step reads as friction.
        let flat = flattened(SCREEN);
        assert_eq!(
            flat.matches("invoke(\"clear_deepgram_key\")").count(),
            1,
            "the saved key row calls clear_deepgram_key in more than one place. Exactly one of them can be the one behind the question"
        );

        let start = flat
            .find("functionsavedWell(ctx){")
            .expect("the saved key row no longer has a resting state");
        let end = start
            + flat[start..]
                .find("functionmask()")
                .expect("mask no longer follows savedWell in this screen");
        assert!(
            !flat[start..end].contains("clear_deepgram_key"),
            "Remove on the resting saved key row removes the key straight away. It must set the asking state instead: one misclick would otherwise destroy a key this app can never show again (record 0002 AC-35)"
        );
    }

    #[test]
    fn the_question_hands_the_keyboard_to_cancel_and_not_to_remove() {
        // covers: AC-35, as a source guard only. Once the question is up, the
        // keyboard goes to the way out. A person who pressed Remove with the
        // keyboard has their finger on Enter, and focus landing on the
        // confirming button turns the question into a formality.
        let flat = flattened(SCREEN);
        assert!(
            flat.contains("cancel.dataset.focusFirst=\"true\""),
            "the way out of the remove question is no longer marked as where focus goes"
        );
        assert!(
            flat.contains("querySelector(\"[data-focus-first]\")"),
            "the remove question no longer hands the keyboard to Cancel. A second press of Enter would then destroy the key rather than back out (record 0002 AC-35)"
        );
    }

    #[test]
    fn the_small_window_learns_when_the_key_is_removed() {
        // covers: AC-35, as a source guard only, and it is the one thing record
        // 0002's sixteenth amendment named as owed by the build. Remove takes
        // the dashboard away, because record 0004's invariant stops holding,
        // and the small window is what is revealed. That window's router last
        // drew the at-rest state, which is a deliberately empty page. Without
        // this listener a person who removes their key is looking at a blank
        // 760x540 window with nothing on it and no way forward.
        let flat = flattened(include_str!("../../../src/main.js"));
        assert!(
            flat.contains("listen(\"dictation:key_cleared\""),
            "the small window's router no longer re-reads the state when the key is removed. It last drew the empty at-rest page, so Remove leaves a person looking at a blank 760x540 window (record 0002, sixteenth amendment)"
        );
    }
}
