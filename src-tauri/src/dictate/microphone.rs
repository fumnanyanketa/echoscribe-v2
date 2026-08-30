//! Capturing the microphone, and turning it into one loudness number.
//!
//! **No audio is kept.** Record 0002 refuses writing audio to disk or into a
//! log, ever, and AGENTS.md's data rules say recorded audio is transient. The
//! shape of this file is what makes that structurally true rather than merely
//! intended: the only thing that leaves the audio callback is a running total
//! of squared sample values and a count of samples. There is no buffer, no
//! ring, no file handle, and no way to reconstruct a sound from what is kept.
//! The samples themselves are borrowed from cpal for the length of one callback
//! and are gone when it returns. Two tests at the bottom guard this file
//! against ever gaining a way to write audio out or hold on to it.
//!
//! **cpal owns the stream, so the stream never leaves the thread that made it.**
//! Milestone 1's hard lesson was that a library which owns a resource will
//! quietly undo anything done to that resource behind its back. The audio
//! equivalent is that a capture stream belongs to the thread that created it:
//! it is created, started and dropped by one thread here, which then does
//! nothing but wait to be told to stop. Dropping that stream is what actually
//! closes the microphone, and it is the only thing that does.
//!
//! Two threads, on purpose:
//!
//!   * **audio** owns the cpal stream for its whole life and is otherwise idle.
//!   * **level** wakes every 60 ms, takes whatever the callback has accumulated
//!     since it last looked, turns it into one number and hands it to the
//!     caller. 60 ms is the design comp's own figure for the level meter.
//!
//! Splitting them means the number arriving at the pill keeps a steady cadence
//! of our choosing rather than whatever buffer size the sound card happens to
//! hand us, and it means nothing the caller does in its callback can ever run
//! on the audio thread and glitch the capture.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ErrorKind, FromSample, Sample, SampleFormat, SizedSample};

/// How often a loudness number goes to the pill. The design comp specifies the
/// level meter as "18 bars, 60 ms updates".
pub const LEVEL_EVERY: Duration = Duration::from_millis(60);

/// Quieter than this reads as nothing at all: the meter sits flat.
const FLOOR_DBFS: f64 = -55.0;
/// Louder than this fills the meter. Comfortable speech at a normal desk
/// distance lands between the two, so the meter spends its life in range
/// instead of pinned at one end.
const CEIL_DBFS: f64 = -12.0;

/// Why the microphone would not open. Record 0002 AC-15 requires each of these
/// to read differently, and its interface surface names the first three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicError {
    /// Windows has microphone access switched off for this app or this machine.
    BlockedByWindows,
    /// Another app is holding the device.
    InUseByAnotherApp,
    /// There is no input device at all.
    NoMicrophoneFound,
    /// It failed for some other reason. Not one of the record's three named
    /// causes, and deliberately not dressed up as one: telling somebody their
    /// microphone is in use by another app when it is not sends them looking in
    /// the wrong place.
    Unavailable,
}

impl MicError {
    /// The machine-readable kind that goes out on `dictation:error`.
    pub fn kind(self) -> &'static str {
        match self {
            MicError::BlockedByWindows => "microphone_blocked_by_windows",
            MicError::InUseByAnotherApp => "microphone_in_use_by_another_app",
            MicError::NoMicrophoneFound => "no_microphone_found",
            MicError::Unavailable => "microphone_unavailable",
        }
    }

    /// One plain sentence naming the cause. Safe to show: it never carries a
    /// device name, a path, or anything from the audio itself.
    pub fn message(self) -> &'static str {
        match self {
            MicError::BlockedByWindows => "Windows is not letting EchoScribe use the microphone.",
            MicError::InUseByAnotherApp => "Another app is using the microphone right now.",
            // "No microphone is plugged in" was the first wording and is
            // replaced by the record: a laptop's microphone is built in and was
            // never plugged in, so that sentence sends a laptop user looking
            // for a cable that does not exist.
            MicError::NoMicrophoneFound => "Windows cannot find a microphone.",
            MicError::Unavailable => "The microphone could not be opened.",
        }
    }
}

/// What the audio callback adds up between one 60 ms tick and the next. Two
/// numbers, and nothing that resembles a recording.
#[derive(Debug, Default, Clone, Copy)]
struct Loudness {
    sum_of_squares: f64,
    samples: u64,
}

/// An open microphone. Dropping it closes the device too, because both threads
/// end as soon as their channel disconnects; `stop` is the version that waits,
/// and so can promise the device is really shut.
pub struct Microphone {
    stop_audio: Sender<()>,
    stop_level: Sender<()>,
    audio: JoinHandle<()>,
    level: JoinHandle<()>,
}

impl Microphone {
    /// Close the microphone and wait until it really is closed.
    ///
    /// The level thread is stopped first, so no loudness number can arrive at a
    /// pill that is on its way off screen. Then the audio thread is told to
    /// drop the stream, and joining it is what makes "the microphone is shut" a
    /// fact rather than a request.
    pub fn stop(self) {
        let Self {
            stop_audio,
            stop_level,
            audio,
            level,
        } = self;

        let _ = stop_level.send(());
        let _ = level.join();
        let _ = stop_audio.send(());
        let _ = audio.join();
    }
}

/// Open the default input device and start reporting loudness.
///
/// `on_level` is called on the level thread roughly every 60 ms with a number
/// from 0.0 (silence) to 1.0 (loud). It is the only thing that ever crosses out
/// of this module, and it carries no audio.
///
/// Returns before the first tick, and only once the device is genuinely
/// capturing: a failure comes back here rather than turning up later, which is
/// what lets the caller keep AC-15's promise that the pill never appears when
/// the microphone will not open.
pub fn open(on_level: Box<dyn Fn(f32) + Send>) -> Result<Microphone, MicError> {
    let shared = Arc::new(Mutex::new(Loudness::default()));

    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), MicError>>();
    let (stop_audio, stop_audio_rx) = mpsc::channel::<()>();
    let audio_shared = Arc::clone(&shared);

    let audio = spawn("echoscribe-dictate-audio", move || {
        // Everything to do with the stream happens here, on this one thread,
        // for the whole life of the stream. cpal owns it; we do not move it.
        let stream = match build_stream(audio_shared) {
            Ok(stream) => stream,
            Err(e) => {
                let _ = ready_tx.send(Err(e));
                return;
            }
        };
        if let Err(e) = stream.play() {
            let _ = ready_tx.send(Err(classify(e.kind())));
            return;
        }
        let _ = ready_tx.send(Ok(()));

        // Park until told to stop, or until the sender goes away.
        let _ = stop_audio_rx.recv();
        // Explicit, because this line is the one that closes the microphone.
        drop(stream);
    })?;

    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let _ = audio.join();
            return Err(e);
        }
        // The thread ended without answering.
        Err(_) => {
            let _ = audio.join();
            return Err(MicError::Unavailable);
        }
    }

    let (stop_level, stop_level_rx) = mpsc::channel::<()>();
    let level = spawn("echoscribe-dictate-level", move || {
        // Wake every 60 ms and report. Anything other than a timeout means the
        // microphone is closing: either `stop` was called, or the `Microphone`
        // was dropped and took the sender with it.
        while let Err(RecvTimeoutError::Timeout) = stop_level_rx.recv_timeout(LEVEL_EVERY) {
            on_level(take_level(&shared));
        }
    });

    match level {
        Ok(level) => Ok(Microphone {
            stop_audio,
            stop_level,
            audio,
            level,
        }),
        Err(e) => {
            // The device opened but we cannot report its level. Close it again
            // rather than leave a microphone open with a dead meter on screen.
            let _ = stop_audio.send(());
            let _ = audio.join();
            Err(e)
        }
    }
}

fn spawn(name: &str, body: impl FnOnce() + Send + 'static) -> Result<JoinHandle<()>, MicError> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(body)
        .map_err(|e| {
            eprintln!("dictate: could not start the {name} thread: {e}");
            MicError::Unavailable
        })
}

/// Build a capture stream on the default input device, at whatever format that
/// device prefers. Runs on the audio thread and nowhere else.
///
/// Record 0002 names no microphone anywhere: it has no device column, no device
/// setting and no device picker, so the one Windows is already set to use is
/// the only microphone this feature has. Choosing between several is a settings
/// screen the record does not describe.
fn build_stream(shared: Arc<Mutex<Loudness>>) -> Result<cpal::Stream, MicError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(MicError::NoMicrophoneFound)?;
    let supported = device
        .default_input_config()
        .map_err(|e| classify(e.kind()))?;
    let format = supported.sample_format();
    let config = supported.config();

    let built = match format {
        SampleFormat::F32 => capture::<f32>(&device, config, shared),
        SampleFormat::F64 => capture::<f64>(&device, config, shared),
        SampleFormat::I8 => capture::<i8>(&device, config, shared),
        SampleFormat::I16 => capture::<i16>(&device, config, shared),
        SampleFormat::I32 => capture::<i32>(&device, config, shared),
        SampleFormat::U8 => capture::<u8>(&device, config, shared),
        SampleFormat::U16 => capture::<u16>(&device, config, shared),
        SampleFormat::U32 => capture::<u32>(&device, config, shared),
        _ => return Err(MicError::Unavailable),
    };
    built.map_err(|e| classify(e.kind()))
}

/// The audio callback, for one sample type.
///
/// This is the only code that ever sees the person's voice. It reads each
/// sample once, squares it into a running total, and keeps nothing. When it
/// returns, `data` is gone.
fn capture<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    shared: Arc<Mutex<Loudness>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut sum = 0.0f64;
            for sample in data {
                let value = f32::from_sample(*sample) as f64;
                sum += value * value;
            }
            // `try_lock`, never `lock`: an audio callback that waits on another
            // thread is how capture starts glitching. Losing one 60 ms window's
            // worth of level is not worth a stall.
            if let Ok(mut acc) = shared.try_lock() {
                acc.sum_of_squares += sum;
                acc.samples += data.len() as u64;
            }
        },
        |e| eprintln!("dictate: the microphone stream reported an error: {e}"),
        None,
    )
}

/// Take everything accumulated since the last tick and turn it into one number.
/// Resets the accumulator, so each tick reports its own 60 ms rather than the
/// whole dictation so far.
fn take_level(shared: &Mutex<Loudness>) -> f32 {
    let taken = match shared.lock() {
        Ok(mut acc) => std::mem::take(&mut *acc),
        Err(_) => return 0.0,
    };
    if taken.samples == 0 {
        return 0.0;
    }
    level_from_rms((taken.sum_of_squares / taken.samples as f64).sqrt())
}

/// Root mean square amplitude turned into a meter reading, 0.0 to 1.0.
///
/// The mapping is in decibels rather than straight amplitude because hearing is
/// logarithmic: on a linear scale ordinary speech sits in the bottom tenth of
/// the meter and the thing barely moves. Amplitude only, and never dressed up
/// as a quality reading (design/design-system.md).
fn level_from_rms(rms: f64) -> f32 {
    if !rms.is_finite() || rms <= 0.0 {
        return 0.0;
    }
    let dbfs = 20.0 * rms.log10();
    (((dbfs - FLOOR_DBFS) / (CEIL_DBFS - FLOOR_DBFS)).clamp(0.0, 1.0)) as f32
}

/// Map cpal's failure onto record 0002's named causes (AC-15).
///
/// Only two kinds are named here, because they are the only two cpal reports
/// reliably on Windows. Blocked-by-Windows never arrives from cpal at all,
/// proven live on 2026-08-30: a privacy block lands in the catch-all, and
/// `consent::refine` is what tells it apart, by reading the consent switches
/// the Windows privacy page writes (record 0002, step 2b). So even a
/// permission-denied kind, should some platform ever produce one, stays the
/// catch-all here and lets the consent check decide.
fn classify(kind: ErrorKind) -> MicError {
    match kind {
        ErrorKind::DeviceBusy => MicError::InUseByAnotherApp,
        ErrorKind::DeviceNotAvailable | ErrorKind::HostUnavailable => MicError::NoMicrophoneFound,
        _ => MicError::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_reads_as_a_flat_meter() {
        // covers: AC-1. "Flat at silence" (design/design-system.md).
        assert_eq!(level_from_rms(0.0), 0.0);
        assert_eq!(level_from_rms(-0.0), 0.0);
        // Well below the floor: a very quiet room, not speech.
        assert_eq!(level_from_rms(0.0001), 0.0, "-80 dBFS is under the floor");
    }

    #[test]
    fn a_loud_sound_fills_the_meter_and_never_overflows_it() {
        // covers: AC-1. The meter has 18 bars and no nineteenth.
        assert_eq!(level_from_rms(1.0), 1.0, "full scale");
        assert_eq!(level_from_rms(50.0), 1.0, "past full scale is still 1.0");
    }

    #[test]
    fn ordinary_speech_lands_in_the_middle_of_the_meter() {
        // The whole point of the decibel mapping: a normal speaking voice must
        // move the meter visibly, not nudge the bottom bar.
        let quiet_speech = level_from_rms(0.01); // -40 dBFS
        let normal_speech = level_from_rms(0.05); // -26 dBFS
        assert!(
            (0.2..0.5).contains(&quiet_speech),
            "quiet speech was {quiet_speech}"
        );
        assert!(
            (0.5..0.9).contains(&normal_speech),
            "normal speech was {normal_speech}"
        );
        assert!(normal_speech > quiet_speech);
    }

    #[test]
    fn a_nonsense_reading_is_treated_as_silence() {
        assert_eq!(level_from_rms(f64::NAN), 0.0);
        assert_eq!(level_from_rms(f64::NEG_INFINITY), 0.0);
        assert_eq!(level_from_rms(f64::INFINITY), 0.0);
    }

    #[test]
    fn every_microphone_failure_reads_differently() {
        // covers: AC-15. Each cause has to send the person somewhere different,
        // so no two may share a kind or a sentence.
        let all = [
            MicError::BlockedByWindows,
            MicError::InUseByAnotherApp,
            MicError::NoMicrophoneFound,
            MicError::Unavailable,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.kind(), b.kind(), "{a:?} and {b:?} share a kind");
                assert_ne!(a.message(), b.message(), "{a:?} and {b:?} share a message");
            }
        }
    }

    #[test]
    fn the_two_causes_cpal_names_reliably_are_kept_apart() {
        // covers: AC-15. These send a person to completely different places:
        // one to another app, one to a cable or a setting.
        assert_eq!(classify(ErrorKind::DeviceBusy), MicError::InUseByAnotherApp);
        assert_eq!(
            classify(ErrorKind::DeviceNotAvailable),
            MicError::NoMicrophoneFound
        );
    }

    #[test]
    fn a_permission_error_is_left_for_the_consent_check() {
        // covers: AC-15, AC-29. cpal on Windows never produces this kind,
        // proven live 2026-08-30, so mapping it to blocked was dead code that
        // made the feature look covered. Blocked is decided by consent.rs
        // reading the Windows switches, and only from the catch-all.
        assert_eq!(classify(ErrorKind::PermissionDenied), MicError::Unavailable);
    }

    #[test]
    fn an_unrecognised_failure_is_not_dressed_up_as_a_named_one() {
        // covers: AC-15. Guessing "another app is using it" when we do not know
        // sends somebody hunting for an app that is not there.
        assert_eq!(
            classify(ErrorKind::UnsupportedConfig),
            MicError::Unavailable
        );
        assert_eq!(classify(ErrorKind::BackendError), MicError::Unavailable);
    }

    #[test]
    fn the_accumulator_is_emptied_by_each_tick() {
        let shared = Mutex::new(Loudness {
            sum_of_squares: 0.25,
            samples: 1,
        });
        assert!(
            take_level(&shared) > 0.0,
            "0.5 rms is well inside the meter"
        );
        assert_eq!(
            take_level(&shared),
            0.0,
            "a tick with no audio since the last one reads as silence, not as a \
             repeat of the last reading"
        );
    }

    /// This module's own source, minus its tests, so a guard cannot be satisfied
    /// by the test that checks it.
    fn this_file() -> &'static str {
        include_str!("microphone.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn nothing_here_can_write_audio_anywhere() {
        // Record 0002's Risk section refuses writing audio to disk or into a
        // log, ever, and says so explicitly survives any future request for a
        // debug mode that saves recordings. This is a source guard, not a
        // behaviour test: it cannot prove no audio is written, only that the
        // tools for writing it are not in this file. The real proof is the
        // shape above, where the callback keeps two numbers and no samples.
        let source = this_file();
        for forbidden in ["std::fs", "File::", "OpenOptions", "BufWriter"] {
            assert!(
                !source.contains(forbidden),
                "microphone.rs now mentions `{forbidden}`. The one thing this \
                 file may never gain is a way to put audio somewhere it lasts \
                 (record 0002, Risk; AGENTS.md data rules)"
            );
        }
    }

    #[test]
    fn the_audio_callback_keeps_no_samples() {
        // The other way audio could be retained: collecting the callback's
        // slice in memory instead of writing it out.
        let source = this_file();
        for forbidden in ["to_vec()", "data.clone()", "extend_from_slice", "VecDeque"] {
            assert!(
                !source.contains(forbidden),
                "microphone.rs now mentions `{forbidden}`. The audio callback \
                 borrows its samples for one call and must keep none of them"
            );
        }
    }
}
