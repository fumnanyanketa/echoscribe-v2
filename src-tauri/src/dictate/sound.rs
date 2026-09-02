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
//! **An entry naming a sound file that has since been deleted also plays
//! silence**, the same hole one machine along, and record 0002 settled it on
//! 2026-08-30 (Still open, the sound bullet, point 1): the path is read from
//! the scheme, environment variables in it are expanded, a bare file name is
//! resolved against the Windows media folder, and the file has to be there. A
//! path that cannot be resolved, or cannot be checked, is played anyway and
//! Windows gets the last word: not knowing is not evidence, the same rule
//! `consent.rs` holds for a switch it cannot read. That is wrong-but-safe on a
//! path form this file fails to resolve, which the record accepts by name.
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
    /// The entry names a sound whose file is there, or one this code cannot
    /// check. Either way the name is worth playing.
    NamesASound,
    /// Playing this entry would make no sound: it is set to nothing, which is
    /// `(None)` on the Windows sound page, or it names a file that is known to
    /// be gone. Both play silence while reporting success, so both are ruled
    /// out before Windows is asked.
    Silent,
    /// The entry could not be read. Not evidence either way, so the name is
    /// still worth trying, the same way `consent.rs` treats a switch it cannot
    /// read as not-deny.
    Unknown,
}

/// Whether the file a scheme entry names is actually on the machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NamedFile {
    There,
    /// Resolved cleanly, and nothing is at the other end. The one answer that
    /// counts as evidence of silence.
    Gone,
    /// The path could not be resolved, or the disk could not answer. Not
    /// evidence, so the name is played anyway and Windows gets the last word.
    CannotTell,
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
        return classify(None, &named_file_on_this_machine);
    }
    let text_len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    classify(
        Some(&String::from_utf16_lossy(&data[..text_len])),
        &named_file_on_this_machine,
    )
}

/// Turn what was read into what it means, kept apart from the reading and the
/// disk so it can be tested. `None` is a read that failed. `named_file` answers
/// whether the file an entry names is actually there.
fn classify(value: Option<&str>, named_file: &dyn Fn(&str) -> NamedFile) -> SchemeEntry {
    match value {
        None => SchemeEntry::Unknown,
        Some("") => SchemeEntry::Silent,
        Some(path) => match named_file(path) {
            // A named file somebody deleted plays silence and reports success,
            // exactly like a `(None)` entry, so it is the same case.
            NamedFile::Gone => SchemeEntry::Silent,
            NamedFile::There | NamedFile::CannotTell => SchemeEntry::NamesASound,
        },
    }
}

/// Whether the file a scheme entry names is on this machine, per record 0002's
/// resolution rules: environment variables expanded, a bare file name resolved
/// against the Windows media folder, and anything unresolvable never read as
/// silence.
fn named_file_on_this_machine(raw: &str) -> NamedFile {
    let Some(path) = resolved(raw) else {
        return NamedFile::CannotTell;
    };
    match std::fs::metadata(&path) {
        Ok(_) => NamedFile::There,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => NamedFile::Gone,
        // The disk would not answer (permissions, a detached drive). Not
        // evidence of a missing file.
        Err(_) => NamedFile::CannotTell,
    }
}

/// The absolute path a scheme entry means, or `None` when this code cannot say.
///
/// `RegGetValueW` already expands `REG_EXPAND_SZ` entries, so most values
/// arrive expanded; this covers a plain `REG_SZ` that still carries `%VAR%`.
/// Windows resolves a bare file name against its media folder, so this does
/// the same. A relative path with separators in it is Windows' resolution
/// order, not ours, so it reads as cannot-tell rather than guessed at.
fn resolved(raw: &str) -> Option<std::path::PathBuf> {
    let expanded = expand_env(raw)?;
    let path = std::path::PathBuf::from(&expanded);
    if path.is_absolute() {
        return Some(path);
    }
    if expanded.contains('\\') || expanded.contains('/') {
        return None;
    }
    let windir = std::env::var("SystemRoot").ok()?;
    Some(
        std::path::PathBuf::from(windir)
            .join("Media")
            .join(expanded),
    )
}

/// `%NAME%` pieces replaced with their values. `None` when a variable is not
/// set or a `%` never closes, because a half-expanded path would name a file
/// that never existed and read as deleted.
fn expand_env(text: &str) -> Option<String> {
    if !text.contains('%') {
        return Some(text.to_string());
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('%')?;
        out.push_str(&std::env::var(&after[..end]).ok()?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
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

    /// A pretend disk for `classify`, so no test depends on this machine's
    /// real files.
    fn disk(answer: NamedFile) -> impl Fn(&str) -> NamedFile {
        move |_| answer
    }

    #[test]
    fn an_entry_set_to_nothing_is_the_one_that_must_be_skipped() {
        // covers: AC-2's fallback. `(None)` on the Windows sound page leaves
        // the entry present and empty, which is what used to slip through. The
        // disk is never even asked for an empty entry.
        let never = |_: &str| panic!("an empty entry needs no file check");
        assert_eq!(classify(Some(""), &never), SchemeEntry::Silent);
    }

    #[test]
    fn an_entry_naming_a_file_that_is_there_is_worth_playing() {
        // covers: AC-2.
        assert_eq!(
            classify(
                Some(r"C:\WINDOWS\media\Windows Hardware Remove.wav"),
                &disk(NamedFile::There)
            ),
            SchemeEntry::NamesASound
        );
    }

    #[test]
    fn an_entry_naming_a_deleted_file_is_skipped_like_a_none_entry() {
        // covers: AC-2's fallback, record 0002's sound bullet point 1. An entry
        // whose file somebody deleted plays silence and reports success, the
        // same hole the `(None)` fix closed, one machine along. It must fall
        // through to the alert pair rather than win with silence.
        assert_eq!(
            classify(Some(r"C:\WINDOWS\media\gone.wav"), &disk(NamedFile::Gone)),
            SchemeEntry::Silent
        );
    }

    #[test]
    fn a_path_that_cannot_be_checked_is_played_anyway() {
        // covers: AC-2, record 0002's sound bullet point 1. Not knowing is not
        // evidence. A path form we fail to resolve degrades to today's
        // behaviour, wrong-but-safe, and Windows gets the last word.
        assert_eq!(
            classify(
                Some(r"%NOT_A_REAL_VAR%\chime.wav"),
                &disk(NamedFile::CannotTell)
            ),
            SchemeEntry::NamesASound
        );
    }

    #[test]
    fn an_unreadable_entry_is_not_evidence_of_silence() {
        // covers: AC-2. A scheme we cannot read must not silence a sound that
        // would have played. The name is tried and Windows gets the last word,
        // matching how `consent.rs` treats an unreadable switch.
        let never = |_: &str| panic!("an unreadable entry needs no file check");
        assert_eq!(classify(None, &never), SchemeEntry::Unknown);
    }

    // ---- Resolving the path a scheme entry names ----

    #[test]
    fn an_absolute_path_resolves_to_itself() {
        assert_eq!(
            resolved(r"C:\WINDOWS\media\chimes.wav"),
            Some(std::path::PathBuf::from(r"C:\WINDOWS\media\chimes.wav"))
        );
    }

    #[test]
    fn environment_variables_are_expanded() {
        // covers: record 0002's sound bullet point 1. Scheme entries routinely
        // read %SystemRoot%\media\... . A made-up variable keeps the test off
        // this machine's real environment.
        std::env::set_var("ECHOSCRIBE_SOUND_TEST_ROOT", r"C:\made-up");
        assert_eq!(
            resolved(r"%ECHOSCRIBE_SOUND_TEST_ROOT%\media\chimes.wav"),
            Some(std::path::PathBuf::from(r"C:\made-up\media\chimes.wav"))
        );
    }

    #[test]
    fn a_variable_that_is_not_set_reads_as_cannot_resolve() {
        // covers: record 0002's sound bullet point 1. A half-expanded path
        // would name a file that never existed and read as deleted, silencing
        // a sound that plays fine.
        assert_eq!(resolved(r"%ECHOSCRIBE_NO_SUCH_VAR%\chimes.wav"), None);
        assert_eq!(expand_env("50% there"), None);
    }

    #[test]
    fn a_bare_file_name_is_resolved_against_the_windows_media_folder() {
        // covers: record 0002's sound bullet point 1. Windows resolves a bare
        // name against its media folder, so the check must look there too.
        std::env::set_var("SystemRoot", r"C:\WINDOWS");
        assert_eq!(
            resolved("chord.wav"),
            Some(std::path::PathBuf::from(r"C:\WINDOWS\Media\chord.wav"))
        );
    }

    #[test]
    fn a_relative_path_with_separators_is_not_guessed_at() {
        // covers: record 0002's sound bullet point 1. Windows' resolution order
        // for these is not ours to reimplement, so it reads as cannot-tell and
        // the name is played anyway.
        assert_eq!(resolved(r"media\chimes.wav"), None);
    }

    #[test]
    fn text_without_variables_passes_through_expansion_unchanged() {
        assert_eq!(
            expand_env(r"C:\WINDOWS\media\chimes.wav"),
            Some(r"C:\WINDOWS\media\chimes.wav".to_string())
        );
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
