//! When a dictation closes on its own (record 0002 AC-8).
//!
//! Two caps, both fixed by the record and neither a setting:
//!
//!   * **5 minutes total.** Runs from the moment the microphone opened.
//!   * **30 seconds with no speech.** Record 0002's Value sourcing is exact
//!     about what "no speech" means: *"Silence means no final wording from
//!     Deepgram in that period."* Deepgram arrives in milestone 4.
//!
//! So in milestone 2 the silence cap is built, tested and **unarmed**. There is
//! nothing on this machine yet that can honestly say a person spoke: loudness
//! says sound is arriving, which a fan or a quiet room full of typing also does.
//! Arming it on loudness would be a different promise from the one AC-8 makes,
//! so it waits for its named source instead of borrowing a nearby one.
//!
//! Milestone 4 changes exactly two things: it builds `Deadlines` with
//! `SilenceWatch::Watching` instead of `Unarmed`, and it calls `speech_heard`
//! on every final Deepgram result. The rules below do not change, and the tests
//! at the bottom already cover the armed behaviour.

use std::time::{Duration, Instant};

/// The hard cap on one dictation. Record 0002 AC-8.
pub const TIME_CAP: Duration = Duration::from_secs(5 * 60);

/// How long without speech closes the microphone. Record 0002 AC-8.
pub const SILENCE_CAP: Duration = Duration::from_secs(30);

/// Why a dictation closed on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiry {
    /// 5 minutes since the microphone opened.
    TimeCap,
    /// 30 seconds without speech.
    Silence,
}

impl Expiry {
    /// The word that goes out on `dictation:closed`. Record 0002's interface
    /// surface names the four reasons: you stopped it, silence, the time cap,
    /// or an error.
    pub fn reason(self) -> &'static str {
        match self {
            Expiry::TimeCap => "the_time_cap",
            Expiry::Silence => "silence",
        }
    }
}

/// Whether the silence cap is running, and since when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SilenceWatch {
    /// Nothing can report speech yet, so the silence cap must not fire. This is
    /// the milestone 2 value and the only one this milestone constructs.
    Unarmed,
    /// Counting from the last thing that counted as speech.
    Watching { since: Instant },
}

/// The two caps for one open microphone, answered against a clock the caller
/// passes in. Pure: it reads no clock of its own, which is what makes the tests
/// below able to fast-forward five minutes without waiting five minutes.
#[derive(Debug, Clone, Copy)]
pub struct Deadlines {
    opened: Instant,
    silence: SilenceWatch,
}

impl Deadlines {
    /// The caps as milestone 2 runs them: the time cap live, the silence cap
    /// unarmed because Deepgram does not exist yet.
    pub fn without_deepgram(opened: Instant) -> Self {
        Self {
            opened,
            silence: SilenceWatch::Unarmed,
        }
    }

    /// The caps as milestone 4 will run them, with the silence clock started at
    /// the moment the microphone opened.
    #[allow(dead_code)] // Milestone 4 calls this. Exercised by the tests below.
    pub fn watching_for_silence(opened: Instant) -> Self {
        Self {
            opened,
            silence: SilenceWatch::Watching { since: opened },
        }
    }

    /// A phrase came back from Deepgram marked final. Restarts the silence
    /// clock. Does nothing while the watch is unarmed.
    #[allow(dead_code)] // Milestone 4 calls this. Exercised by the tests below.
    pub fn speech_heard(&mut self, now: Instant) {
        if let SilenceWatch::Watching { .. } = self.silence {
            self.silence = SilenceWatch::Watching { since: now };
        }
    }

    /// Whether the microphone should close now, and why. The time cap is
    /// checked first: if both have run out, the one that closes the whole
    /// dictation is the truer answer.
    pub fn expired(&self, now: Instant) -> Option<Expiry> {
        if now.duration_since(self.opened) >= TIME_CAP {
            return Some(Expiry::TimeCap);
        }
        if let SilenceWatch::Watching { since } = self.silence {
            if now.duration_since(since) >= SILENCE_CAP {
                return Some(Expiry::Silence);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, secs: u64) -> Instant {
        base + Duration::from_secs(secs)
    }

    #[test]
    fn the_time_cap_fires_at_five_minutes_and_not_before() {
        // covers: AC-8, the 5 minute half.
        let opened = Instant::now();
        let caps = Deadlines::without_deepgram(opened);

        assert_eq!(caps.expired(at(opened, 299)), None, "4:59 is still going");
        assert_eq!(
            caps.expired(at(opened, 300)),
            Some(Expiry::TimeCap),
            "5:00 closes it"
        );
    }

    #[test]
    fn the_silence_cap_never_fires_while_it_is_unarmed() {
        // covers: AC-8, the deferred 30 second half. This is the property that
        // keeps milestone 2 honest: with nothing able to report speech, the
        // silence cap must stay quiet rather than close the pill after 30
        // seconds of somebody talking.
        let opened = Instant::now();
        let caps = Deadlines::without_deepgram(opened);

        assert_eq!(caps.expired(at(opened, 30)), None);
        assert_eq!(caps.expired(at(opened, 120)), None);
        // ...right up to the moment the other cap takes over.
        assert_eq!(caps.expired(at(opened, 299)), None);
    }

    #[test]
    fn the_silence_cap_fires_at_thirty_seconds_once_armed() {
        // covers: AC-8, the 30 second half, as milestone 4 will run it.
        let opened = Instant::now();
        let caps = Deadlines::watching_for_silence(opened);

        assert_eq!(caps.expired(at(opened, 29)), None);
        assert_eq!(caps.expired(at(opened, 30)), Some(Expiry::Silence));
    }

    #[test]
    fn speech_restarts_the_silence_clock() {
        // covers: AC-8. Someone who keeps talking is never cut off.
        let opened = Instant::now();
        let mut caps = Deadlines::watching_for_silence(opened);

        caps.speech_heard(at(opened, 25));
        assert_eq!(caps.expired(at(opened, 50)), None, "25s since last speech");
        assert_eq!(
            caps.expired(at(opened, 55)),
            Some(Expiry::Silence),
            "30s since last speech"
        );
    }

    #[test]
    fn speech_cannot_hold_the_microphone_open_past_the_time_cap() {
        // covers: AC-8. The 5 minute cap is a hard ceiling, so a person who
        // talks without pause still gets closed.
        let opened = Instant::now();
        let mut caps = Deadlines::watching_for_silence(opened);

        caps.speech_heard(at(opened, 299));
        assert_eq!(caps.expired(at(opened, 300)), Some(Expiry::TimeCap));
    }

    #[test]
    fn speech_heard_is_ignored_while_the_watch_is_unarmed() {
        // Milestone 4 wiring this up early must not accidentally arm the cap.
        let opened = Instant::now();
        let mut caps = Deadlines::without_deepgram(opened);

        caps.speech_heard(at(opened, 10));
        assert_eq!(caps.expired(at(opened, 299)), None);
    }

    #[test]
    fn the_reasons_are_the_words_the_interface_is_told() {
        assert_eq!(Expiry::TimeCap.reason(), "the_time_cap");
        assert_eq!(Expiry::Silence.reason(), "silence");
    }
}
