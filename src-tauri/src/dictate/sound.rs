//! The two sounds the pill makes: one when the microphone opens, one when it
//! closes.
//!
//! Record 0002 AC-2: they come from the machine's own sound scheme, played by
//! name. Nothing ships with the app. The names tried first are the ones Windows
//! uses for a device being plugged in and unplugged. If the scheme has no sound
//! under those names, `PlaySoundW` reports failure and we fall back to the alert
//! pair the record names (`SystemAsterisk` on open, `SystemExclamation` on
//! close). Both are less apt, which is why they are only a fallback.
//!
//! Milestone 1 owes the answer to "which pair actually plays" back to the
//! record's Still open section. `chosen_open_alias` / `chosen_close_alias`
//! print one line to stderr saying which name won, so that spike can be read
//! off a normal run. No audio is ever recorded, written or logged; this only
//! ever names a system sound.

use windows::core::{w, PCWSTR};
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_NODEFAULT};

/// Play the "microphone opened" sound, if `enabled`.
pub fn play_open(enabled: bool) {
    if enabled {
        play_first_that_works(w!("DeviceConnect"), w!("SystemAsterisk"), "open");
    }
}

/// Play the "microphone closed" sound, if `enabled`.
pub fn play_close(enabled: bool) {
    if enabled {
        play_first_that_works(w!("DeviceDisconnect"), w!("SystemExclamation"), "close");
    }
}

/// Try `preferred`; if the scheme has nothing under that name, try `fallback`.
/// Prints which name was used the first time each is asked for, for the record's
/// open spike.
fn play_first_that_works(preferred: PCWSTR, fallback: PCWSTR, which: &str) {
    if play_alias(preferred) {
        report_once(which, "device event name");
    } else if play_alias(fallback) {
        report_once(which, "alert fallback");
    } else {
        report_once(which, "nothing played");
    }
}

/// Ask Windows to play a sound-scheme entry by name, without blocking and
/// without the default beep if the name resolves to no sound. Returns whether
/// Windows accepted it.
fn play_alias(alias: PCWSTR) -> bool {
    // SND_ALIAS: `alias` is a sound-scheme event name, not a path.
    // SND_ASYNC: return immediately, do not hold up the toggle.
    // SND_NODEFAULT: if the name has no sound, play silence, not the system beep.
    unsafe { PlaySoundW(alias, None, SND_ALIAS | SND_ASYNC | SND_NODEFAULT).as_bool() }
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
