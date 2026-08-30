//! The two sounds the pill makes: one when the microphone opens, one when it
//! closes.
//!
//! Record 0002 AC-2: they come from the machine's own sound scheme, played by
//! name. Nothing ships with the app. The names tried first are the ones Windows
//! uses for a device being plugged in and unplugged, with the alert pair the
//! record names (`SystemAsterisk` on open, `SystemExclamation` on close) behind
//! them. Both are less apt, which is why they are only a fallback.
//!
//! **`PlaySoundW` reporting success does not mean a person heard anything.**
//! That assumption used to live in this file, and it cost both sounds on
//! exactly the machines the fallback exists for. Windows resolves a name like
//! `DeviceDisconnect` through the sound scheme in the registry. Setting that
//! event to `(None)` on the Windows sound page leaves the entry present and set
//! to nothing. With `SND_NODEFAULT` the call then plays silence and still
//! reports success, so the fallback was never reached and both sounds went
//! quiet. Observed live on 2026-08-30; see
//! `docs/evidence/dictate-with-a-hotkey/AC-2-sound-fallback-never-fires.md`.
//!
//! A name that is absent from the scheme altogether does report failure, so the
//! return value is worth keeping. It is one of two checks, not the whole
//! answer: the scheme entry must name a sound, and the call must then succeed.
//!
//! Known gap, deliberately not covered here: an entry naming a sound file that
//! has since been deleted would also play silence. That is a different machine
//! from the one this fixes, and it is a question for the record rather than a
//! guess to make in this file.
//!
//! Milestone 1 owes the answer to "which pair actually plays" back to the
//! record's Still open section. One line goes to stderr the first time each
//! sound is asked for, saying which name won, so that spike can be read off a
//! normal run. No audio is ever recorded, written or logged; this only ever
//! names a system sound. The sound scheme is read here and never written.

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_NODEFAULT};

/// Play the "microphone opened" sound, if `enabled`.
pub fn play_open(enabled: bool) {
    if enabled {
        play_first_that_works("DeviceConnect", "SystemAsterisk", "open");
    }
}

/// Play the "microphone closed" sound, if `enabled`.
pub fn play_close(enabled: bool) {
    if enabled {
        play_first_that_works("DeviceDisconnect", "SystemExclamation", "close");
    }
}

/// What the machine's sound scheme holds for one event name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SchemeEntry {
    /// The entry names a sound.
    NamesASound,
    /// The entry is there and set to nothing. This is `(None)` on the Windows
    /// sound page, and it is the case that used to slip through.
    Silent,
    /// The entry could not be read. Not evidence either way, so the name is
    /// still worth trying, the same way `consent.rs` treats a switch it cannot
    /// read as not-deny.
    Unknown,
}

/// Try `preferred`, then `fallback`, and report which one actually made a
/// sound. Prints once per sound per run, for the record's open spike.
fn play_first_that_works(preferred: &str, fallback: &str, which: &str) {
    let outcome = choose(preferred, fallback, &mut play_alias);
    report_once(which, outcome);
}

/// The order the two names are tried in, kept apart from the playing so it can
/// be tested. `attempt` answers whether that name actually produced a sound,
/// which is not the same question as whether Windows accepted the name.
fn choose(preferred: &str, fallback: &str, attempt: &mut dyn FnMut(&str) -> bool) -> &'static str {
    if attempt(preferred) {
        "device event name"
    } else if attempt(fallback) {
        "alert fallback"
    } else {
        "nothing played"
    }
}

/// Whether this name produced a sound. Both checks have to pass.
fn play_alias(alias: &str) -> bool {
    // An entry set to nothing plays silence and still reports success, so it is
    // ruled out before Windows is asked at all.
    if scheme_entry(alias) == SchemeEntry::Silent {
        return false;
    }
    let wide = HSTRING::from(alias);
    // SND_ALIAS: `alias` is a sound-scheme event name, not a path.
    // SND_ASYNC: return immediately, do not hold up the toggle.
    // SND_NODEFAULT: if the name has no sound, stay quiet, do not beep.
    unsafe {
        PlaySoundW(
            PCWSTR(wide.as_ptr()),
            None,
            SND_ALIAS | SND_ASYNC | SND_NODEFAULT,
        )
    }
    .as_bool()
}

/// Read what the sound scheme holds for one event name.
///
/// This is the same place Windows itself resolves the name from. Read only:
/// nothing in this file may ever change a person's sound settings.
fn scheme_entry(alias: &str) -> SchemeEntry {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};

    let subkey = HSTRING::from(format!(r"AppEvents\Schemes\Apps\.Default\{alias}\.Current"));
    // Room for any real sound path. A longer one comes back as an error, which
    // reads as unknown, so the name is still tried.
    let mut data = [0u16; 512];
    let mut size = std::mem::size_of_val(&data) as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            // The unnamed default value of the `.Current` key.
            PCWSTR::null(),
            RRF_RT_REG_SZ,
            None,
            Some(data.as_mut_ptr() as *mut std::ffi::c_void),
            Some(&mut size),
        )
    };
    if result != ERROR_SUCCESS {
        return classify(None);
    }
    let text_len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    classify(Some(&String::from_utf16_lossy(&data[..text_len])))
}

/// Turn what was read into what it means, kept apart from the reading so it can
/// be tested. `None` is a read that failed.
fn classify(value: Option<&str>) -> SchemeEntry {
    match value {
        None => SchemeEntry::Unknown,
        Some("") => SchemeEntry::Silent,
        Some(_) => SchemeEntry::NamesASound,
    }
}

fn report_once(which: &str, outcome: &str) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static OPEN_DONE: AtomicBool = AtomicBool::new(false);
    static CLOSE_DONE: AtomicBool = AtomicBool::new(false);
    let done = if which == "open" {
        &OPEN_DONE
    } else {
        &CLOSE_DONE
    };
    if done
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        eprintln!("dictate: {which} sound -> {outcome}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run `choose` against a made-up machine, and record which names it tried.
    fn choose_on(silent: &[&str]) -> (&'static str, Vec<String>) {
        let mut tried = Vec::new();
        let outcome = choose("DeviceDisconnect", "SystemExclamation", &mut |alias| {
            tried.push(alias.to_string());
            !silent.contains(&alias)
        });
        (outcome, tried)
    }

    #[test]
    fn a_working_device_sound_is_used_and_the_fallback_is_left_alone() {
        // covers: AC-2. The device event names are the apt pair, so nothing
        // else is tried when they work.
        let (outcome, tried) = choose_on(&[]);
        assert_eq!(outcome, "device event name");
        assert_eq!(tried, ["DeviceDisconnect"]);
    }

    #[test]
    fn a_device_sound_set_to_none_falls_through_to_the_alert_pair() {
        // covers: AC-2's fallback, and the bug found on 2026-08-30. This is the
        // machine the fallback exists for: the device event name is present but
        // set to nothing, so it makes no sound and the alert pair must be
        // reached. Windows reports success for such a name, which is why the
        // scheme entry is checked and not just the return value.
        let (outcome, tried) = choose_on(&["DeviceDisconnect"]);
        assert_eq!(outcome, "alert fallback");
        assert_eq!(tried, ["DeviceDisconnect", "SystemExclamation"]);
    }

    #[test]
    fn both_names_silent_is_reported_honestly() {
        // covers: AC-2. If neither pair can make a sound, the spike line must
        // say so rather than claim one played.
        let (outcome, tried) = choose_on(&["DeviceDisconnect", "SystemExclamation"]);
        assert_eq!(outcome, "nothing played");
        assert_eq!(tried, ["DeviceDisconnect", "SystemExclamation"]);
    }

    #[test]
    fn an_entry_set_to_nothing_is_the_one_that_must_be_skipped() {
        // covers: AC-2's fallback. `(None)` on the Windows sound page leaves
        // the entry present and empty, which is what used to slip through.
        assert_eq!(classify(Some("")), SchemeEntry::Silent);
    }

    #[test]
    fn an_entry_naming_a_sound_is_worth_playing() {
        // covers: AC-2.
        assert_eq!(
            classify(Some(r"C:\WINDOWS\media\Windows Hardware Remove.wav")),
            SchemeEntry::NamesASound
        );
    }

    #[test]
    fn an_unreadable_entry_is_not_evidence_of_silence() {
        // covers: AC-2. A scheme we cannot read must not silence a sound that
        // would have played. The name is tried and Windows gets the last word,
        // matching how `consent.rs` treats an unreadable switch.
        assert_eq!(classify(None), SchemeEntry::Unknown);
        assert_ne!(classify(None), SchemeEntry::Silent);
    }

    /// This module's own source, minus its tests.
    fn this_file() -> &'static str {
        include_str!("sound.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn nothing_here_can_write_to_the_sound_scheme() {
        // covers: the data rules. This file reads the sound scheme to decide
        // what to play, and may never gain a way to change a person's sound
        // settings.
        let source = this_file();
        for forbidden in [
            "RegSetValue",
            "RegCreateKey",
            "RegDeleteKey",
            "RegDeleteValue",
        ] {
            assert!(
                !source.contains(forbidden),
                "sound.rs now mentions `{forbidden}`. It reads the sound scheme \
                 and may never write to it"
            );
        }
    }
}
