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
//!
//! **Two mechanisms, one door, and a list deciding between them** (record
//! 0002, "How characters reach a focused window", settled 2026-08-31).
//! Simulated keystrokes are the default for every app. Receivers proven to
//! collapse backlogged injected keystrokes, held in `collapse_list.rs`, get
//! the direct channel instead: each character posted straight to the focused
//! text box, skipping the shared input queue where the backlog builds. Both
//! run the password check first, both carry characters and never keys, and
//! neither touches the clipboard. The channel was proved before it was wired
//! in, by the spike test at the bottom of this file: the finding's own 205
//! character burst arrived intact in the new Notepad, exact string match, on
//! 2026-08-31.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, PostMessageW, GUITHREADINFO,
    WM_CHAR,
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
    // The collapse list decides which mechanism runs, per phrase, because
    // AC-7 lets focus move between phrases. On the list and addressable
    // means the direct channel; everything else, including every flavour of
    // cannot tell, means the proven default keystrokes.
    if super::collapse_list::focused_window_collapses_injected_keystrokes() {
        if let Some(text_box) = focused_text_box() {
            if !post_chars(text_box, text) {
                // Counts nothing and names nothing: the words themselves may
                // never appear here (see the guards below).
                eprintln!(
                    "dictate: Windows refused a posted character, so part of a phrase may not \
                     have reached the focused window"
                );
            }
            return Typed::AtTheCursor;
        }
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
}

/// Find the text box that has focus inside the foreground window.
///
/// The direct channel needs an exact address in a way `SendInput` never did:
/// simulated keystrokes go into the shared input queue and Windows routes
/// them to whatever has focus, but a posted character message goes to one
/// window and no other. Asking Windows which control holds focus, rather
/// than settling for the top level window, is what keeps "wherever the
/// cursor already was" true on this path too. `None` when Windows will not
/// say, and the caller treats that as cannot tell.
fn focused_text_box() -> Option<HWND> {
    unsafe {
        let window = GetForegroundWindow();
        if window.is_invalid() {
            return None;
        }
        let thread = GetWindowThreadProcessId(window, None);
        if thread == 0 {
            return None;
        }
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        GetGUIThreadInfo(thread, &mut info).ok()?;
        if info.hwndFocus.is_invalid() {
            None
        } else {
            Some(info.hwndFocus)
        }
    }
}

/// The direct channel (record 0002, "How characters reach a focused window",
/// settled 2026-08-31): post one phrase straight to the focused text box,
/// one character message per UTF-16 unit, skipping the shared input queue
/// where a stalled receiver's backlog builds and collapses.
///
/// It is still a character and never a key: `WM_CHAR` carries the character
/// in full and nothing here can press Ctrl, Alt, Enter or a function key.
/// The clipboard is never touched. Runs only for receivers on the collapse
/// list, which the record grows by amendment and evidence, never from here.
///
/// `false` means Windows refused a post, which the caller reports the same
/// way it reports a partly accepted `SendInput`: the call was refused, so
/// part of the phrase may not have arrived.
fn post_chars(text_box: HWND, text: &str) -> bool {
    for unit in text.encode_utf16() {
        // Repeat count 1 in the message's low bits, matching what a real
        // keystroke's character message carries.
        if unsafe { PostMessageW(Some(text_box), WM_CHAR, WPARAM(unit as usize), LPARAM(1)) }
            .is_err()
        {
            return false;
        }
    }
    true
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

    /// The finding's own burst: the four phrases the 2026-08-31 sitting
    /// typed, 205 characters in all, the comparability anchor record 0002's
    /// step 4a names. The ghost probe below types them through `SendInput`;
    /// the spike posts them through the direct channel. Same burst, so the
    /// two proofs stay comparable. The last phrase is deliberately long,
    /// because the collapse observed on 2026-08-31 was timing dependent and
    /// the hardest case has to be in the burst or a clean run proves little.
    const FINDING_BURST: [&str; 4] = [
        "Hello. Hello. Hello.",
        "In the country of the blind,",
        "a one eyed man is the king.",
        "The quick brown fox jumps over the lazy dog, while a second fox \
         waits its turn behind the fence and a third one watches them both.",
    ];

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
    fn nothing_here_can_read_the_receiving_document() {
        // covers: AGENTS.md data rules. The direct channel hands this file
        // the address of a text box in somebody else's program. Writing to
        // it is this file's whole job; reading from it would be this app
        // looking at a document it was never given. No production line here
        // may ever fetch text back out of a window.
        let source = this_file();
        for forbidden in ["WM_GETTEXT", "GetWindowText", "EM_GETTEXT"] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. This file types into \
                 the focused window and must never read out of one"
            );
        }
    }

    #[test]
    fn the_direct_channel_posts_characters_and_never_key_messages() {
        // covers: AC-3, AC-20, as a source guard only. The record's channel
        // row says both mechanisms carry characters and never keys. On the
        // default path that is KEYEVENTF_UNICODE, asserted directly above. On
        // the posted path it is WM_CHAR, and the equivalent betrayal would be
        // a key message: a WM_KEYDOWN built from text would give some word
        // the power to press Enter in somebody's document. The posting lands
        // in another program, so only the ignored spike can watch it live;
        // this guard watches the whole production source instead, and fails
        // if any key message joins the character one.
        let source = this_file();
        for forbidden in [
            "WM_KEYDOWN",
            "WM_KEYUP",
            "WM_SYSKEYDOWN",
            "WM_SYSKEYUP",
            "WM_SYSCHAR",
            "keybd_event",
        ] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. The direct channel \
                 posts characters and may never post a key (record 0002, How \
                 characters reach a focused window)"
            );
        }
    }

    #[test]
    fn neither_mechanism_can_touch_the_clipboard() {
        // covers: AC-3, AC-7, as a source guard only. The record's channel
        // row promises neither mechanism ever touches the clipboard, because
        // paste is the classic shortcut for injecting text and it destroys
        // whatever the person had copied. Every clipboard API and message
        // carries the capital word, so watching the whole production source
        // for it catches the entire family at once; prose here spells it in
        // lowercase so the comments can keep saying the promise out loud.
        let source = this_file();
        for forbidden in ["Clipboard", "WM_PASTE", "WM_COPY", "WM_CUT"] {
            assert!(
                !source.contains(forbidden),
                "typing.rs now mentions `{forbidden}`. Neither typing \
                 mechanism may ever touch the clipboard (record 0002, How \
                 characters reach a focused window)"
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

    #[test]
    fn a_post_to_a_window_that_no_longer_exists_is_reported_as_a_refusal() {
        // covers: AC-3. The direct channel's honesty check: `post_chars` says
        // `false` when Windows refuses a post, and the caller turns that into
        // the "part of a phrase may not have arrived" line. A message-only
        // window stands in for the receiver: real enough that Windows accepts
        // posts to it while it exists, invisible, and never focused, so
        // nothing on the machine running the tests can be typed into. This
        // fails if `post_chars` starts swallowing a refusal, which would turn
        // a lost phrase into a claimed success, the exact shape of fault
        // standing rule 14 exists for.
        use windows::core::w;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
        };

        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("echoscribe post_chars test"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        }
        .expect("a message-only window for the test");

        assert!(
            post_chars(window, "ab"),
            "a live window accepts posted characters"
        );

        unsafe { DestroyWindow(window) }.expect("the test window closes");

        assert!(
            !post_chars(window, "ab"),
            "posting to a window that no longer exists must be reported as a \
             refusal, never as success"
        );
    }

    /// Step 4a's spike (record 0002, "How characters reach a focused
    /// window"). Proves the direct channel delivers the finding's own burst
    /// intact into the new Windows Notepad, before the channel is wired into
    /// `type_at_cursor`. Run from src-tauri, with EchoScribe not running and
    /// an empty new Notepad tab focused:
    ///
    ///   cargo test --no-default-features spike_direct_channel -- --ignored --nocapture
    ///
    /// It waits up to thirty seconds for the new Notepad to hold focus,
    /// proven by the recogniser the tenth amendment named rather than by a
    /// window title. It requires the focused tab to be empty, so nobody's
    /// own words are ever read or printed. Then it posts the burst, reads
    /// the tab back, and compares exact strings, the same proof the finding
    /// used. Intact means build the channel. Anything else means stop and
    /// take the choice back to the user, per the record.
    #[test]
    #[ignore]
    fn spike_direct_channel_the_findings_burst_into_the_new_notepad() {
        use windows::Win32::UI::WindowsAndMessaging::{
            SendMessageW, WM_CLEAR, WM_GETTEXT, WM_GETTEXTLENGTH,
        };
        // The edit control's select-all message, 0x00B1. The windows crate
        // files it under a feature this project does not otherwise need, and
        // it is only used to clean up the spike's own scratch.
        const EM_SETSEL: u32 = 0x00B1;

        let burst_length: usize = FINDING_BURST.iter().map(|p| p.len()).sum();
        assert_eq!(burst_length, 205, "the burst must stay the finding's own");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !super::super::collapse_list::focused_window_collapses_injected_keystrokes() {
            assert!(
                std::time::Instant::now() < deadline,
                "the new Notepad never took focus, so nothing was posted"
            );
            std::thread::sleep(std::time::Duration::from_millis(250));
        }

        let text_box = focused_text_box().expect("Notepad is focused but no text box is");

        let read_back = |hwnd: HWND| -> String {
            unsafe {
                let length = SendMessageW(hwnd, WM_GETTEXTLENGTH, None, None).0 as usize;
                let mut buffer = vec![0u16; length + 1];
                let copied = SendMessageW(
                    hwnd,
                    WM_GETTEXT,
                    Some(WPARAM(buffer.len())),
                    Some(LPARAM(buffer.as_mut_ptr() as isize)),
                )
                .0 as usize;
                String::from_utf16_lossy(&buffer[..copied])
            }
        };

        let before = read_back(text_box);
        assert!(
            before.is_empty(),
            "the focused tab already holds {} characters; give the spike an \
             empty tab so it never reads anybody's words",
            before.encode_utf16().count()
        );

        for phrase in FINDING_BURST {
            assert!(post_chars(text_box, phrase), "Windows refused a post");
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        // Give the receiver a moment to translate everything before reading.
        std::thread::sleep(std::time::Duration::from_secs(2));

        let after = read_back(text_box);
        let expected: String = FINDING_BURST.concat();
        println!(
            "spike: expected {} characters, the tab holds {}",
            expected.encode_utf16().count(),
            after.encode_utf16().count()
        );
        assert_eq!(after, expected, "the burst did not arrive intact");
        println!("spike: intact, exact string match, 205 of 205");

        // Leave nothing behind on success. On failure everything stays on
        // screen as evidence.
        unsafe {
            SendMessageW(text_box, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
            SendMessageW(text_box, WM_CLEAR, None, None);
        }
    }

    #[test]
    fn the_two_outcomes_are_distinct() {
        // covers: AC-20. Refusing and typing must never be the same value, or
        // the caller cannot tell a blocked phrase from a typed one and the pill
        // would say nothing.
        assert_ne!(Typed::AtTheCursor, Typed::RefusedPasswordField);
    }
}
