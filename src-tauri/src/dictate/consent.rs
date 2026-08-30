//! Telling blocked-by-Windows apart for real (record 0002, step 2b).
//!
//! cpal on Windows never says "permission denied": a microphone blocked by the
//! Windows privacy setting falls into its catch-all, proven live on 2026-08-30.
//! So when the microphone fails to open for no named reason, this module asks
//! Windows directly whether microphone access is switched off, by reading the
//! same three switches the Windows privacy page writes: the machine-wide
//! microphone toggle, the per-user toggle, and the per-user toggle for desktop
//! apps. Deny on any of the three means blocked. Allow on all three, or the
//! switches being unreadable at all, keeps the honest catch-all, because a
//! switch that cannot be read is not evidence of blocking.
//!
//! **Read only, forever.** Record 0002's Risk section refuses writing to a
//! Windows privacy setting, ever. This file reads three registry values and
//! does nothing else; changing them is the person's act, on the Windows
//! settings page the app can open for them. The switches are a Windows
//! convention, not a contract: if a future Windows moves them, the read fails,
//! and the failure degrades to the catch-all with Try again, which is
//! wrong-but-safe rather than wrong-and-misleading.

use super::microphone::MicError;

/// Sharpen a microphone failure that has no named cause.
///
/// Called once, at the one place dictation opens the microphone. The two kinds
/// the audio layer names reliably (device busy, no device) pass straight
/// through, as does blocked itself if a future audio layer ever learns to say
/// it. Only the catch-all triggers the consent check.
pub fn refine(e: MicError) -> MicError {
    refine_given(e, any_switch_denies())
}

/// The decision itself, separated from the registry so it can be tested.
fn refine_given(e: MicError, denied: bool) -> MicError {
    if e == MicError::Unavailable && denied {
        MicError::BlockedByWindows
    } else {
        e
    }
}

/// Whether any of the three consent switches says deny.
///
/// The three registry values below are the ones the Windows microphone privacy
/// page writes. Each holds the string `Allow` or `Deny`. Anything else,
/// including the value being absent or unreadable, does not count as deny.
#[cfg(windows)]
fn any_switch_denies() -> bool {
    use windows::core::w;
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const STORE: windows::core::PCWSTR = w!(
        r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone"
    );
    const STORE_DESKTOP_APPS: windows::core::PCWSTR = w!(
        r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone\NonPackaged"
    );

    // The machine-wide toggle, the per-user toggle, and the per-user toggle
    // for desktop apps, in the order the privacy page shows them.
    switch_denies(HKEY_LOCAL_MACHINE, STORE)
        || switch_denies(HKEY_CURRENT_USER, STORE)
        || switch_denies(HKEY_CURRENT_USER, STORE_DESKTOP_APPS)
}

/// Read one switch. True only when it was read cleanly and says `Deny`.
#[cfg(windows)]
fn switch_denies(
    root: windows::Win32::System::Registry::HKEY,
    subkey: windows::core::PCWSTR,
) -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegGetValueW, RRF_RT_REG_SZ};

    // "Allow" and "Deny" both fit with room to spare. A longer value comes
    // back as an error, which reads as not-deny, exactly as it should.
    let mut data = [0u16; 16];
    let mut size = (data.len() * std::mem::size_of::<u16>()) as u32;
    let result = unsafe {
        RegGetValueW(
            root,
            subkey,
            w!("Value"),
            RRF_RT_REG_SZ,
            None,
            Some(data.as_mut_ptr() as *mut std::ffi::c_void),
            Some(&mut size),
        )
    };
    if result != ERROR_SUCCESS {
        return false;
    }
    let text_len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    String::from_utf16_lossy(&data[..text_len]) == "Deny"
}

/// The consent switches are Windows only. Off Windows there is nothing to
/// read, so a nameless failure simply stays the catch-all.
#[cfg(not(windows))]
fn any_switch_denies() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_catch_all_can_become_blocked() {
        // covers: AC-15, AC-29. The consent check may sharpen "I do not know"
        // into blocked. It may never overrule a cause the audio layer actually
        // named, because those send a person somewhere specific and correct.
        assert_eq!(
            refine_given(MicError::Unavailable, true),
            MicError::BlockedByWindows
        );
        assert_eq!(
            refine_given(MicError::InUseByAnotherApp, true),
            MicError::InUseByAnotherApp
        );
        assert_eq!(
            refine_given(MicError::NoMicrophoneFound, true),
            MicError::NoMicrophoneFound
        );
        assert_eq!(
            refine_given(MicError::BlockedByWindows, true),
            MicError::BlockedByWindows
        );
    }

    #[test]
    fn no_deny_means_the_honest_catch_all_stands() {
        // covers: AC-15. A switch that says allow, or cannot be read, is not
        // evidence of blocking, so the error stays "could not be opened".
        assert_eq!(
            refine_given(MicError::Unavailable, false),
            MicError::Unavailable
        );
    }

    /// This module's own source, minus its tests.
    fn this_file() -> &'static str {
        include_str!("consent.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn nothing_here_can_write_to_the_registry() {
        // covers: the Risk section's refusal to write a Windows privacy
        // setting, ever. A source guard, not a behaviour test: the writing
        // functions must never be named in this file.
        let source = this_file();
        for forbidden in [
            "RegSetValue",
            "RegCreateKey",
            "RegDeleteKey",
            "RegDeleteValue",
        ] {
            assert!(
                !source.contains(forbidden),
                "consent.rs now mentions `{forbidden}`. This file reads the \
                 consent switches and may never gain a way to change one \
                 (record 0002, Risk)"
            );
        }
    }
}
