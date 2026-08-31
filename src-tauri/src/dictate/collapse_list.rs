//! The collapse list: receivers proven to collapse backlogged injected
//! keystrokes, and the one question typing asks about them (record 0002,
//! "How characters reach a focused window", settled 2026-08-31).
//!
//! Some applications translate simulated keystrokes late when they stall, and
//! deliver every keystroke that queued during the stall as a copy of the
//! newest one. Receivers *proven* to do that, evidence in
//! `docs/evidence/dictate-with-a-hotkey/finding-new-notepad-collapses-injected-unicode.md`,
//! are typed to through a direct channel instead of simulated keystrokes.
//! This file holds the record's list and answers one question: is the window
//! about to receive this phrase on it?
//!
//! **The list grows only by amendment.** A receiver joins record 0002's list
//! by carrying evidence of the same shape as the finding, never by a quick
//! addition here on the way past a bug. That is the record's rule, restated
//! at the exact place the temptation will be felt.
//!
//! **How a receiver is recognised, and why.** By the Store package identity
//! of the process behind the focused window, called the package family name,
//! settled by the record's tenth amendment. It was chosen over the program
//! file name, which the old and the new Notepad share and any program may
//! take, and over the text box's class name, which other applications may
//! share: either one would let the list grow silently. When Windows will not
//! answer, the answer is "not on the list" and the proven default keystrokes
//! run, the same cannot-tell rule the password check follows.
//!
//! Nothing here reads text, stores anything, or touches the words being
//! typed. It looks at who the receiver is, never at what anybody wrote.

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_SUCCESS};
use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Record 0002's collapse list. Exactly one entry today: the new Windows
/// Notepad, its package family name read from the machine on 2026-08-31
/// rather than assumed. Read the module notes before adding anything here.
const COLLAPSE_LIST: [&str; 1] = ["Microsoft.WindowsNotepad_8wekyb3d8bbwe"];

/// Is the window about to receive typed text proven to collapse backlogged
/// injected keystrokes?
///
/// Asked at the moment of typing, once per phrase, the same moment the
/// password check asks its question, because AC-7 lets focus move between
/// phrases. `false` covers both honest answers: the receiver is not on the
/// list, and Windows would not say who the receiver is. Cannot tell means
/// the default keystrokes run.
pub fn focused_window_collapses_injected_keystrokes() -> bool {
    match package_family_of_the_focused_window() {
        Some(name) => COLLAPSE_LIST.iter().any(|entry| *entry == name),
        None => false,
    }
}

/// The package family name of the process behind the foreground window, or
/// `None` when Windows will not say: no foreground window, a process this
/// one may not inspect, or a program with no Store identity at all, which is
/// most desktop programs.
fn package_family_of_the_focused_window() -> Option<String> {
    unsafe {
        let window = GetForegroundWindow();
        if window.is_invalid() {
            return None;
        }
        let mut process_id = 0u32;
        GetWindowThreadProcessId(window, Some(&mut process_id));
        if process_id == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        // 64 is the documented ceiling for a package family name, doubled so
        // a change at Microsoft's end degrades to "cannot tell" rather than a
        // truncated name that matches nothing.
        let mut buffer = [0u16; 128];
        let mut length = buffer.len() as u32;
        let result = GetPackageFamilyName(process, &mut length, Some(PWSTR(buffer.as_mut_ptr())));
        let _ = CloseHandle(process);
        if result != ERROR_SUCCESS || length == 0 {
            return None;
        }
        // `length` counts the closing zero, which is not part of the name.
        Some(String::from_utf16_lossy(&buffer[..length as usize - 1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This module's own source, minus its tests.
    fn this_file() -> &'static str {
        include_str!("collapse_list.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn the_list_holds_exactly_what_the_record_names() {
        // covers: AC-3, and record 0002's rule that a receiver joins only by
        // amendment. One entry, the new Windows Notepad, by the identity the
        // tenth amendment wrote down. A second entry appearing here without a
        // matching amendment is the "quick addition in code" the record
        // forbids.
        assert_eq!(COLLAPSE_LIST, ["Microsoft.WindowsNotepad_8wekyb3d8bbwe"]);
    }

    #[test]
    fn nothing_here_can_read_a_document_or_write_anywhere() {
        // covers: AGENTS.md data rules. This file identifies the receiver and
        // must never grow the ability to read what anybody wrote, store what
        // it learned, or send it anywhere. The same whole-source guard shape
        // typing.rs carries, with no line filter.
        let source = this_file();
        for forbidden in [
            "WM_GETTEXT",
            "GetWindowText",
            "std::fs",
            "File::",
            "OpenOptions",
            "BufWriter",
            "reqwest",
        ] {
            assert!(
                !source.contains(forbidden),
                "collapse_list.rs now mentions `{forbidden}`. This file asks \
                 who the receiver is and may never read or keep anything else"
            );
        }
    }
}
