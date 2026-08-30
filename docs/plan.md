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

- [x] Design it: [0002-dictate-with-a-hotkey](decisions/0002-dictate-with-a-hotkey.md), 31 acceptance criteria
- [x] The hotkey and the pill · `src-tauri/src/dictate/`, `src/dictate/` · built, awaiting `/check verify` (live two-monitor, focus and sound proof)
- [x] The microphone and the waveform · `src-tauri/src/dictate/microphone.rs`, `src-tauri/src/dictate/limits.rs`, `src/dictate/` · built, awaiting `/check verify`. AC-8's 30 second silence cap is built and deliberately unarmed until milestone 4 arms it. See `docs/evidence/dictate-with-a-hotkey/milestone-2-decisions-owed.md`
- [ ] Somewhere to read a microphone error · step 2a in the record · AC-28, AC-29, AC-31. `/canvas` first: there is no drawn comp for a microphone error on the EchoScribe window, and the registry's "Error pill" line needs correcting from one action to none. Settled by the fourth amendment of 2026-08-30
- [ ] The Deepgram key
- [ ] Transcription and typing
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
