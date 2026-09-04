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

/// What this dictation asks Deepgram for, beyond the fixed model, punctuation
/// and interim results.
///
/// **Both values are read once, before the microphone opens, and never again
/// for the life of this dictation.** That is what makes record 0005's AC-3 and
/// record 0006's AC-4 promises rather than races: a person who changes their
/// language or adds a word while dictating changes the *next* dictation, and
/// the one reconnect attempt AC-14 allows re-sends exactly what the first
/// connection asked for. Holding them here is what makes that structural: there
/// is nowhere in this file to ask again.
///
/// Neither value is ever logged. The language is one of 64 literals and says
/// what language somebody speaks; the terms are a person's own custom
/// vocabulary, which record 0005's risk section treats as personal data.
pub struct Asked {
    /// One of the 64 literals in the language feature's catalogue, never a
    /// string that came from the interface (record 0006 AC-3).
    pub language: &'static str,
    /// This account's custom vocabulary, already inside Deepgram's limits by
    /// the vocabulary feature's own rules, and empty when there is none
    /// (record 0005 AC-3, AC-10).
    pub keyterms: Vec<String>,
    /// The rate the microphone actually opened at. It belongs here for the same
    /// reason as the other two, and it is the same rule: this is what Deepgram
    /// is told about this one stream, fixed before the first connection and
    /// re-sent unchanged by the reconnect. A reconnect that described the audio
    /// differently from the socket it replaced would garble every word after
    /// it.
    pub sample_rate: u32,
}

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
    asked: Asked,
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
                asked,
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
    asked: Asked,
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

    let mut handle = match connect(&client, &asked).await {
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
    let hold_cap = asked.sample_rate as usize * RECONNECT_WINDOW.as_secs() as usize;

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
                &asked,
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

/// Everything this feature asks of Deepgram for one stream, as options.
///
/// Pure, and separate from [`connect`] for one reason: **it is the only place
/// in this app where what is asked of an outside service is decided, and it is
/// worth being able to assert the exact query rather than trust a reading of
/// the code.** The tests at the bottom of this file do exactly that, through
/// the SDK's own `Options::urlencoded`, which is what makes record 0005's AC-14
/// and record 0006's AC-3 and AC-12 provable rather than argued.
///
/// Called from [`connect`] and from nowhere else, so the first connection and
/// the one reconnect attempt cannot ask for different things.
fn stream_options(asked: &Asked) -> Options {
    let mut options = Options::builder()
        .model(MODEL)
        // The account's chosen language, from record 0006, read once before the
        // microphone opened. English until a person chooses otherwise, which is
        // exactly what record 0002 had fixed here in code.
        //
        // `Language::from` maps a known tag to its own variant and anything
        // else to the SDK's documented `Other`, and both serialise to the tag
        // itself, so one line covers all 64. The string is always one of that
        // feature's literals: nothing that arrived from the interface reaches
        // here (record 0006's risk section).
        .language(Language::from(asked.language.to_string()))
        // The words go straight into somebody's document. A person who has to
        // add every full stop by hand has not saved any time.
        .punctuate(true);

    // Record 0005: the account's custom vocabulary, as Deepgram's `keyterm`.
    // Empty means the parameter is not sent at all, so an account with no words
    // asks for exactly what this feature asked for before that record existed.
    // The SDK percent encodes each term, which is what stops a word holding an
    // ampersand from adding a parameter of its own (record 0005 AC-14).
    if !asked.keyterms.is_empty() {
        options = options.keyterms(asked.keyterms.iter().map(String::as_str));
    }
    options.build()
}

/// Open the socket, with everything this feature asks of Deepgram.
///
/// `asked` carries the two things that are the person's rather than this
/// record's, and it is borrowed rather than taken so that the one reconnect
/// attempt asks for exactly the same thing this call did.
async fn connect(client: &Deepgram, asked: &Asked) -> Result<WebsocketHandle, DeepgramError> {
    client
        .transcription()
        .stream_request_with_options(stream_options(asked))
        // The microphone is downmixed to one channel of signed 16-bit samples
        // before it gets here, at whatever rate the device runs at. One channel
        // because this is one person dictating: sending two would have Deepgram
        // treat them as separate speakers and charge for both.
        .encoding(Encoding::Linear16)
        .sample_rate(asked.sample_rate)
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
    asked: &Asked,
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

        match connect(client, asked).await {
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
            //
            // The count for the pill's chip is read off the same string in the
            // same breath (AC-36, record 0002's twentieth amendment). It is
            // therefore a count of exactly what reached the cursor, and it is
            // the same string `save_dictation` writes, so the running figure on
            // the chip and the final figure on record 0007's history row are
            // two readings of one string and cannot disagree. `chars().count()`
            // is the call `vocabulary::rules` counts with, so this project has
            // one meaning of "a character". A lock this side could not take
            // sends no count rather than a wrong one, and the chip keeps the
            // last figure it had.
            let mut characters = None;
            if let Ok(mut typed) = typed.lock() {
                typed.push_str(&to_type);
                characters = Some(typed.chars().count());
            }
            // Broadcast rather than sent to the pill alone. Record 0002's
            // clearing table gives a spent allowance one proof that it is over,
            // the first finalised words, and the window holding that message
            // has to receive them.
            let _ = app.emit(
                "dictation:text",
                json!({ "text": text, "characters": characters }),
            );
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

    /* ---- What is actually asked of Deepgram ---------------------------
     *
     * Records 0005 and 0006 both come down to one query string, and these
     * tests read it rather than reading the code that builds it. The seam is
     * `stream_options`, and the assertion goes through the SDK's own
     * `Options::urlencoded`, so what is checked is the text that leaves this
     * machine and not this file's intentions about it.
     */

    /// This file's own source, minus its tests, flattened so a guard survives
    /// a formatter moving a call across lines. The same helper, for the same
    /// reason, as the ones in `settings.rs`, `lib.rs` and the two new features.
    fn this_file_flattened() -> String {
        include_str!("transcribe.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    /// An `Asked` for a test, so each one below says only what it is about.
    fn asked(language: &'static str, keyterms: &[&str]) -> Asked {
        Asked {
            language,
            keyterms: keyterms.iter().map(|t| t.to_string()).collect(),
            sample_rate: 48_000,
        }
    }

    /// The query `stream_options` produces, exactly as Deepgram receives it.
    fn query(asked: &Asked) -> String {
        stream_options(asked)
            .urlencoded()
            .expect("the options always serialise")
    }

    #[test]
    fn an_account_with_no_words_asks_for_exactly_what_it_did_before() {
        // covers: record 0005 AC-10, and record 0006 AC-1's default. This is
        // the whole of "dictation works exactly as it did before this feature
        // existed", and it is the one test that would catch either feature
        // changing the request for somebody who has not used it. English,
        // nova-3, punctuation, and no keyterm parameter at all.
        assert_eq!(
            query(&asked("en", &[])),
            "model=nova-3&language=en&punctuate=true"
        );
    }

    #[test]
    fn the_chosen_language_is_what_is_asked_for() {
        // covers: record 0006 AC-3, AC-5. The Rust half of "the words come out
        // in that language": Deepgram is told which one. That the words then
        // come back right is Deepgram's to do and /check verify's to prove.
        // `multi` is included because AC-5 is about it and because it is the
        // one entry that is not a language.
        for language in ["ja", "pl", "zh-HK", "multi", "he"] {
            let q = query(&asked(language, &[]));
            assert!(
                q.contains(&format!("language={language}")),
                "the query {q:?} does not ask for {language}"
            );
        }
    }

    #[test]
    fn the_custom_vocabulary_is_asked_for_as_keyterm() {
        // covers: record 0005 AC-3. One `keyterm` per word, in the order the
        // list came in, which is the order the screen shows.
        assert_eq!(
            query(&asked("en", &["Fumnanya", "EchoScribe"])),
            "model=nova-3&language=en&punctuate=true&keyterm=Fumnanya&keyterm=EchoScribe"
        );
    }

    #[test]
    fn a_word_holding_an_ampersand_cannot_add_a_parameter_of_its_own() {
        // covers: record 0005 AC-14, and its risk section's first threat. A
        // custom word goes into the query of the streaming address, so a word
        // reading `&language=de` reaching it unencoded would change what is
        // asked of Deepgram, silently, on every dictation. This is the test
        // that says it cannot: the language asked for is still English, and the
        // word arrives as one encoded value.
        let q = query(&asked("en", &["a&language=de"]));
        assert_eq!(
            q,
            "model=nova-3&language=en&punctuate=true&keyterm=a%26language%3Dde"
        );
        // Said the other way round, because the assertion above would also
        // pass if the encoding changed shape: there is exactly one language in
        // this request and it is the one that was chosen.
        assert_eq!(q.matches("language=").count(), 1);
        assert!(!q.contains("language=de"));
    }

    #[test]
    fn a_word_with_a_space_in_it_stays_one_word() {
        // covers: record 0005 AC-3. A phrase is the point of allowing spaces,
        // and a phrase split into two keyterms would boost two ordinary words
        // instead of one name. Deepgram's documented form for this is a plus
        // sign inside one value.
        let q = query(&asked("en", &["Fumnanya Nketa"]));
        assert!(
            q.ends_with("keyterm=Fumnanya+Nketa"),
            "the query {q:?} does not carry the phrase as one keyterm"
        );
        assert_eq!(q.matches("keyterm=").count(), 1);
    }

    #[test]
    fn the_vocabulary_and_a_chosen_language_are_asked_for_together() {
        // covers: record 0006 AC-12, "the words I have added still apply in
        // the language I picked". Deepgram documents keyterm as working for
        // nova-3 monolingual and multilingual alike, and this is the half this
        // project controls: both parameters are on the same request, neither
        // dropping the other. Whether Deepgram then honours them in Japanese
        // is /check verify's, and both records name it as unproven.
        let q = query(&asked("ja", &["Nketa"]));
        assert_eq!(q, "model=nova-3&language=ja&punctuate=true&keyterm=Nketa");
        let q = query(&asked("multi", &["EchoScribe", "Fumnanya"]));
        assert!(q.contains("language=multi"));
        assert_eq!(q.matches("keyterm=").count(), 2);
    }

    #[test]
    fn a_word_in_a_non_latin_script_survives_the_request_whole() {
        // covers: record 0005 AC-3, AC-14, and record 0006 AC-12. The budget
        // is counted in characters precisely so that a Japanese or Arabic term
        // is a legitimate one, so a term in one has to arrive intact. Encoded
        // rather than mangled, and one value rather than several.
        let q = query(&asked("ja", &["エコー"]));
        assert!(
            q.ends_with("keyterm=%E3%82%A8%E3%82%B3%E3%83%BC"),
            "the query {q:?} does not carry the Japanese term intact"
        );
        assert_eq!(q.matches("keyterm=").count(), 1);
    }

    #[test]
    fn punctuation_is_asked_for_in_every_language() {
        // covers: record 0006 AC-3. Record 0006 says punctuation stays on for
        // every language, on Deepgram's documentation alone, and its Still open
        // records that what happens for a language that does not support it is
        // undocumented. This test does not settle that. It settles the half
        // this file owns: that no language turns punctuation off here, which is
        // the change somebody would reach for if a language ever refused it,
        // and which the record says must be an amendment rather than a quick
        // edit.
        for language in ["en", "ja", "ar", "multi"] {
            assert!(
                query(&asked(language, &[])).contains("punctuate=true"),
                "punctuation is not asked for in {language}"
            );
        }
    }

    #[test]
    fn what_is_asked_of_deepgram_is_decided_in_exactly_one_place() {
        // covers: record 0005 AC-3 and record 0006 AC-4, as a source guard.
        // The first connection and the one reconnect attempt must ask for the
        // same thing: a reconnect that sent a different language would change
        // language mid dictation, and one that dropped the keyterms would
        // quietly stop using a person's words halfway through a sentence. Both
        // go through `connect`, which goes through `stream_options`, and this
        // is what keeps it that way. The behaviour needs a real socket dying
        // mid dictation, which is /check verify's; this only stops a second
        // options builder appearing.
        let flat = this_file_flattened();
        assert_eq!(
            flat.matches("Options::builder()").count(),
            1,
            "transcribe.rs now builds Deepgram options in more than one place. \
             The first connection and the reconnect would then be able to ask \
             for different things (record 0005 AC-3, record 0006 AC-4)"
        );
        assert_eq!(
            flat.matches("stream_options(asked)").count(),
            1,
            "connect no longer takes its options from stream_options"
        );
        // And the reconnect asks with the same `asked` it was handed, rather
        // than reading either feature again.
        assert!(
            flat.contains("connect(client,asked).await"),
            "the reconnect no longer passes the same `asked` the first \
             connection used"
        );
        for feature in ["crate::vocabulary", "crate::language"] {
            assert!(
                !flat.contains(feature),
                "transcribe.rs now reads {feature} itself. Both are read once \
                 in mod.rs before the microphone opens, and reading either \
                 here would let a change reach a dictation already running"
            );
        }
    }

    #[test]
    fn neither_the_language_nor_a_word_is_ever_logged() {
        // covers: record 0005 AC-13, and record 0006's risk section. A person's
        // custom vocabulary is personal data by record 0005's own reckoning,
        // and a log is a file. `Asked`'s own doc comment promises this; this is
        // the test that keeps the promise.
        let flat = this_file_flattened();
        for forbidden in ["{asked", "{language}", "keyterms:?", "asked:?"] {
            assert!(
                !flat.contains(forbidden),
                "transcribe.rs now puts {forbidden} in a log line, which would \
                 write a person's own words or their language to a file"
            );
        }
    }

    #[test]
    fn the_pill_lays_transcribed_words_out_in_their_own_direction() {
        // covers: record 0006 AC-11's pill half. A line of Arabic or Hebrew in
        // a container that declares left to right is shown backwards, and the
        // whole mechanism is one attribute on the transcript line and its two
        // halves. It is a source guard because the rendering itself needs a
        // running app and a person who can read the script; what this catches
        // is the attribute being tidied away as decoration, which is exactly
        // what it looks like.
        //
        // Same shape as the guards in mod.rs and error_screen.rs, which read
        // src/main.js for the same reason.
        const PILL: &str = include_str!("../../../src/dictate/pill.html");
        for element in ["pill__line\"", "pill__final\"", "pill__interim\""] {
            let at = PILL
                .find(element)
                .unwrap_or_else(|| panic!("the pill no longer has a {element}"));
            let tag_end = PILL[at..]
                .find('>')
                .expect("an element always closes its tag");
            assert!(
                PILL[at..at + tag_end].contains("dir=\"auto\""),
                "the pill's {element} no longer carries dir=\"auto\", so a line \
                 of Arabic or Hebrew would be shown backwards (record 0006 \
                 AC-11)"
            );
        }
        // And no list of right to left languages appeared beside it, which
        // record 0006 and design/registry.md both forbid: a direction guessed
        // from the setting is wrong the moment a Hebrew speaker dictates an
        // English product name.
        for hint in ["dir=\"rtl\"", "rtl-languages", "RTL_LANGUAGES"] {
            assert!(
                !PILL.contains(hint),
                "the pill now holds {hint}. dir=\"auto\" is the whole mechanism"
            );
        }
    }
}
