//! Streaming the microphone to Deepgram, and turning what comes back into
//! typed words (record 0002 AC-3, AC-4, AC-8, AC-13, AC-14, AC-20).
//!
//! **Only finalised wording is ever typed.** Deepgram sends the same phrase
//! several times as it makes its mind up, and only the last of those carries
//! `is_final`. AC-4 refuses ever taking typed text back, so the unfinished ones
//! go to the pill as grey text and nowhere near a keystroke. That split is the
//! whole reason this file separates `dictation:interim` from `dictation:text`.
//!
//! **Nothing here keeps the audio.** A chunk of samples is turned into bytes,
//! handed to the socket and dropped. Audio never reaches a disk, a log or a
//! table, whatever else this file does.
//!
//! **What it does keep, and only until the dictation ends, is what it typed.**
//! Milestone 5 needs one row per finished dictation (AC-17), so the finalised
//! phrases are appended to a string in memory as each one lands at the cursor,
//! and [`Session::stop`] hands that string over for the row. It is the same
//! string the joining space is decided from, so what is stored cannot drift
//! from what was typed. Nothing refused is in it, because a phrase is appended
//! only after the keystrokes actually landed. Unfinished wording is never in
//! it: interim results go to the pill and are dropped (AC-33).
//!
//! **The key never leaves this file.** It arrives from the credential vault,
//! goes into the client, and is never logged, never put on an event and never
//! included in an error. Every sentence a person can read here comes from
//! `deepgram_key.rs`, which is fixed wording, so no failure can print anything
//! read off the key.
//!
//! **How a failure is classified, and why the reconnect does the work.** A
//! socket that dies mid dictation says almost nothing useful about why. So this
//! file does not guess: any mid dictation failure triggers the one reconnect
//! AC-14 allows, and it is the *reconnect's* own answer that names the cause. A
//! handshake that comes back 402 says the allowance is spent. One that comes
//! back 403 says the key is not allowed to stream. One that never answers at
//! all says the connection is gone. That way the person is told what Deepgram
//! actually said a moment ago, rather than what a close code was guessed to
//! mean.

use std::sync::{Arc, Mutex, Once};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use deepgram::common::options::{Encoding, Language, Model, Options};
use deepgram::common::stream_response::StreamResponse;
use deepgram::listen::websocket::WebsocketHandle;
use deepgram::{Deepgram, DeepgramError, TungsteniteError};
use serde_json::json;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{self, error::TryRecvError};

use super::deepgram_key::KeyError;
use super::limits::Deadlines;
use super::pill_window;
use super::typing::{self, Typed};

/// The model every dictation runs on.
///
/// Deepgram's current general model. Chosen on 2026-08-30 and written up as
/// point 1 of
/// `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`.
/// AGENTS.md says accuracy is the one thing this app must do well, and the
/// model is the largest single lever on it. It also carries 70+ languages,
/// which is what plan row 4 inherits.
const MODEL: Model = Model::Nova3;

/// How long the one reconnect attempt AC-14 allows is given before it is called
/// a failure, and, separately, the most audio that is held while it runs. One
/// number used for both, so a long outage can never grow a buffer.
const RECONNECT_WINDOW: Duration = Duration::from_secs(5);

/// How long to wait for a message from Deepgram before going back to look for
/// more audio. Not a timeout on anything: it is how the one socket handle is
/// shared between sending and receiving without either starving the other.
const POLL: Duration = Duration::from_millis(20);

/// How many chunks of audio may be waiting for the socket before the oldest are
/// dropped.
///
/// A bound rather than an unbounded queue on purpose. The audio callback must
/// never block or grow without limit, and a socket that has stopped draining is
/// exactly when an unbounded queue would swallow the machine's memory. At the
/// 60 ms cadence the microphone reports on, this is a few seconds of slack.
const AUDIO_QUEUE: usize = 64;

/// Why a dictation ended from this file's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// Deepgram stopped it. Carries the fixed kind, code, sentence and action.
    Deepgram(KeyError),
    /// The focused field was a password field, so nothing was typed and
    /// dictation stops (record 0002 AC-20 and its Risk section).
    BlockedPasswordField,
}

/// Where the microphone puts its samples. Held by the audio callback.
///
/// Made before either end exists, because the microphone needs somewhere to
/// send audio before it has opened, and Deepgram needs the microphone's sample
/// rate before it can be told what it is receiving.
pub struct Feed(mpsc::Sender<Vec<i16>>);

impl Feed {
    /// Hand one chunk of microphone samples to the stream.
    ///
    /// Called from the audio callback, so it never blocks and never waits on a
    /// lock. A full queue drops the chunk rather than stalling capture: losing
    /// a fraction of a second of audio is bad, and glitching the microphone for
    /// everybody is worse.
    pub fn push(&self, samples: Vec<i16>) {
        let _ = self.0.try_send(samples);
    }
}

/// The other end of the feed, handed to `start`.
pub struct Intake(mpsc::Receiver<Vec<i16>>);

/// Make the pair. The microphone gets the [`Feed`], the stream gets the
/// [`Intake`].
pub fn channel() -> (Feed, Intake) {
    let (tx, rx) = mpsc::channel::<Vec<i16>>(AUDIO_QUEUE);
    (Feed(tx), Intake(rx))
}

/// A live connection to Deepgram for one dictation.
///
/// **Stop the microphone before this.** The microphone owns the only [`Feed`],
/// so closing it is what tells the stream there is no more audio coming. That
/// is what lets the last words of a sentence come back rather than being cut
/// off mid phrase.
pub struct Session {
    worker: JoinHandle<()>,
    /// What this dictation has typed so far, in memory only. See
    /// [`Session::stop`] and the transcript paragraph at the top of this file.
    typed: Arc<Mutex<String>>,
}

impl Session {
    /// Wait for the stream to finish and close, and hand back everything this
    /// dictation typed, exactly as it was typed, joining spaces included.
    ///
    /// Safe to read here and nowhere earlier: the worker is the only other
    /// holder and it has finished by the time the join returns, so this is not
    /// reading a transcript while one is still being written. Empty means
    /// nothing was ever typed, and `mod.rs` saves no row for that (AC-17).
    pub fn stop(self) -> String {
        let _ = self.worker.join();
        self.typed
            .lock()
            .map(|typed| typed.clone())
            .unwrap_or_default()
    }
}

/// Start streaming to Deepgram for one dictation.
///
/// Returns as soon as the worker is running, without waiting for the socket.
/// That is deliberate: AC-1 gives the pill one second from the second tap, and
/// a network handshake has no business inside that budget. A key Deepgram
/// refuses is therefore found a moment *after* the pill is up, which is the
/// shape AC-30 already describes: something goes wrong while the microphone is
/// open, so the pill says so and closes.
///
/// `on_end` is called at most once, from the worker thread, when Deepgram or a
/// password field ends the dictation. A normal close never calls it.
pub fn start(
    app: AppHandle,
    key: String,
    sample_rate: u32,
    deadlines: Arc<Mutex<Deadlines>>,
    on_end: Box<dyn Fn(Ended) + Send>,
    intake: Intake,
) -> std::io::Result<Session> {
    let typed = Arc::new(Mutex::new(String::new()));
    let worker_typed = Arc::clone(&typed);
    let worker = std::thread::Builder::new()
        .name("echoscribe-dictate-transcribe".into())
        .spawn(move || {
            // The typing happens on this thread, so this is where UI Automation
            // has to be made available.
            typing::prepare_thread();
            choose_the_cryptography();

            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
            {
                Ok(runtime) => runtime,
                Err(e) => {
                    eprintln!("dictate: could not start the transcription runtime: {e}");
                    on_end(Ended::Deepgram(KeyError::CheckFailed));
                    return;
                }
            };
            runtime.block_on(run(
                app,
                key,
                sample_rate,
                deadlines,
                on_end,
                intake.0,
                worker_typed,
            ));
        })?;

    Ok(Session { worker, typed })
}

/// Say which of the two available cryptography libraries secures the connection
/// to Deepgram, once for the life of the process.
///
/// **Without this the very first connection panics**, and this is the only
/// place in the app that needs it. Two versions of `reqwest` are compiled in,
/// one under sign-in and one under the Deepgram SDK, and each switches on a
/// different provider feature of the shared `rustls`. Both features being on at
/// once is exactly the case rustls refuses to guess about, so it panics rather
/// than pick. Sign-in never met this because its own client names its provider
/// outright; the websocket underneath the Deepgram SDK is the one caller that
/// asks rustls to decide, so it is the one caller that has to be told.
///
/// `aws-lc-rs` because that is what the Deepgram SDK's own REST client selects
/// for itself, so all traffic to Deepgram is secured the same way.
///
/// A second call cannot change what the first chose, which is why the result is
/// ignored: `Once` already makes it a single call, and the ignored error is the
/// harmless case of something else having installed a provider first.
fn choose_the_cryptography() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// The whole life of one dictation's connection.
async fn run(
    app: AppHandle,
    key: String,
    sample_rate: u32,
    deadlines: Arc<Mutex<Deadlines>>,
    on_end: Box<dyn Fn(Ended) + Send>,
    mut audio_rx: mpsc::Receiver<Vec<i16>>,
    typed: Arc<Mutex<String>>,
) {
    let client = match Deepgram::new(&key) {
        Ok(client) => client,
        Err(e) => {
            // Nothing about the key is printed, only that building the client
            // failed. `e` is the SDK's own error and carries no key.
            eprintln!("dictate: could not build the Deepgram client: {e}");
            on_end(Ended::Deepgram(KeyError::CheckFailed));
            return;
        }
    };

    let mut handle = match connect(&client, sample_rate).await {
        Ok(handle) => handle,
        Err(e) => {
            // The very first connection. There is nothing to reconnect to, so
            // the handshake's own answer is the answer.
            on_end(Ended::Deepgram(classify(&e)));
            return;
        }
    };

    // Audio spoken while the socket is down, held in memory only and capped at
    // `RECONNECT_WINDOW` worth of samples. Never written anywhere.
    let mut held: Vec<Vec<i16>> = Vec::new();
    let mut held_samples: usize = 0;
    let hold_cap = sample_rate as usize * RECONNECT_WINDOW.as_secs() as usize;

    // What this dictation has typed so far. It does two jobs and no third: it
    // decides whether a joining space belongs in front of the next finalised
    // phrase, and it is the one row AC-17 asks for once the dictation ends.
    // Both from the same string on purpose, so what is stored can never drift
    // from what was typed.

    loop {
        let mut dropped = false;

        // 1. Everything the microphone has produced since the last pass.
        loop {
            match audio_rx.try_recv() {
                Ok(chunk) => {
                    if handle.send_data(to_bytes(&chunk)).await.is_err() {
                        hold(&mut held, &mut held_samples, hold_cap, chunk);
                        dropped = true;
                        break;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    // The microphone closed. Ask Deepgram for whatever it is
                    // still holding, take those last words, and finish. This is
                    // an ordinary ending and `on_end` is not called.
                    finish(&mut handle, &app, &deadlines, &on_end, &typed).await;
                    return;
                }
            }
        }

        // 2. Whatever Deepgram has to say, if anything is ready.
        if !dropped {
            match tokio::time::timeout(POLL, handle.receive()).await {
                Ok(Some(Ok(response))) => {
                    if let Some(ended) = on_response(response, &app, &deadlines, &typed) {
                        on_end(ended);
                        return;
                    }
                }
                Ok(Some(Err(e))) => {
                    eprintln!("dictate: the Deepgram stream reported an error: {e}");
                    dropped = true;
                }
                // The socket closed under us.
                Ok(None) => dropped = true,
                // Nothing ready yet. Perfectly normal.
                Err(_) => {}
            }
        }

        // 3. AC-14: one reconnect attempt, and the attempt names the cause.
        if dropped {
            match reconnect(
                &client,
                sample_rate,
                &mut audio_rx,
                &mut held,
                &mut held_samples,
                hold_cap,
            )
            .await
            {
                Ok(fresh) => {
                    handle = fresh;
                    // Everything said during the outage, in the order it was
                    // said, then the buffer goes back to empty.
                    for chunk in held.drain(..) {
                        if handle.send_data(to_bytes(&chunk)).await.is_err() {
                            break;
                        }
                    }
                    held_samples = 0;
                }
                Err(cause) => {
                    on_end(Ended::Deepgram(cause));
                    return;
                }
            }
        }
    }
}

/// Open the socket, with everything this feature asks of Deepgram.
async fn connect(client: &Deepgram, sample_rate: u32) -> Result<WebsocketHandle, DeepgramError> {
    let options = Options::builder()
        .model(MODEL)
        // Fixed to English by record 0002's Still open section. Plan row 4 owns
        // making it a choice, and will add a language field to `dictation`.
        .language(Language::en)
        // The words go straight into somebody's document. A person who has to
        // add every full stop by hand has not saved any time.
        .punctuate(true)
        .build();

    client
        .transcription()
        .stream_request_with_options(options)
        // The microphone is downmixed to one channel of signed 16-bit samples
        // before it gets here, at whatever rate the device runs at. One channel
        // because this is one person dictating: sending two would have Deepgram
        // treat them as separate speakers and charge for both.
        .encoding(Encoding::Linear16)
        .sample_rate(sample_rate)
        .channels(1)
        // The grey half of the pill's transcript line. Only finalised wording is
        // ever typed, so these never reach a keystroke.
        .interim_results(true)
        .handle()
        .await
}

/// The one reconnect attempt AC-14 allows.
///
/// It keeps taking audio while it tries, so a blip costs the person nothing,
/// and it gives up at `RECONNECT_WINDOW`. Whatever the attempt comes back with
/// is what names the cause, which is why nothing above this tries to read
/// meaning out of a close code.
async fn reconnect(
    client: &Deepgram,
    sample_rate: u32,
    audio_rx: &mut mpsc::Receiver<Vec<i16>>,
    held: &mut Vec<Vec<i16>>,
    held_samples: &mut usize,
    hold_cap: usize,
) -> Result<WebsocketHandle, KeyError> {
    let deadline = Instant::now() + RECONNECT_WINDOW;
    let mut last = KeyError::ConnectionLost;

    while Instant::now() < deadline {
        // Keep the microphone's output, so a recovered connection does not
        // leave a silent hole in the middle of a sentence.
        while let Ok(chunk) = audio_rx.try_recv() {
            hold(held, held_samples, hold_cap, chunk);
        }

        match connect(client, sample_rate).await {
            Ok(handle) => return Ok(handle),
            Err(e) => {
                last = classify(&e);
                // A refusal with an answer is final: Deepgram has told us the
                // key is spent or not allowed, and trying again for five
                // seconds cannot change that.
                if last != KeyError::ConnectionLost {
                    return Err(last);
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    }
    Err(last)
}

/// Keep a chunk for the reconnect, oldest first out once the cap is reached.
fn hold(held: &mut Vec<Vec<i16>>, held_samples: &mut usize, cap: usize, chunk: Vec<i16>) {
    *held_samples += chunk.len();
    held.push(chunk);
    while *held_samples > cap && !held.is_empty() {
        let oldest = held.remove(0);
        *held_samples = held_samples.saturating_sub(oldest.len());
    }
}

/// The microphone closed normally. Ask for the last words and take them.
async fn finish(
    handle: &mut WebsocketHandle,
    app: &AppHandle,
    deadlines: &Arc<Mutex<Deadlines>>,
    on_end: &(dyn Fn(Ended) + Send),
    typed: &Mutex<String>,
) {
    if handle.finalize().await.is_err() {
        return;
    }
    if handle.close_stream().await.is_err() {
        return;
    }
    // Deepgram sends a terminal message once it is done. Bounded so a socket
    // that never sends one cannot hold the close open.
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        match tokio::time::timeout(POLL, handle.receive()).await {
            Ok(Some(Ok(StreamResponse::TerminalResponse { .. }))) | Ok(None) => return,
            Ok(Some(Ok(response))) => {
                if let Some(ended) = on_response(response, app, deadlines, typed) {
                    on_end(ended);
                    return;
                }
            }
            Ok(Some(Err(_))) => return,
            Err(_) => {}
        }
    }
}

/// One message from Deepgram. Returns `Some` only when it ends the dictation.
fn on_response(
    response: StreamResponse,
    app: &AppHandle,
    deadlines: &Arc<Mutex<Deadlines>>,
    typed: &Mutex<String>,
) -> Option<Ended> {
    let StreamResponse::TranscriptResponse {
        is_final, channel, ..
    } = response
    else {
        return None;
    };

    let text = channel
        .alternatives
        .first()
        .map(|alternative| alternative.transcript.trim())
        .unwrap_or_default();
    if text.is_empty() {
        return None;
    }

    if !is_final {
        // The grey half of the transcript line. It goes to the pill and
        // nowhere else, and it is never typed, never stored and never logged.
        let _ = app.emit_to(
            pill_window::LABEL,
            "dictation:interim",
            json!({ "text": text }),
        );
        return None;
    }

    // AC-8: this is the only thing in the app that can honestly say a person
    // spoke. Loudness cannot: a fan does that too.
    if let Ok(mut caps) = deadlines.lock() {
        caps.speech_heard(Instant::now());
    }

    // A single space joins consecutive finalised phrases (record 0002,
    // settled 2026-08-31). Deepgram hands every phrase over trimmed, so
    // without this two phrases collide into one word at the cursor. The
    // space rides in front of the phrase through the same one door, so the
    // password check covers it too. Nothing goes in front of a dictation's
    // first phrase, and the events below carry the phrase exactly as it
    // arrived.
    //
    // "Has typed anything yet" is read off the same string that will be saved,
    // so the space in the row and the space at the cursor are the one space.
    let typed_anything_yet = typed.lock().map(|typed| !typed.is_empty());
    let mut to_type = String::with_capacity(text.len() + 1);
    if typed_anything_yet.unwrap_or(false) {
        to_type.push(' ');
    }
    to_type.push_str(text);

    match typing::type_at_cursor(&to_type) {
        Typed::AtTheCursor => {
            // AC-17: what reached the cursor, and only that, is what the row
            // holds. Appended after the keystrokes landed rather than before,
            // so nothing refused is ever in it.
            if let Ok(mut typed) = typed.lock() {
                typed.push_str(&to_type);
            }
            // Broadcast rather than sent to the pill alone. Record 0002's
            // clearing table gives a spent allowance one proof that it is over,
            // the first finalised words, and the window holding that message
            // has to receive them.
            let _ = app.emit("dictation:text", json!({ "text": text }));
            None
        }
        Typed::RefusedPasswordField => {
            let _ = app.emit_to(
                pill_window::LABEL,
                "dictation:blocked",
                json!({
                    "code": "BLOCKED_PASSWORD_FIELD",
                    "message": "EchoScribe will not type into a password field.",
                }),
            );
            Some(Ended::BlockedPasswordField)
        }
    }
}

/// Signed 16-bit samples, little endian, which is what `Encoding::Linear16`
/// means.
fn to_bytes(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

/// What a refused or failed connection means, in this feature's own words.
///
/// Only the handshake's status is read. Nothing is guessed from a close code
/// and nothing is read out of a message body, so no wording can drift with a
/// change at Deepgram's end. Anything that is not an answer at all, a dead
/// network or a name that will not resolve, is a lost connection.
fn classify(e: &DeepgramError) -> KeyError {
    match e {
        DeepgramError::WsError(ws) => match ws.as_ref() {
            TungsteniteError::Http(response) => {
                KeyError::from_stream_status(response.status().as_u16())
            }
            _ => KeyError::ConnectionLost,
        },
        _ => KeyError::ConnectionLost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This module's own source, minus its tests.
    fn this_file() -> &'static str {
        include_str!("transcribe.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    #[test]
    fn samples_become_little_endian_pairs_of_bytes() {
        // covers: AC-3. Linear16 is signed 16-bit little endian. Get the byte
        // order wrong and Deepgram receives noise, transcribes nothing, and
        // nothing on screen says why.
        assert_eq!(to_bytes(&[0x0102]), vec![0x02, 0x01]);
        assert_eq!(to_bytes(&[-1]), vec![0xFF, 0xFF]);
        assert_eq!(to_bytes(&[0, 1]), vec![0x00, 0x00, 0x01, 0x00]);
        assert_eq!(to_bytes(&[]).len(), 0);
    }

    #[test]
    fn every_sample_survives_the_conversion() {
        // covers: AC-3. Two bytes per sample, always, so a dropped or added
        // byte cannot shift every sample after it by half a value.
        let samples: Vec<i16> = (-500..500).collect();
        assert_eq!(to_bytes(&samples).len(), samples.len() * 2);
    }

    #[test]
    fn the_held_audio_never_grows_past_its_cap() {
        // covers: AC-14, and AGENTS.md's rule that audio is transient. The
        // reconnect holds what the person says so a blip costs them nothing,
        // and the cap is what stops a long outage eating the machine.
        let mut held = Vec::new();
        let mut samples = 0usize;
        let cap = 100;

        for _ in 0..50 {
            hold(&mut held, &mut samples, cap, vec![0i16; 10]);
        }
        assert!(
            samples <= cap,
            "held {samples} samples against a cap of {cap}"
        );
        assert!(
            held.iter().map(|c| c.len()).sum::<usize>() <= cap,
            "the buffer itself is over the cap even though the count is not"
        );
    }

    #[test]
    fn the_oldest_audio_is_the_first_dropped() {
        // covers: AC-14. What a person said most recently is what they are
        // still expecting to see appear.
        let mut held = Vec::new();
        let mut samples = 0usize;
        hold(&mut held, &mut samples, 4, vec![1i16, 1]);
        hold(&mut held, &mut samples, 4, vec![2i16, 2]);
        hold(&mut held, &mut samples, 4, vec![3i16, 3]);
        assert_eq!(held.len(), 2, "the cap holds two chunks of two");
        assert_eq!(held[0][0], 2, "the first chunk was dropped, not the last");
        assert_eq!(held[1][0], 3);
    }

    #[test]
    fn a_refusal_with_an_answer_is_never_called_a_lost_connection() {
        // covers: AC-13, AC-14. These send a person to completely different
        // places: one to their Deepgram console, one to try again. Confusing
        // them is the mistake record 0002 refuses throughout.
        assert_eq!(KeyError::from_stream_status(402), KeyError::NoAllowance);
        assert_eq!(KeyError::from_stream_status(403), KeyError::KeyNotAllowed);
        assert_eq!(KeyError::from_stream_status(401), KeyError::Rejected);
        assert_ne!(KeyError::from_stream_status(402), KeyError::ConnectionLost);
    }

    #[test]
    fn an_answer_deepgram_did_not_explain_is_the_honest_catch_all() {
        // covers: AC-13. Same reasoning as the fourth microphone error and the
        // fourth key error: a rate limit or an outage is not a bad key, and
        // saying so sends the person to replace one that was fine.
        assert_eq!(KeyError::from_stream_status(429), KeyError::CheckFailed);
        assert_eq!(KeyError::from_stream_status(500), KeyError::CheckFailed);
    }

    #[test]
    fn nothing_here_can_write_audio_or_words_anywhere() {
        // covers: AGENTS.md data rules. Audio is captured, sent and discarded.
        // Transcribed text goes to the cursor and to local history, and this
        // file is neither of those places: it is the pipe between them.
        let source = this_file();
        for forbidden in ["std::fs", "File::", "OpenOptions", "BufWriter", "create("] {
            assert!(
                !source.contains(forbidden),
                "transcribe.rs now mentions `{forbidden}`. Neither the audio \
                 nor the words may be put anywhere they last"
            );
        }
    }

    #[test]
    fn no_log_line_here_can_carry_the_key_the_audio_or_the_words() {
        // covers: record 0002's Risk section, and AGENTS.md data rules. Every
        // message this file prints goes to stderr and on a bad day into a
        // support file, so none of them may interpolate a secret or a person's
        // speech.
        //
        // The whole source, with no line filter, the same way the two guards
        // above it work. An earlier version only inspected lines beginning with
        // `eprintln!`, so a message wrapped onto the line below its macro was
        // never read. Scanning the source cannot be defeated by how a message
        // happens to be wrapped.
        let source = this_file();
        for forbidden in [
            "{key}",
            "{text}",
            "{chunk}",
            "{samples}",
            "{response}",
            "{transcript}",
            "{alternative}",
        ] {
            assert!(
                !source.contains(forbidden),
                "transcribe.rs now mentions `{forbidden}`. Neither the key, the \
                 audio nor the words a person spoke may reach stderr"
            );
        }
    }

    #[test]
    fn the_model_is_the_one_that_was_decided() {
        // covers: the milestone 4 decision of 2026-08-30, point 1. A model swap
        // changes accuracy, cost and which languages plan row 4 can offer, so
        // it is a decision and never a tidy-up.
        assert_eq!(MODEL, Model::Nova3);
    }

    #[test]
    fn choosing_the_cryptography_leaves_a_provider_installed() {
        // covers: AC-3, and the crash fixed at 64bf2c7. Two versions of
        // reqwest each switch on a different crypto provider feature of the
        // shared rustls, rustls refuses to guess between two, and the very
        // first websocket connect panicked, killing the transcription thread
        // silently: pill up, meter moving, nothing typed, no error anywhere.
        // The cure is one install before any connect. This calls the same
        // function the worker thread calls, then asks rustls the question the
        // connect would ask. Gut the install out of `choose_the_cryptography`
        // and this fails, because nothing else in this crate installs a
        // process default: sign-in's client names its provider per client.
        choose_the_cryptography();
        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "no process-default crypto provider is installed, so the first \
             websocket connect will panic again (the crash of 2026-08-31, \
             fixed at 64bf2c7)"
        );
    }

    #[test]
    fn the_worker_thread_still_chooses_the_cryptography() {
        // covers: AC-3, as a source guard only. The test above keeps
        // `choose_the_cryptography` correct, and by calling it, also keeps
        // the compiler's dead code warning from ever noticing the call site
        // going missing. So the call site needs its own watcher: this fails
        // if the worker thread stops making the call. The panic it prevents
        // only fires on a live connect to Deepgram, which no automated test
        // reaches, which is why the call is guarded at the source. Flattened
        // so a formatter moving the call across lines changes nothing.
        let flat: String = this_file()
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect();
        assert!(
            flat.contains("choose_the_cryptography();"),
            "transcribe.rs no longer calls choose_the_cryptography() on the \
             worker thread. The very first websocket connect will panic and \
             the thread will die silently (fixed at 64bf2c7)"
        );
    }

    #[test]
    fn the_reconnect_holds_no_more_than_it_waits() {
        // covers: AC-14. One number used twice is what makes the promise
        // checkable: the buffer can never outlast the attempt that justifies it.
        assert_eq!(RECONNECT_WINDOW, Duration::from_secs(5));
        let at_48k = 48_000 * RECONNECT_WINDOW.as_secs() as usize;
        assert_eq!(at_48k, 240_000, "five seconds of 48 kHz mono samples");
    }
}
