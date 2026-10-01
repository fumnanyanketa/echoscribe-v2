# Verify: the whole dictation tool, end to end, 2026-10-01

Approach bar: tracer bullet, from the header of `docs/plan.md`. The question
this run answers is the one the user asked: is the whole pipe sound, and is
this a working dictation tool. It is not a per-record sweep of all 96
acceptance criteria, so no plan verify box is ticked by this run.

Verdict on the question asked: **PASS, with one real defect found, fixed and
re-proved in the same sitting.**

How this run spoke: this machine's text-to-speech played through the
speakers, and the app's real microphone heard it. Nothing was mocked. The
hotkey was a real double tap of left Ctrl injected at the keyboard level,
the receiver was a real Win32 edit control in another process, and every
database claim below was read from the one real
`%APPDATA%\com.echoscribe.app\echoscribe.sqlite3`.

## What was proved, with the evidence

| Behaviour | Outcome | Evidence |
|---|---|---|
| Launch lands signed in after 17 days away (record 0003's done-when) | met | `local-only/signed-in-history-after-17-days.png`: account block "Signed in since September 4, 2026", session row `last_verified_at` updated |
| History screen shows real past rows at launch (row 5, and the two-database fix live) | met | same screenshot; rows with date, duration, character count, language chip, Copy |
| Double tap Ctrl opens the pill and the microphone | met | window enumeration gained `EchoScribe dictation` 1008x236; `local-only/pill-live-transcript-waveform.png` shows the waveform moving and the transcript line filling |
| Spoken words land as typed text at the cursor in another app | met | `landed-text-dictated-by-tts.txt`: the scripted English sentence landed character-perfect |
| Second double tap ends the dictation and a history row is written (AC-17) | met | dictation row id 7, 40s, `en`, text matches landed text byte for byte |
| Language: pick German on the Languages screen, next dictation is German (row 4) | met | `transcription_language` row flipped to `de`; the German paragraph landed character-perfect with umlauts; row id 8 carries `de`; then set back to English, row flipped to `en` |
| The pill's elapsed and count chip appears past 20 seconds (AC-36) | met | `pill-german-with-elapsed-count-chip.png`: chip reads `0:22 · 229 characters`, right-aligned in the reserved band |
| Vocabulary: add a term, it is stored for the account and shapes the next dictation (row 3) | met | `vocabulary_term` row `EchoScribe` for the account; next dictation landed "EchoScribe" as one word, exact casing, where untaught speech would get two words |
| Everything survives a restart: session, history, language, vocabulary | met | `local-only/history-after-restart-three-new-rows.png` after a clean exit and relaunch |
| Closing the dashboard with its X exits the whole app (record 0004 AC-6) | met | process gone within 3 seconds of the click, dev runner exited 0 |
| Dictating into a password field types nothing (AC-20, data rule) | met | `password-box-empty-after-dictation.png`: a full sentence spoken while a real password edit held focus, box empty, and no history row was written for it |
| Exactly one dashboard window exists after launch | **not met, then fixed, then met** | see the finding below; after the fix, one `Tauri Window` on three consecutive cold starts |

## The one defect found: a ghost second dashboard

On the second launch of the night the process owned **two** identical
top-level `Tauri Window` dashboards stacked at the same rect, one of them
unreachable by any `get_webview_window` for the life of the process.

Cause, read from the source and consistent with the live timing: `settle`
runs on whichever thread calls it. `shell::init` calls it on the main
thread while the sign-in renewal thread's first refresh emits
`auth:signed_in`, whose listener calls it on the renewal thread. Both ask
"is the dashboard open", both hear no, both build. The second build takes
the label in Tauri's window map and the first becomes the ghost. A race,
so most launches look fine: the night's first launch had one window, the
second had two.

This is the family the 2026-09-11 review flagged as its biggest cluster
(timing seams around window creation), landed as a concrete bug.

Fix, applied this sitting in `src-tauri/src/shell/mod.rs`: a `settle_gate:
Mutex<()>` on the Shell state, held for the whole of `settle`, so the
ask-then-create is atomic across threads. A poisoned lock settles anyway.
A source-guard test in the file's own style,
`settle_takes_the_gate_before_asking_whether_the_dashboard_exists`, fails
the build if the gate leaves `settle` or slips below the first ask.

Re-proved: 344 tests pass, clippy clean with `-D warnings`, and three
consecutive cold starts of the fixed binary each showed exactly one
dashboard window.

## Found outside the criteria

- An open microphone hears the room, and the room had a person in it.
  Dictation 1's row begins with ambient speech before the scripted
  sentence. Correct behaviour, but it is why this run's raw proofs are
  split: anything carrying a person's words or account identity lives in
  `local-only/`, which `.gitignore` keeps off the public repository.
- The hotkey needs a beat after launch. A double tap fired seconds after
  the process started did nothing once; the same tap worked after the
  session's first refresh had landed. Consistent with arming on sign-in
  confirmation. Not judged a defect; worth one deliberate look the next
  time record 0002 is swept.
- Across dictations the app types with no joining space: a new dictation
  starting where an old one ended produces "dog.Guten". Within one
  dictation the joining space is correct. The record decides joining
  only within a dictation, so this is unspecified rather than wrong.
- A transcription slip, not a typing fault: the synthetic voice's
  "and it listens" landed as "and listens". Same class as the filed
  "markets" slip in `step-4a-typing-proof.md`.

## Not checked, and why

- Sign out and the browser-handoff sign-in: signing out would destroy the
  one working session, and signing back in needs a person at a browser.
  Blocked, deliberately, on an unattended run.
- The microphone error kinds and the Windows-settings clearing table
  (AC-15, AC-28 to AC-32): they need the OS microphone consent switches
  flipped, which this run does not touch. Old live evidence from
  2026-08-30 stands; nothing re-proved tonight.
- The two dictation sounds: nothing in this rig can hear. The pill and
  the typing were the observed truths.
- Replace and Remove on the Deepgram key: both would mutate the one real
  saved key. Blocked, deliberately.
- Offline behaviour, the second-monitor cases, a real edge-drag on the
  dashboard floor, and the AC-27 click-through proof: same standing as
  before, they need hands or hardware this run does not have.
- The pill's refusal sentence on screen for the password case: the typed
  half is proved above; the shown half kept losing the z-order race to a
  capture this rig could make. The 2026-08-31 frames in
  `docs/evidence/dictate-with-a-hotkey/frames/` remain the filed proof.

## Files this run wrote

- This folder: the report, two clean screenshots, the landed-text record,
  `.gitignore`, and four `local-only/` proofs that stay on this machine.
- `src-tauri/src/shell/mod.rs`: the settle gate, its comment, its test.
- Nothing else in the repository was touched.
