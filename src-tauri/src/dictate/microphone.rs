//! Capturing the microphone, and turning it into one loudness number.
//!
//! **No audio is kept.** Record 0002 refuses writing audio to disk or into a
//! log, ever, and AGENTS.md's data rules say recorded audio is transient:
//! captured, sent for transcription, and discarded.
//!
//! **This changed shape in milestone 4 and the change is worth reading.** Until
//! milestone 4 the only thing that left the audio callback was a running total
//! of squared sample values, and nothing here could produce a sound at all.
//! Milestone 4 is where audio genuinely starts leaving, because that is the
//! feature: a person's voice goes to Deepgram. So the guarantee is now narrower
//! and more exact, and it is this:
//!
//!   * The callback turns its borrowed samples into one chunk, hands that chunk
//!     to exactly one sink, and keeps nothing. When it returns, both the
//!     borrowed slice and the chunk are gone from here.
//!   * Nothing in this file accumulates audio across callbacks. There is no
//!     buffer, no ring and no growing vector. The only state that survives a
//!     callback is two numbers for the loudness meter.
//!   * There is still no file handle, no network call and no log line carrying
//!     audio, and the tests at the bottom guard all three.
//!
//! Where the chunk goes next is the caller's business, and the caller is
//! `transcribe.rs`, which sends it to Deepgram and drops it. Nothing on either
//! side keeps it.
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

use std::sync::atomic::{AtomicU64, Ordering};
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

    /// The same sentence, for a failure that arrived after the microphone had
    /// opened. Three of the four hold either way; the catch-all's does not,
    /// because "could not be opened" is false about a microphone that did open,
    /// so it gets the record's second sentence. Same kind, same code, same one
    /// action, never a fifth code (record 0002, the device settlement of
    /// 2026-08-30).
    pub fn message_mid_dictation(self) -> &'static str {
        match self {
            MicError::Unavailable => "The microphone stopped working.",
            other => other.message(),
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
    sample_rate: u32,
}

impl Microphone {
    /// How many samples a second this device is running at.
    ///
    /// Deepgram has to be told, because linear16 carries no header saying so,
    /// and a wrong rate transcribes as gibberish rather than as an error. Read
    /// from the device rather than assumed: record 0002 stores no device and
    /// offers no picker, so whatever Windows is set to is what this is.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

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
            sample_rate: _,
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
/// from 0.0 (silence) to 1.0 (loud).
///
/// `on_died` is called if the stream fails after it opened, with the failure
/// already classified into the same four named kinds an open failure gets: the
/// record's device settlement runs one classification, twice (record 0002,
/// AC-30). It is called from the audio backend's own thread, so it must not
/// block and must not touch this microphone; the caller sends a command and
/// returns. Nothing about the failure but the kind ever leaves this module.
///
/// Returns before the first tick, and only once the device is genuinely
/// capturing: a failure comes back here rather than turning up later, which is
/// what lets the caller keep AC-15's promise that the pill never appears when
/// the microphone will not open.
pub fn open(
    on_level: Box<dyn Fn(f32) + Send>,
    on_audio: Box<dyn Fn(Vec<i16>) + Send>,
    on_died: Arc<dyn Fn(MicError) + Send + Sync>,
) -> Result<Microphone, MicError> {
    let shared = Arc::new(Mutex::new(Loudness::default()));

    let (ready_tx, ready_rx) = mpsc::channel::<Result<u32, MicError>>();
    let (stop_audio, stop_audio_rx) = mpsc::channel::<()>();
    let audio_shared = Arc::clone(&shared);

    let audio = spawn("echoscribe-dictate-audio", move || {
        // Everything to do with the stream happens here, on this one thread,
        // for the whole life of the stream. cpal owns it; we do not move it.
        let (stream, sample_rate) = match build_stream(audio_shared, on_audio, on_died) {
            Ok(built) => built,
            Err(e) => {
                let _ = ready_tx.send(Err(e));
                return;
            }
        };
        if let Err(e) = stream.play() {
            let _ = ready_tx.send(Err(logged_cause("starting the capture stream", &e)));
            return;
        }
        let _ = ready_tx.send(Ok(sample_rate));

        // Park until told to stop, or until the sender goes away.
        let _ = stop_audio_rx.recv();
        // Explicit, because this line is the one that closes the microphone.
        drop(stream);
    })?;

    let sample_rate = match ready_rx.recv() {
        Ok(Ok(rate)) => rate,
        Ok(Err(e)) => {
            let _ = audio.join();
            return Err(e);
        }
        // The thread ended without answering.
        Err(_) => {
            eprintln!(
                "dictate: the audio thread ended without saying whether the microphone opened."
            );
            let _ = audio.join();
            return Err(MicError::Unavailable);
        }
    };

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
            sample_rate,
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
fn build_stream(
    shared: Arc<Mutex<Loudness>>,
    on_audio: Box<dyn Fn(Vec<i16>) + Send>,
    on_died: Arc<dyn Fn(MicError) + Send + Sync>,
) -> Result<(cpal::Stream, u32), MicError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(MicError::NoMicrophoneFound)?;
    let supported = device
        .default_input_config()
        .map_err(|e| logged_cause("reading the default input format", &e))?;
    let format = supported.sample_format();
    let config = supported.config();
    let sample_rate = config.sample_rate;
    let channels = config.channels as usize;

    let built = match format {
        SampleFormat::F32 => capture::<f32>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::F64 => capture::<f64>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::I8 => capture::<i8>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::I16 => capture::<i16>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::I32 => capture::<i32>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::U8 => capture::<u8>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::U16 => capture::<u16>(&device, config, shared, channels, on_audio, on_died),
        SampleFormat::U32 => capture::<u32>(&device, config, shared, channels, on_audio, on_died),
        other => {
            eprintln!("dictate: the microphone records in {other:?}, which nothing here can read.");
            return Err(MicError::Unavailable);
        }
    };
    built
        .map(|stream| (stream, sample_rate))
        .map_err(|e| logged_cause("building the capture stream", &e))
}

/// Classify a failure to open the microphone, and say what it really was.
///
/// **The saying is the point.** `MicError` is four buckets, and three quarters
/// of what the audio layer knows is thrown away making one. That was fine
/// until a microphone would not open on 2026-09-03: the person saw "The
/// microphone could not be opened", and so did everybody trying to find out
/// why, because nothing anywhere had kept the reason. A whole investigation
/// went on a question one line could have answered.
///
/// Safe to log. A cpal error names the stage, the error kind and the Windows
/// code. It never carries audio, a key or anything a person said, which is
/// what AGENTS.md's data rules forbid.
fn logged_cause(stage: &str, e: &cpal::Error) -> MicError {
    let cause = classify(e.kind());
    eprintln!(
        "dictate: the microphone would not open, {stage}: kind={:?}, {e}. Reported as {cause:?}.",
        e.kind()
    );
    cause
}

/// The audio callback, for one sample type.
///
/// This is the only code that ever sees the person's voice. It reads each
/// sample once, adds it to a running total for the meter, and turns the frame
/// into one signed 16-bit value for Deepgram. When it returns, `data` is gone
/// and so is the chunk it made.
///
/// **Downmixed to one channel**, by averaging the channels of each frame. One
/// person is dictating into one microphone; handing Deepgram two channels would
/// have it treat them as two speakers and charge for both. Averaging rather
/// than taking the first channel, because on some devices the first channel is
/// the quiet one.
fn capture<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    shared: Arc<Mutex<Loudness>>,
    channels: usize,
    on_audio: Box<dyn Fn(Vec<i16>) + Send>,
    on_died: Arc<dyn Fn(MicError) + Send + Sync>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = channels.max(1);
    // Counted per dictation, so the rate limit below cannot be defeated by a
    // long session and cannot carry over into the next one.
    let glitches = AtomicU64::new(0);
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut sum = 0.0f64;
            let mut mono: Vec<i16> = Vec::with_capacity(data.len() / channels + 1);
            for frame in data.chunks(channels) {
                let mut total = 0.0f32;
                for sample in frame {
                    total += f32::from_sample(*sample);
                }
                let value = total / frame.len() as f32;
                sum += (value as f64) * (value as f64);
                mono.push(to_i16(value));
            }
            let frames = mono.len() as u64;

            // `try_lock`, never `lock`: an audio callback that waits on another
            // thread is how capture starts glitching. Losing one 60 ms window's
            // worth of level is not worth a stall.
            if let Ok(mut acc) = shared.try_lock() {
                acc.sum_of_squares += sum;
                acc.samples += frames;
            }

            // The one way audio leaves this file. The sink does not block: it
            // hands the chunk to a bounded queue and returns.
            on_audio(mono);
        },
        // The stream died after it opened. Until 2026-08-30 this went to stderr
        // and nowhere else, and a person watching the pill saw MIC OPEN over a
        // dead device (record 0002, "The microphone dying after it opened").
        // The same classification an open failure gets, then straight out to
        // the caller, which ends the dictation before AC-8's silence cap can
        // swallow the evidence. The error's text never leaves this line.
        move |e| {
            // Not every report here is a death, and reading them all as one is
            // what ended three dictations by itself on 2026-09-02.
            if !stopped_capturing(e.kind()) {
                let n = glitches.fetch_add(1, Ordering::Relaxed) + 1;
                // The first one, then every fiftieth. A busy machine can report
                // dozens a second, and a log nobody can read is not a log.
                if n == 1 || n.is_multiple_of(50) {
                    eprintln!(
                        "dictate: the microphone glitched, dropping a moment of \
                         audio. Dictation continues. ({n} so far.)"
                    );
                }
                return;
            }
            eprintln!("dictate: the microphone stream reported an error: {e}");
            on_died(classify(e.kind()));
        },
        None,
    )
}

/// One averaged sample, as the signed 16-bit value `Encoding::Linear16` means.
///
/// Clamped, and scaled by `i16::MAX` rather than `i16::MIN`, so a sample at or
/// past full scale cannot wrap round to the opposite sign and arrive at
/// Deepgram as a click in the middle of a word.
fn to_i16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
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

/// Whether a report from an open stream means the microphone has actually
/// stopped capturing.
///
/// **cpal uses one callback for two different things**, and this is the
/// distinction the code was missing. Most kinds arrive from the audio
/// backend's run loop as it gives up, and after them no more audio comes. One
/// kind, `Xrun`, is a *notification*: the sound card lost a moment of audio
/// and Windows says so by raising a discontinuity flag on the very next packet
/// it hands over. cpal reports it and keeps reading that packet and every one
/// after it. Its own documentation says so in as many words: "causing a
/// potential audio glitch", and for the same kind elsewhere, "audio will still
/// play".
///
/// Measured on this machine on 2026-09-02, not assumed. With the audio thread
/// starved by ordinary CPU load, the built-in Realtek array reported this 88
/// times in 35 seconds and delivered 1,429,632 more frames after the first
/// one. Idle for 45 seconds it reported none. Evidence:
/// `docs/evidence/dictate-with-a-hotkey/finding-microphone-dies-on-its-own.md`.
///
/// Only `Xrun` is listed, and it is the only non-fatal kind Windows can
/// produce. A default-device change looks similar and is not the same thing:
/// cpal reports it as the stream being invalidated or the device being gone,
/// the old device stays bound, and ending the dictation is the right answer
/// (AC-31). `RealtimeDenied` is the other non-fatal kind in the library and
/// cannot arrive at all: it needs cpal's `realtime` feature, which is off.
fn stopped_capturing(kind: ErrorKind) -> bool {
    !matches!(kind, ErrorKind::Xrun)
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

    /// The live proof, opposite the bug. Ignored by default because it opens
    /// the real microphone and takes half a minute.
    ///
    ///   cargo test -- --ignored a_load_starved_microphone
    ///
    /// It starves the audio thread with ordinary CPU load, which is what the
    /// harness-driven verify sitting of 2026-09-02 was doing without meaning
    /// to, and then asserts the two things that were false that day: nothing
    /// reports the microphone as dead, and audio keeps arriving throughout.
    ///
    /// **What it cannot prove on its own.** It cannot see the glitch count,
    /// because that number stays inside the stream. A run where Windows
    /// happens to report no glitch at all passes this test without testing
    /// anything. The glitch count was measured separately, by a spike that
    /// printed it: 88 reports in 35 seconds under this same load, and
    /// 1,429,632 frames delivered after the first. Those numbers are in
    /// `docs/evidence/dictate-with-a-hotkey/finding-microphone-dies-on-its-own.md`.
    ///
    /// **No audio is kept**, here least of all. The chunks are counted and
    /// dropped, the same as everywhere else in this file.
    #[test]
    #[ignore]
    fn a_load_starved_microphone_does_not_end_the_dictation() {
        // covers: AC-30. 64 spinners on an 8 core machine was enough to make
        // Windows report a glitch within 5 seconds, every time it was tried.
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        for _ in 0..64 {
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                let mut x = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
                    std::hint::black_box(x);
                }
            });
        }

        let died = Arc::new(Mutex::new(Vec::<MicError>::new()));
        let frames = Arc::new(AtomicU64::new(0));
        let chunks = Arc::new(AtomicU64::new(0));

        let died_here = Arc::clone(&died);
        let counted = Arc::clone(&frames);
        let chunked = Arc::clone(&chunks);
        let mic = microphone_or_skip(open(
            Box::new(|_| {}),
            Box::new(move |samples: Vec<i16>| {
                counted.fetch_add(samples.len() as u64, Ordering::Relaxed);
                chunked.fetch_add(1, Ordering::Relaxed);
                // Dropped here, like everywhere else. Nothing is kept.
            }),
            Arc::new(move |e| {
                died_here
                    .lock()
                    .expect("the death list mutex poisoned")
                    .push(e);
            }),
        ));
        let Some(mic) = mic else { return };

        std::thread::sleep(Duration::from_secs(25));
        let halfway = frames.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_secs(5));

        let total = frames.load(Ordering::Relaxed);
        let deaths = died.lock().expect("the death list mutex poisoned").clone();
        stop.store(true, Ordering::Relaxed);
        mic.stop();

        println!(
            "30 s under load: {} chunks, {total} frames, {} reported deaths",
            chunks.load(Ordering::Relaxed),
            deaths.len()
        );
        assert!(
            deaths.is_empty(),
            "reported dead {} time(s) under nothing but CPU load: {deaths:?}. \
             This is the bug of 2026-09-02 back again",
            deaths.len()
        );
        assert!(
            total > halfway,
            "no audio arrived in the last 5 seconds, so the stream really did \
             stop and the fix is hiding a genuine failure"
        );
    }

    /// `None` when this machine has no microphone to test with, so the suite
    /// stays runnable on a machine without one. A real failure to open still
    /// fails the test: a blocked or busy microphone is a result, not an
    /// absence.
    fn microphone_or_skip(opened: Result<Microphone, MicError>) -> Option<Microphone> {
        match opened {
            Ok(mic) => Some(mic),
            Err(MicError::NoMicrophoneFound) => {
                println!("skipped: this machine has no input device");
                None
            }
            Err(e) => panic!("the microphone would not open at all: {e:?}"),
        }
    }

    #[test]
    fn a_glitch_is_not_a_death() {
        // covers: AC-30. The bug of 2026-09-02: three dictations ended on their
        // own because every report from an open stream was read as the
        // microphone stopping. `Xrun` is not that. Windows raises it on the
        // next packet after the sound card loses a moment of audio, and it
        // hands that packet over and every one after it. Measured live: 88
        // reports in 35 seconds under load, 1,429,632 frames delivered after
        // the first. Ending a dictation on one costs the person their sentence
        // for a hiccup they would not otherwise have noticed.
        assert!(
            !stopped_capturing(ErrorKind::Xrun),
            "a dropped moment of audio is a glitch, not a dead microphone"
        );
    }

    #[test]
    fn everything_else_from_an_open_stream_still_ends_the_dictation() {
        // covers: AC-30. The other half, and the half that must not be lost to
        // the fix above. After any of these no more audio arrives, so saying
        // nothing would leave MIC OPEN over a dead device, which is exactly
        // what the settlement of 2026-08-30 was written to stop.
        for kind in [
            ErrorKind::DeviceNotAvailable,
            ErrorKind::DeviceBusy,
            ErrorKind::HostUnavailable,
            ErrorKind::StreamInvalidated,
            ErrorKind::BackendError,
            ErrorKind::PermissionDenied,
            ErrorKind::Other,
        ] {
            assert!(
                stopped_capturing(kind),
                "{kind:?} means no more audio is coming and must end the dictation"
            );
        }
    }

    #[test]
    fn a_default_device_change_is_not_treated_as_a_glitch() {
        // covers: AC-30, AC-31. This is the near miss. Changing the Windows
        // default mid dictation reports as the stream being invalidated, or as
        // the device being gone when there is no replacement. cpal keeps its
        // run loop alive there too, but the old device stays bound, so the
        // person would go on dictating into the microphone they just stopped
        // using. That is an ending, not a hiccup.
        assert!(stopped_capturing(ErrorKind::StreamInvalidated));
        assert!(stopped_capturing(ErrorKind::DeviceNotAvailable));
        assert_eq!(
            classify(ErrorKind::DeviceNotAvailable),
            MicError::NoMicrophoneFound
        );
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
    fn only_the_catch_all_changes_its_sentence_mid_dictation() {
        // covers: AC-30, the device settlement of 2026-08-30. "The microphone
        // could not be opened." is false about a microphone that did open, so
        // the catch-all gets the record's second sentence mid dictation. The
        // other three sentences are true either way and must not change: same
        // kinds, same codes, never a fifth.
        assert_eq!(
            MicError::Unavailable.message_mid_dictation(),
            "The microphone stopped working."
        );
        for kind in [
            MicError::BlockedByWindows,
            MicError::InUseByAnotherApp,
            MicError::NoMicrophoneFound,
        ] {
            assert_eq!(kind.message_mid_dictation(), kind.message());
        }
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
    fn the_audio_callback_accumulates_nothing_across_calls() {
        // Milestone 4 changed what this guard has to protect, and the change is
        // deliberate. Until milestone 4 the callback produced no audio at all
        // and the guard forbade it owning any. Milestone 4 is where a person's
        // voice legitimately starts leaving, so the narrower and still exact
        // promise is that nothing is retained *between* callbacks: one chunk is
        // made, handed to one sink, and dropped.
        //
        // These are the shapes that would break it, by holding audio in state
        // that outlives a single call.
        let source = this_file();
        for forbidden in ["VecDeque", "static AUDIO", "lazy_static", "OnceLock<Vec"] {
            assert!(
                !source.contains(forbidden),
                "microphone.rs now mentions `{forbidden}`. The callback may                  build one chunk and hand it on; it may never keep audio from                  one call to the next (AGENTS.md data rules: audio is transient)"
            );
        }
    }

    #[test]
    fn there_is_exactly_one_way_audio_leaves_this_file() {
        // The chunk is handed to the sink the caller passed in, and to nothing
        // else. A second call site would be a second destination for a person's
        // voice, which is the thing the data rules are most exact about.
        let source = this_file();
        assert_eq!(
            source.matches("on_audio(").count(),
            1,
            "audio now leaves microphone.rs by more than one route"
        );
    }

    #[test]
    fn a_full_scale_sample_never_wraps_to_the_opposite_sign() {
        // covers: AC-3. Scaling by i16::MIN, or not clamping, turns the loudest
        // moment of a word into a click at the other extreme. Deepgram hears
        // that as noise, and nothing on screen would say why accuracy dropped.
        assert_eq!(to_i16(1.0), i16::MAX);
        assert_eq!(
            to_i16(1.5),
            i16::MAX,
            "past full scale is held, not wrapped"
        );
        assert_eq!(to_i16(-1.5), -i16::MAX, "and the same the other way");
        assert!(to_i16(-1.0) < 0);
        assert_eq!(to_i16(0.0), 0);
    }

    #[test]
    fn silence_converts_to_silence() {
        // covers: AC-3. A quiet room must reach Deepgram as quiet, not as a
        // constant offset, which would read as a hum and cost accuracy.
        assert_eq!(to_i16(0.0), 0);
        assert_eq!(to_i16(-0.0), 0);
    }
}
