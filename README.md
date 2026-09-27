# EchoScribe

**Speak, and it types. A desktop dictation tool that turns your voice into text wherever your cursor is, triggered by a hotkey.**

**Status:** in active development — see [CHANGELOG.md](CHANGELOG.md) for exactly what works and what doesn't yet
**Platform:** Windows first, macOS planned
**Designed and built by:** Fumnanya Nketa, using AI-assisted development

---

## Why it exists

Typing breaks your flow. When you speak, you say more and get more of what's in your head out — but most dictation tools either live inside one app, or wait until you stop talking before anything appears.

EchoScribe is for someone mid-way through an email or document who wants to switch to speaking for a stretch, or anyone who can't or doesn't want to type right now. The one thing it must do well is **accuracy**: fixing its mistakes should never take longer than typing would have.

It's version 2 of [EchoScribe](https://github.com/fumnanyanketa/echoscribe), which ran in the browser. This version is a native desktop app, so it can type into any application.

## What it does

- **Dictate anywhere with a hotkey.** A small floating pill shows a live waveform while you speak, and words appear as you talk, not after you stop
- **Sign in** through your browser, the same way desktop apps like GitHub Desktop do
- **Teach it your words.** Add the names and terms it keeps getting wrong, and they're sent with every dictation so they're recognised
- **Speak in your language.** Choose from the languages the transcription model supports
- **See what you've said before.** A local history of past dictations

## Architecture

```
┌─────────────────────────────── Your computer ───────────────────────────────┐
│                                                                             │
│   Web interface (HTML, CSS, JS)          Rust core (Tauri)                   │
│   • floating pill + waveform  ◄──────►   • global hotkey                     │
│   • dashboard + settings                 • microphone capture                │
│                                          • types text into the active app    │
│                                          • SQLite: history, words, settings  │
│                                          • Windows Credential Manager: keys  │
└──────────────────────────────┬─────────────────────────────┬────────────────┘
                               │                             │
                               ▼                             ▼
                     Clerk (sign-in, OAuth)         Deepgram (live streaming
                                                    transcription)
```

There is no server of its own. Everything except sign-in and transcription stays on your machine.

## Key design decisions

Every significant decision is written up as an architecture decision record in [`docs/decisions/`](docs/decisions/), including the options that were rejected and why.

**Tauri, not Electron or native Windows.** Electron would carry the web interface over to Mac too, but bundles a full browser — 150–300 MB and more memory, for an app that runs in the background all day. A native Windows build would be the smallest and snappiest, but its interface wouldn't carry over to Mac at all. Tauri gives a Rust core with a web interface: light, and portable.

**No backend server.** With no cross-device sync in scope, the app talks to Clerk and Deepgram directly and keeps everything else in one local SQLite file. That removes an entire layer to build, host and pay for.

**Deepgram for transcription**, because live streaming is the whole feel of the product. It was the only option researched with confirmed streaming delay under 200 ms, the cheapest per minute, and the most generous free credit. The trade-off is narrower language coverage than Google's speech service.

**Each person brings their own Deepgram key.** A shared key would mean one account paying for every user's transcription, with no billing to recover the cost. Instead each person uses their own free Deepgram allowance, at the price of one extra setup step.

**Keys never touch a file.** The Deepgram key is stored in the Windows Credential Manager, not in the database or a config file.

## How it's built

Development follows a written process: each feature gets a decision record with numbered acceptance criteria, then a build, then a verification run against those criteria, with the evidence saved in [`docs/evidence/`](docs/evidence/). There's a full code review and a security sweep in [`docs/reviews/`](docs/reviews/) — dependency audit and the full test suite included.

## Tech stack

| Layer | Technology |
|---|---|
| App framework | Tauri 2 |
| Core | Rust |
| Interface | Plain HTML, CSS and JavaScript, no framework |
| Local storage | SQLite |
| Secrets | Windows Credential Manager |
| Sign-in | Clerk (OAuth, via the system browser) |
| Transcription | Deepgram, live streaming |

## Running locally

Requires Rust, Node.js and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for Windows.

```bash
npm install
npm run tauri dev

cd src-tauri && cargo test    # run the Rust test suite
```

You'll need your own [Deepgram](https://deepgram.com) API key, entered in the app's settings on first run.
