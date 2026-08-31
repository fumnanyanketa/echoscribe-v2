# EchoScribe v2 · plan

**Build approach:** Tracer bullet. EchoScribe is mostly plumbing — a
hotkey, a microphone, an outside transcription service, and typing text
into whatever app you're using. The scary risk is those pieces not
connecting, so phase 1 proves the whole pipe works end to end, for real,
before any one piece is made great.

**Last reconciled:** 2026-08-24

## At a glance

| # | Feature | Phase | Weight | Status |
|---|---|---|---|---|
| 1 | Sign in | 1 | heavy | in progress |
| 2 | Dictate with a hotkey | 1 | heavy | in progress |
| 3 | Teach it your words | 2 | medium | planned |
| 4 | Speak in your language | 2 | medium | planned |
| 5 | See what you've said before | 2 | medium | planned |

---

## Phase 1: the core promise works, end to end

Sign in, press a hotkey anywhere on the machine, speak, and watch the
words type themselves out wherever your cursor was. This alone is
something you could use daily.

### 1. Sign in · in progress · heavy

You need an account so your settings and words belong to you, and so
what you say can be tied back to you.

Done when: I can make an account, close the app, reopen it, and I'm
still signed in.

- [x] Design it: [0003-sign-in](decisions/0003-sign-in.md), 18 acceptance criteria
- [x] The sign-in screen and the browser handoff · `src-tauri/src/sign_in/`, `src/sign-in/`
- [x] Staying signed in, and signing out · `src-tauri/src/sign_in/renewal.rs`
- [x] Offline, and a second account on the machine · `src-tauri/src/sign_in/clerk.rs`, `src/sign-in/`

### 2. Dictate with a hotkey · in progress · heavy

The whole point of EchoScribe. Press a hotkey, speak, and the text
appears wherever your cursor is, in any app you're typing into. Marked
heavy because your voice leaves the machine to an outside transcription
service — that's personal data, even though the brief only flagged
sign-in by name.

Done when: I press the hotkey, speak a sentence, and it appears as
typed text exactly where my cursor was, in any app.

- [x] Design it: [0002-dictate-with-a-hotkey](decisions/0002-dictate-with-a-hotkey.md), 33 acceptance criteria
- [x] The hotkey and the pill · `src-tauri/src/dictate/`, `src/dictate/` · built, awaiting `/check verify` (live two-monitor, focus and sound proof). AC-27's focus fix is proved: re-verified live 2026-08-30, evidence `docs/evidence/dictate-with-a-hotkey/AC-27-pill-no-longer-takes-focus.md`. The sound half is not: the fallback failed live, was fixed in `661ce2f`, and the eighth amendment, on 2026-08-30, added one more piece of `/develop` work to step 1a before it is re-run, a check that the sound file a scheme entry names is actually there
- [x] The microphone and the waveform · `src-tauri/src/dictate/microphone.rs`, `src-tauri/src/dictate/limits.rs`, `src/dictate/` · built, awaiting `/check verify`. AC-8's 30 second silence cap is built and deliberately unarmed until milestone 4 arms it. See `docs/evidence/dictate-with-a-hotkey/milestone-2-decisions-owed.md`
- [x] Somewhere to read a microphone error · step 2a in the record · AC-28, AC-29, AC-31. `src-tauri/src/dictate/mod.rs`, `src-tauri/src/dictate/microphone.rs`, `src/dictate/mic-error.js` · built 2026-08-30, awaiting `/check verify` (the live proof of all four error kinds; AC-30's mid-dictation half waits for milestone 4). The design half landed first: registry gained the six "Microphone error" rows and the "Error pill" correction
- [x] Telling blocked-by-Windows apart for real · step 2b in the record · AC-15, AC-29. `src-tauri/src/dictate/consent.rs`, `src-tauri/src/dictate/microphone.rs` · built 2026-08-30, awaiting `/check verify` (step 2a's live proof covers it: machine-wide toggle off, then desktop-apps toggle off, each must read `MICROPHONE_BLOCKED_BY_WINDOWS` with Open Windows settings). Reads the three Windows consent switches, read only, when the failure has no named cause. Settled by the fifth amendment of 2026-08-30
- [x] The Deepgram key · milestone 3 in the record · AC-9, AC-10, AC-11, part of AC-12, AC-13's wording. `src-tauri/src/dictate/deepgram_key.rs`, `src-tauri/src/dictate/key_vault.rs`, `src-tauri/src/dictate/store.rs`, `src/dictate/key-setup.js` · built 2026-08-30, awaiting `/check verify` (a real key accepted and dictation starting, a wrong key rejected with nothing saved, and the key surviving a restart). The five decisions the gate found owed were ratified into the record by the sixth amendment, on 2026-08-30: the get-a-key address, what request checks a key, the response-to-cause mapping, the four sentences, and AC-13's two next steps. That amendment also settled what an error screen shows once the cause is fixed, which is the new AC-32. AC-12's Settings display and AC-13's live trigger are milestones 5 and 4
- [x] Clearing an error the person has already fixed · step 3a in the record · AC-32. `src-tauri/src/dictate/pill_window.rs`, `src-tauri/src/dictate/mod.rs`, `src/main.js` · built 2026-08-30, awaiting `/check verify` (switch microphone access off in Windows, press the hotkey, read the error, switch access back on without touching EchoScribe, press the hotkey again: the words land and the window behind them no longer shows the error). `dictation:opened` is now broadcast rather than sent to the pill alone, and the shell leaves the microphone error screen on it, quietly. The three key errors already cleared on a key being accepted and saved, which milestone 3 built. The allowance kind clears on the first finalised words and belongs to milestone 4; three source guards in `mod.rs` keep it out of the shell. Settled by that same sixth amendment
- [ ] Transcription and typing · milestone 4 in the record · AC-3, AC-4, AC-7, AC-8's silence half, AC-20. `src-tauri/src/dictate/transcribe.rs`, `src-tauri/src/dictate/typing.rs`, `src-tauri/src/dictate/microphone.rs`, `src-tauri/src/dictate/limits.rs`, `src-tauri/src/dictate/deepgram_key.rs`, `src-tauri/src/dictate/mod.rs` · **the streaming and typing half is built, 2026-08-30; the half a person reads when it goes wrong is not, and is blocked on `/canvas`.** Built: streaming to Deepgram on `nova-3`, typing finalised phrases at the focused cursor as character-only keystrokes, the password field refusal, the one reconnect attempt holding five seconds of audio, the silence cap armed, and every Deepgram failure classified into a kind, code, sentence and action. Not built, because `design/registry.md` draws none of it and record 0002 forbids inventing it: the pill's words for a mid dictation failure, the pill's `MIC STOPPED`, the microphone catch-all's second sentence, and the EchoScribe window's mid dictation Deepgram error screen. The registry's own "Error pill" row is part of the problem, not the answer: it keeps the working pill's 232x44 and the decided sentences do not fit. Six decisions the gate found owed were answered by the user on 2026-08-30, written up in `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`, and carried into the record by the ninth amendment on 2026-08-31, which added AC-33 for the pill's grey interim line and settled the typing cure the Notepad finding forced: keystrokes stay the default, the pacing is ratified as a mitigation, and receivers proven to collapse injected keystrokes get a direct character channel, step 4a in the record, spike first. Two spikes the record asks for are still owed and both need the user: a deliberately narrow key pointed at the stream, and how widely `IsPassword` really reaches
- [ ] Transcription and typing, the part a person reads · `/canvas` first, then `/develop` · the four undrawn states above, plus AC-13's and AC-14's window action and AC-30's mid dictation half. Nothing here is a new decision: the kinds, the codes, the sentences and the actions are all fixed and already ride on `dictation:error`
- [ ] History and settings

---

## Phase 2: make it more accurate and more yours

The three things confirmed as being built, not cut, on top of the
working core loop. Order among these three is a judgment call, not a
dependency — any could move first.

### 3. Teach it your words · needs a decision · medium

Accuracy is the one thing EchoScribe has to get right. This lets you
fix words or names it keeps getting wrong.

Done when: I add a word or name it mis-transcribes, and the next time I
say it, it comes out right.

- [ ] Design it: /architect teach it your words

### 4. Speak in your language · needs a decision · medium

Done when: I pick a language other than English, speak, and the text
comes out correctly in that language.

- [ ] Design it: /architect speak in your language

### 5. See what you've said before · needs a decision · medium

Done when: I open my history and see the text from something I
dictated earlier, even after closing and reopening the app.

- [ ] Design it: /architect see what you've said before

---

## Out of scope (v2)

Not building: payments/billing, a mobile version, cloud sync across
devices, sharing/collaboration, notifications, voice commands or
voice-driven editing, an on-device transcription model (a service is
used instead), real-time captions for calls or meetings.

Full reasoning: [docs/brief.md](brief.md).
