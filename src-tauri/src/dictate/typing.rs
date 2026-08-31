//! Typing finalised wording at whatever cursor has focus (record 0002 AC-3,
//! AC-7), and refusing to type into a password field (AC-20).
//!
//! **Two Windows calls, and one of them is the whole of AC-20.** Asking which
//! window has focus is not needed: `SendInput` posts to the foreground window's
//! own input queue, so "wherever the cursor already was" is what the operating
//! system does for us. Asking whether that field is a password field is a
//! separate question, put to Windows UI Automation at the moment of typing, per
//! the record's Value sourcing row for AC-20.
//!
//! **What this file refuses to do.** Record 0002's Risk section says
//! transcribed text is "outside input and is never interpreted". So every
//! character that arrives here is sent as a literal character and nothing else.
//! Nothing is parsed, no character is treated as a command, and no virtual key
//! code is ever derived from the text. `KEYEVENTF_UNICODE` is what makes that
//! structurally true rather than merely intended: it carries a character, not a
//! key, so there is no arrangement of transcribed words that can press Ctrl,
//! Alt, Enter or a function key. A transcript saying "control alt delete" types
//! those three words.
//!
//! **When Windows will not say whether a field is a password field, the text is
//! typed.** That is the record's own rule, and it is deliberate: refusing
//! everywhere UI Automation is unsure would break dictation in ordinary apps,
//! which is most of them. How wide the gap actually is, is spike 2, which the
//! record asks milestone 4 to run and record.

use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
};

/// How many UTF-16 units go in one `SendInput` call.
///
/// Small on purpose, and paired with [`BREATHER`]. Record 0002 names the risk,
/// "a small number of apps handle fast simulated input badly", and on
/// 2026-08-31 it was observed, not guessed: a 28 character phrase handed to
/// Windows in one accepted call landed in the new Notepad as its first 3
/// characters followed by 25 copies of its last character, with EchoScribe not
/// even running. Windows loses nothing; the receiving application translates
/// the backlog late, and every backlogged character resolves to the newest one
/// instead of its own value. Keeping the backlog shallow, few characters per
/// call with a pause between calls, makes that rarer. It cannot make it
/// impossible, and the same sitting proved it: at this pace the new Notepad
/// still collapsed a phrase whenever it stalled longer than the pause, while a
/// classic edit control received every character of every burst intact. So
/// this pace is a mitigation for slow receivers, not a cure, and the cure is a
/// decision, not a tuning: see
/// `docs/evidence/dictate-with-a-hotkey/finding-new-notepad-collapses-injected-unicode.md`.
const BATCH: usize = 8;

/// The pause between one `SendInput` call and the next, within one phrase.
///
/// What it buys is written on [`BATCH`]. What it costs is bounded and small: a
/// 50 character phrase is 7 calls, so 6 pauses, roughly 60 ms added to words
/// that already arrive a beat behind the voice (record 0002's Risk section
/// accepts that beat). Dictation produces a phrase every second or two, never
/// a stream of keystrokes, so this pace is invisible at the cursor.
const BREATHER: std::time::Duration = std::time::Duration::from_millis(10);

/// What happened when a phrase was handed to the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typed {
    /// The characters were sent to the focused window.
    AtTheCursor,
    /// The focused field is a password field, so nothing was sent (AC-20).
    RefusedPasswordField,
}

/// Set up this thread for asking UI Automation questions. Called once, on the
/// thread that will do the typing.
///
/// A failure here is not fatal and is not reported to anybody: it means the
/// password check will answer "cannot tell" for the rest of the session, and
/// the record's rule for "cannot tell" is to type normally. Dictation still
/// works; AC-20's protection is what is lost, which is why it is said out loud
/// in the log rather than swallowed.
pub fn prepare_thread() {
    // MTA, because this thread has no message loop of its own.
    let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if hr.is_err() {
        eprintln!(
            "dictate: could not start COM on the typing thread ({hr:?}), so no password field \
             check is possible for this dictation"
        );
    }
}

/// Type one finalised phrase at the focused cursor, unless that cursor is in a
/// password field.
///
/// The password check runs first and every time, not once per dictation: AC-7
/// lets a person click into another window mid dictation, so the field this
/// phrase is going into is not the field the last one went into.
pub fn type_at_cursor(text: &str) -> Typed {
    if focused_field_is_a_password() {
        return Typed::RefusedPasswordField;
    }
    send(text);
    Typed::AtTheCursor
}

/// Ask Windows UI Automation whether the focused element is a password field.
///
/// Returns `false` when it cannot tell, which is the record's rule and not an
/// oversight: `false` here means "type it", and refusing everywhere UI
/// Automation is silent would break dictation in ordinary applications.
///
/// A fresh automation object each call, deliberately. A cached one goes stale
/// across the focus changes AC-7 invites, and this runs once per phrase rather
/// than once per keystroke, so the cost lands in the right place.
fn focused_field_is_a_password() -> bool {
    unsafe {
        let Ok(automation) =
            CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
        else {
            return false;
        };
        let Ok(focused) = automation.GetFocusedElement() else {
            return false;
        };
        match focused.CurrentIsPassword() {
            Ok(is_password) => is_password.as_bool(),
            // The element exists but will not answer this question. Older and
            // custom-drawn applications do this, and it is the gap spike 2
            // measures.
            Err(_) => false,
        }
    }
}

/// Send the characters, as characters.
///
/// UTF-16 rather than `char`, because that is the unit `KEYEVENTF_UNICODE`
/// takes. A character outside the basic range arrives as two units and is sent
/// as two, in order, which is what Windows expects.
fn send(text: &str) {
    let units: Vec<u16> = text.encode_utf16().collect();
    // PROBE START: temporary, added 2026-08-30. Three running totals and the
    // one line at the end of this function. Counts only, never the characters.
    let (mut handed, mut accepted, mut batches) = (0usize, 0usize, 0usize);
    // PROBE END
    let mut first = true;
    for chunk in units.chunks(BATCH) {
        // The pause sits between calls, never before the first or after the
        // last, so a phrase shorter than one batch pays nothing.
        if !first {
            std::thread::sleep(BREATHER);
        }
        first = false;
        let mut inputs: Vec<INPUT> = Vec::with_capacity(chunk.len() * 2);
        for unit in chunk {
            inputs.push(key_event(*unit, false));
            inputs.push(key_event(*unit, true));
        }
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        // PROBE START
        handed += inputs.len();
        accepted += sent as usize;
        batches += 1;
        // PROBE END
        // Record 0002's own lesson, now standing rule 14 in AGENTS.md: the real
        // effect of this call happens in another program, so the return value
        // says how many events Windows accepted and never that anything was
        // typed. There is nothing here that could check the other half, because
        // the other half is somebody else's document. What is worth catching is
        // the case Windows itself reports: fewer events accepted than handed
        // over, which is a blocked or throttled input queue.
        if sent as usize != inputs.len() {
            eprintln!(
                "dictate: Windows accepted {sent} of {} keystrokes, so part of a phrase may not \
                 have reached the focused window",
                inputs.len()
            );
        }
    }
    // PROBE START: remove these three lines and the two blocks above.
    eprintln!(
        "dictate probe: typed a phrase, handed windows {handed} key events in {batches} calls, \
         windows accepted {accepted}"
    );
    // PROBE END
}

/// One key event carrying a character rather than a key.
///
/// `wVk` is zero and stays zero. A virtual key code here would be a keystroke
/// the person did not say, and transcribed text is never allowed to become one.
fn key_event(unit: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: unit,
                dwFlags: if up {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_UNICODE
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This module's own source, minus its tests.
    fn this_file() -> &'static str {
        include_str!("typing.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn a_character_is_sent_as_a_character_and_never_as_a_key() {
        // covers: AC-3, and record 0002's Risk section, "treat transcribed text
        // as anything but characters to type". This is the property that stops
        // a transcript ever pressing a key: the scan code carries the
        // character and the virtual key code stays empty.
        let down = key_event(b'a' as u16, false);
        let up = key_event(b'a' as u16, true);
        unsafe {
            assert_eq!(down.Anonymous.ki.wVk, Default::default(), "no virtual key");
            assert_eq!(up.Anonymous.ki.wVk, Default::default(), "no virtual key");
            assert_eq!(down.Anonymous.ki.wScan, b'a' as u16);
            assert!(down.Anonymous.ki.dwFlags.contains(KEYEVENTF_UNICODE));
            assert!(!down.Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP));
            assert!(up.Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP));
        }
        assert_eq!(down.r#type, INPUT_KEYBOARD);
    }

    #[test]
    fn nothing_here_maps_text_onto_keys() {
        // covers: AC-3, AC-20. A source guard on the one change that would turn
        // this file from a typist into an interpreter. `VK_` anywhere here
        // would mean some character had been given a key's meaning, and a
        // transcript saying "enter" could then press Enter in somebody's
        // document.
        let source = this_file();
        for forbidden in ["VK_", "VIRTUAL_KEY", "MapVirtualKey", "VkKeyScan"] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. Transcribed text is \
                 outside input and is only ever characters, never keys \
                 (record 0002, Risk)"
            );
        }
    }

    #[test]
    fn nothing_here_can_write_the_transcript_anywhere() {
        // covers: AGENTS.md data rules. Transcribed text goes to two places
        // only, the cursor and the local history for that account. This file is
        // the cursor. It must never become the other one, or a third.
        let source = this_file();
        for forbidden in ["std::fs", "File::", "OpenOptions", "BufWriter", "reqwest"] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. The words a person spoke \
                 may not be written or sent from here"
            );
        }
    }

    #[test]
    fn the_transcript_is_never_put_in_a_log() {
        // covers: AGENTS.md data rules, and record 0002's Risk section. Every
        // message this file prints has to be safe, which means none of them may
        // carry the text. A `{text}` in an error line would put a person's
        // words in stderr, and on a bad day in a support file.
        //
        // The whole source, with no line filter, the same way the two guards
        // above it work. An earlier version of this test only inspected lines
        // beginning with `eprintln!`, which in this file is always the macro on
        // its own line and the message on the next, so the message was never
        // the thing being read. It passed on every log line it existed to
        // check. Scanning the source cannot miss that, and cannot be defeated
        // by how a message happens to be wrapped.
        let source = this_file();
        for forbidden in ["{text}", "{unit}", "{units}", "{phrase}", "{chunk}"] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. The words a person spoke \
                 may not reach stderr, and on a bad day a support file"
            );
        }
    }

    #[test]
    fn a_phrase_longer_than_one_batch_is_still_all_sent() {
        // covers: AC-3. Guards the chunking arithmetic rather than the send: a
        // phrase of 100 characters is 200 key events, which is more than one
        // batch, and every one of them has to be built.
        let text = "a".repeat(100);
        let units: Vec<u16> = text.encode_utf16().collect();
        let batches: Vec<_> = units.chunks(BATCH).collect();
        assert_eq!(batches.len(), 13, "100 units at 8 a batch");
        assert_eq!(
            batches.iter().map(|c| c.len()).sum::<usize>(),
            100,
            "no unit is dropped between batches"
        );
    }

    #[test]
    fn a_character_outside_the_basic_range_becomes_two_units() {
        // covers: AC-3. Deepgram can return anything, and a name or a language
        // with characters above the basic range must not lose them. UTF-16 is
        // the unit KEYEVENTF_UNICODE takes, so this is the right split.
        let units: Vec<u16> = "\u{1F600}".encode_utf16().collect();
        assert_eq!(units.len(), 2, "a surrogate pair, sent in order");
        let text_units: Vec<u16> = "café".encode_utf16().collect();
        assert_eq!(text_units.len(), 4, "an accented character is one unit");
    }

    // PROBE START: temporary, the second instrument, added 2026-08-31. A
    // dictation with Deepgram removed: three fixed phrases, the same shapes as
    // the garbled sitting, through the exact live path, prepare_thread then
    // type_at_cursor. Fixed strings, so no transcript is involved. Run with
    //
    //   cargo test --no-default-features probe_typing -- --ignored --nocapture
    //
    // from src-tauri, then click into Notepad within ten seconds. Corrupted
    // in Notepad means typing corrupts on its own, with Deepgram exonerated.
    // Clean means the corruption needs the live pipeline, or the text arrived
    // already wrong. Delete this whole test to remove it.
    #[test]
    #[ignore]
    fn probe_typing_three_fixed_phrases_wherever_the_cursor_is() {
        std::thread::sleep(std::time::Duration::from_secs(10));
        prepare_thread();
        for phrase in [
            "Hello. Hello. Hello.",
            "In the country of the blind,",
            "a one eyed man is the king.",
            // Deliberately long, to stress the pacing: the collapse observed
            // on 2026-08-31 was timing dependent, so the hardest case has to
            // be in the probe or a clean run proves little.
            "The quick brown fox jumps over the lazy dog, while a second fox \
             waits its turn behind the fence and a third one watches them both.",
        ] {
            let _ = type_at_cursor(phrase);
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
    }
    // PROBE END

    #[test]
    fn the_two_outcomes_are_distinct() {
        // covers: AC-20. Refusing and typing must never be the same value, or
        // the caller cannot tell a blocked phrase from a typed one and the pill
        // would say nothing.
        assert_ne!(Typed::AtTheCursor, Typed::RefusedPasswordField);
    }
}
