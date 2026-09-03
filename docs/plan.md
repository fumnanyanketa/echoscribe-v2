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
- [x] The hotkey and the pill · `src-tauri/src/dictate/`, `src/dictate/` · built, awaiting `/check verify` (live two-monitor, focus and sound proof). AC-27's focus fix is proved: re-verified live 2026-08-30, evidence `docs/evidence/dictate-with-a-hotkey/AC-27-pill-no-longer-takes-focus.md`. The sound half is not: the fallback failed live, was fixed in `661ce2f`, and the eighth amendment, on 2026-08-30, added one more piece of `/develop` work to step 1a before it is re-run, a check that the sound file a scheme entry names is actually there. That check was built 2026-08-31 in `src-tauri/src/dictate/sound.rs`, unit tested, and step 1a's sound re-verify is now unblocked
- [x] The microphone and the waveform · `src-tauri/src/dictate/microphone.rs`, `src-tauri/src/dictate/limits.rs`, `src/dictate/` · built, awaiting `/check verify`. AC-8's 30 second silence cap is built and deliberately unarmed until milestone 4 arms it. See `docs/evidence/dictate-with-a-hotkey/milestone-2-decisions-owed.md`
- [x] Somewhere to read a microphone error · step 2a in the record · AC-28, AC-29, AC-31. `src-tauri/src/dictate/mod.rs`, `src-tauri/src/dictate/microphone.rs`, `src/dictate/mic-error.js` · built 2026-08-30, awaiting `/check verify` (the live proof of all four error kinds; AC-30's mid-dictation half waits for milestone 4). The design half landed first: registry gained the six "Microphone error" rows and the "Error pill" correction
- [x] Telling blocked-by-Windows apart for real · step 2b in the record · AC-15, AC-29. `src-tauri/src/dictate/consent.rs`, `src-tauri/src/dictate/microphone.rs` · built 2026-08-30, awaiting `/check verify` (step 2a's live proof covers it: machine-wide toggle off, then desktop-apps toggle off, each must read `MICROPHONE_BLOCKED_BY_WINDOWS` with Open Windows settings). Reads the three Windows consent switches, read only, when the failure has no named cause. Settled by the fifth amendment of 2026-08-30
- [x] The Deepgram key · milestone 3 in the record · AC-9, AC-10, AC-11, part of AC-12, AC-13's wording. `src-tauri/src/dictate/deepgram_key.rs`, `src-tauri/src/dictate/key_vault.rs`, `src-tauri/src/dictate/store.rs`, `src/dictate/key-setup.js` · built 2026-08-30, awaiting `/check verify` (a real key accepted and dictation starting, a wrong key rejected with nothing saved, and the key surviving a restart). The five decisions the gate found owed were ratified into the record by the sixth amendment, on 2026-08-30: the get-a-key address, what request checks a key, the response-to-cause mapping, the four sentences, and AC-13's two next steps. That amendment also settled what an error screen shows once the cause is fixed, which is the new AC-32. AC-12's Settings display and AC-13's live trigger are milestones 5 and 4
- [x] Clearing an error the person has already fixed · step 3a in the record · AC-32. `src-tauri/src/dictate/pill_window.rs`, `src-tauri/src/dictate/mod.rs`, `src/main.js` · built 2026-08-30, awaiting `/check verify` (switch microphone access off in Windows, press the hotkey, read the error, switch access back on without touching EchoScribe, press the hotkey again: the words land and the window behind them no longer shows the error). `dictation:opened` is now broadcast rather than sent to the pill alone, and the shell leaves the microphone error screen on it, quietly. The three key errors already cleared on a key being accepted and saved, which milestone 3 built. The allowance kind clears on the first finalised words and belongs to milestone 4; three source guards in `mod.rs` keep it out of the shell. Settled by that same sixth amendment
- [x] Transcription and typing · milestone 4 in the record · AC-3, AC-4, AC-7, AC-8's silence half, AC-20. `src-tauri/src/dictate/transcribe.rs`, `src-tauri/src/dictate/typing.rs`, `src-tauri/src/dictate/microphone.rs`, `src-tauri/src/dictate/limits.rs`, `src-tauri/src/dictate/deepgram_key.rs`, `src-tauri/src/dictate/mod.rs` · the streaming and typing half built 2026-08-30, the readable half 2026-08-31 (the row below), both awaiting `/check verify`. Built: streaming to Deepgram on `nova-3`, typing finalised phrases at the focused cursor as character-only keystrokes, the password field refusal, the one reconnect attempt holding five seconds of audio, the silence cap armed, and every Deepgram failure classified into a kind, code, sentence and action. Six decisions the gate found owed were answered by the user on 2026-08-30, written up in `docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`, and carried into the record by the ninth amendment on 2026-08-31, which added AC-33 for the pill's grey interim line and settled the typing cure the Notepad finding forced: keystrokes stay the default, the pacing is ratified as a mitigation, and receivers proven to collapse injected keystrokes get a direct character channel, step 4a in the record, spike first. Two spikes the record asks for are still owed and both need the user: a deliberately narrow key pointed at the stream, and how widely `IsPassword` really reaches
- [x] Transcription and typing, the part a person reads · built 2026-08-31 by `/develop`, after `/canvas` drew the states and the twelfth amendment settled the pill's one geometry (472x52, sized once at open, never resized, the counter band reserved in the footprint) · AC-20's showing half, AC-30's mid dictation half, AC-33, AC-13's and AC-14's window action. `src/dictate/pill.html`, `pill.js`, `pill.css` (the transcript line with the grey interim tail, the before-words state, the error pill, MIC STOPPED), `src-tauri/src/dictate/pill_window.rs` and `pill_mouse.rs` (the geometry and its tests), `src-tauri/src/dictate/mod.rs` (the eleventh amendment's 2 second hold, the device-death ending, the window brought forward on Deepgram endings and never on the password refusal), `src-tauri/src/dictate/microphone.rs` (the stream death report and the catch-all's second sentence), `src/main.js`, `src/dictate/deepgram-error.js` and `.css` (the mid dictation Deepgram error screen, clearing on the first finalised words), `src/dictate/key-setup.js` (a mid dictation rejected key lands there with its error line). Awaiting `/check verify`; any change to the pill's geometry re-opens the AC-27 live click proof, which rides in that pass. The elapsed and word count chip is deliberately not built: its band is reserved, and its two values have no source named in the record yet
- [ ] The pill's elapsed and word count chip · needs its sources named (which clock, which words) before `/develop` builds it into the reserved band · design/registry.md draws it; no acceptance criterion requires it
- [x] History and settings, the half that needs no screen · milestone 5 in the record · AC-17, AC-18, and the Rust side of AC-12, AC-19, AC-21 and AC-22. `src-tauri/src/dictate/store.rs` (`save_dictation`, `save_hotkey`, `save_sounds_enabled`, the strict `Hotkey::from_chosen` beside the lenient `from_stored`, and 13 new tests), `src-tauri/src/dictate/transcribe.rs` (the finalised phrases kept in memory as they are typed and handed over at the close; the old `typed_something` flag is gone, the same string now decides the joining space and fills the row), `src-tauri/src/dictate/mod.rs` (`Live` carries the account and the start time, one `record` function on every one of the three close paths), `src-tauri/src/dictate/settings.rs` (new: `get_hotkey`, `set_hotkey`, `get_dictation_sounds`, `set_dictation_sounds`, so the record's twelve commands all exist), `src-tauri/src/lib.rs`. Built 2026-09-02, awaiting the user's own drive and then `/check verify`. The gate found AC-17's word "completed" had no named source and the user settled it the same day, carried into the record by the thirteenth amendment: a row when, and only when, at least one finalised phrase reached the cursor, whatever ended the dictation
- [ ] The app shell, the window the settings screen lives in · Design it: [0004-the-app-shell](decisions/0004-the-app-shell.md), 7 acceptance criteria · a prerequisite of the row below, and it serves rows 1 to 5 rather than belonging to any of them. Settled 2026-09-02, after `/canvas` handed four questions to `/architect`: a second window for the 1200x800 dashboard alongside today's 760x540 dark one, resizable with a 960x640 floor, remembering its size and place per account; the small window shown only for a pre-shell state or an interruption, hiding to reveal the dashboard when an error clears itself, which is how AC-32 reads with two windows; the rail holding only sections that work, so Settings alone today with Dictation and Transcription beneath it and nothing greyed out; and the landing on Settings, Dictation, which plan row 5's record moves to History when it exists. Three milestones: the two windows, the remembered geometry, then the rail and the surface. **Not started. It was blocked on two small things on 2026-09-03**, when `/develop`'s gate stopped before writing any code: the rail's account block takes over the Sign out from `mountSignedIn`, which record 0004 deletes, and that same stub is the only place record 0003's AC-14 "Working offline" sign exists, with nothing drawn for it anywhere in `design/registry.md`. The user chose on 2026-09-03 to have it drawn and recorded before the build rather than flagged after, so `/canvas` owes one registry row and `/architect` owes one amendment. The gate also answered the record's first "Still open" item off the generated schema, that Tauri grants no per-command permission for an application command, and found that the account block's four values travel on the existing `get_auth_state`, a third command the Interface surface does not name. All of it, with what each skill owes, is in [docs/evidence/the-app-shell/gate-2026-09-03-blocked.md](evidence/the-app-shell/gate-2026-09-03-blocked.md) · **Unblocked 2026-09-03.** `/canvas` drew the `Offline row` and `/architect` amended both records, so the build starts at milestone 1. No criterion was added and the count stays at 7: the offline sign lives in the account block, record 0003's AC-14 stays record 0003's and milestone 3 proves it, `get_auth_state` is named as the third command the dashboard calls, and `dashboard.json` grants `pill.json`'s two event permissions and nothing else, the user's choice on 2026-09-03, because the offline row has to arrive and go while the dashboard is open · **All three milestones built 2026-09-03**, awaiting `/check verify` and `/test`. `src-tauri/src/shell/` (new: `mod.rs` decides which of the two windows is at rest, `dashboard_window.rs` creates, places and closes the dashboard, `geometry.rs` holds the clamp and the screen match with 12 tests, `store.rs` holds the `shell_window` table with 8 tests, `rail.rs` holds the fixed rail with 4 tests), `src-tauri/capabilities/dashboard.json` (the two event permissions and nothing else), `src-tauri/tauri.conf.json` (the small window starts hidden, so Rust decides which window a person sees first), `src-tauri/src/lib.rs`, `src-tauri/src/dictate/deepgram_key.rs` (two new signals, `dictation:key_saved` and `dictation:key_cleared`, so the shell learns the moment a key changes the invariant), `src/shell/` (new: `dashboard.html`, `dashboard.js`, `rail.js`, `account-block.js`, `dashboard.css`), `src/main.js` (the small window's router, now routing a signed-in person with no key to the key setup screen and drawing nothing when the dashboard has the screen), `src/sign-in/sign-in.js` and `.css` (`mountSignedIn` and every `.signed-in` rule deleted in the same change, per standing rule 11), `src/shell.css` (the stale placeholder comment). Proved live the same day: the dashboard opens at exactly 1200x800 centred in the working area with the small window hidden behind it, the rail draws Settings with Dictation active and Transcription resting, the account block draws the initials, name, mail address, date and Sign out, the `OFFLINE` row draws above them, and a move and resize survived a restart to the pixel. One bug was found and fixed in that sitting: the offline row never cleared, because the session's first refresh emits `auth:signed_in` while the dashboard page is still loading and the listener was registered after it; both listeners now go up before anything is asked of Rust. **Three things are reported not done rather than worked around**: AC-5's unplug half and AC-4's second-screen half need two monitors of different sizes and this machine has one; AC-2 is only half answerable until record 0002's milestone 5 lands, because the rail's items go active but the white surface behind them is empty and `design/registry.md` draws no component for a surface waiting for its screen; and AC-3's 960x640 floor needs a real mouse drag on a window edge, which `SetWindowPos` cannot stand in for
- [ ] The settings screen · the visible half of AC-12, AC-19, AC-21 and AC-22 · no longer blocked on `/canvas`, which drew every component it needs on 2026-09-02: `Hotkey choice`, `Sound switch` and `Setting error line`. Blocked instead on the shell above, because until that exists there is no window the screen can live in and no route a person could take to reach it. Record 0002's fourteenth amendment settled the two error sentences the screen says, `SETTING_NOT_SAVED` on a refused write and `SETTINGS_NOT_READ` on a failed read. The four commands it will call are already there

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
