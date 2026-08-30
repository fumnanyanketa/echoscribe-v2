# Verify: Dictate with a hotkey, record 0002. 2026-08-30

**Live interactive run.** The app was driven for real on the user's own
machine. The user performed every physical action and reported what they saw
and heard; this session read the database, the Credential Manager, the Windows
consent switches and the app's own stderr independently. Nothing below was
inferred from reading the code.

Build: commit `5ffc7f2`, clean tree. Two full restarts during the run.

The milestone 1 report of 2026-08-29 is preserved at
`report-2026-08-29-milestone-1.md`.

Approach bar: **tracer bullet**, from `docs/plan.md` and AGENTS.md. The whole
path must work end to end for real; only breadth may be faked. Three of five
milestones are built, so the slice under test is the hotkey, the pill, the
microphone, the error screens and the Deepgram key. Nothing streams and
nothing is typed yet.

Weight: heavy.

## Verdict: BLOCKED

**Nothing failed.** Not one of the 32 criteria came back `not met`. Eleven are
met with cited evidence, twenty-one are blocked, and every block is either a
milestone that does not exist yet, a machine limitation, or a decision the
user took during the sitting.

The verdict is BLOCKED and not PASS because a pass requires every numbered
criterion met, and nine of them belong to milestones 4 and 5. Read the table,
not the word.

**What this sitting was called to do, it did.** The 2026-08-29 run returned
FAIL on AC-27 and AC-5 and BLOCKED on much of the rest for want of a person at
the keyboard. Both failures are fixed and proved fixed. The level meter, the 5
minute cap, the microphone error kinds, the consent switches, the Deepgram key
and AC-32 were all exercised live for the first time.

One real defect was found, and it is not against a numbered criterion: **the
sound fallback does not fire**, and on a machine whose scheme has no device
sounds both dictation sounds are silent.

## The criteria

| Criterion | Outcome | Evidence |
|---|---|---|
| AC-1 pill and moving waveform within 1s | met | `AC-1-level-meter-moves.md` |
| AC-2 the machine's own device sounds | met | `AC-2-sounds.md`, and this run's stderr `open sound -> device event name` |
| AC-3 words typed at the cursor while speaking | blocked | milestone 4, not built |
| AC-4 typed text never taken back | blocked | milestone 4, not built |
| AC-5 second double tap closes, closing sound | met | `AC-5-hotkey-after-touching-the-pill.md` |
| AC-6 Ctrl+C then Ctrl+V does not start | met | live, step B3. Nothing happened |
| AC-7 words follow focus to another window | blocked | milestone 4, not built |
| AC-8 30s silence or 5 minute cap | blocked | `AC-8-five-minute-cap.md`. 5 minute half **met**; silence half built and deliberately unarmed until milestone 4 |
| AC-9 no key saved, guided setup screen | met | `AC-9-10-deepgram-key.md`. Link-out clause on the user's build-session observation |
| AC-10 valid key checked, saved, works after restart | blocked | `AC-9-10-deepgram-key.md`. Save and restart survival **met** first hand; "dictation works" needs a live stream, milestone 4 |
| AC-11 invalid key rejected, nothing saved | met | user's build-session observation, not re-observed here |
| AC-12 Settings shows last four only | blocked | milestone 5. Stored: `key_last_four = 64ef`, key absent from the database |
| AC-13 saved key stops being accepted | blocked | milestone 4 wires the trigger. Kinds, codes, sentences and actions built |
| AC-14 connection drops, one reconnect | blocked | milestone 4, not built |
| AC-15 microphone unavailable, cause named, privacy link | met | `AC-15-28-29-microphone-errors-live.md` |
| AC-16 hotkey does nothing signed out | met | live, step D3. Signed out, several double taps, nothing at all |
| AC-17 each dictation saved and survives restart | blocked | milestone 5. `dictation` table present, 0 rows |
| AC-18 a second account sees none of the first's | blocked | milestone 5, not exercised |
| AC-19 exactly two hotkeys, either side counts | blocked | Settings is milestone 5. Either-side clause **met** live: right Ctrl opened and closed the pill |
| AC-20 password field refusal | blocked | milestone 4, not built |
| AC-21 one sound switch in Settings | blocked | milestone 5, not built |
| AC-22 no way to set any other hotkey | blocked | milestone 5. No settings surface exists to check |
| AC-23 first open bottom centre, above taskbar, focused screen | blocked | not re-observed: a dragged position was already stored, so "first time" could not recur. Focused-screen clause needs a second monitor |
| AC-24 dragged position remembered | blocked | `AC-24-drag-stored-in-the-database.md`. Remembering **met** first hand; cross-screen clause needs a second monitor |
| AC-25 pill stays put when focus moves screens | blocked | one monitor on the machine |
| AC-26 screen unplugged or resized, pill still on screen | blocked | one monitor on the machine |
| AC-27 clicking or dragging never moves the cursor | met | `AC-27-pill-no-longer-takes-focus.md` |
| AC-28 window forward with code, sentence, one action | met | `AC-15-28-29-microphone-errors-live.md` |
| AC-29 blocked opens privacy page, others Try again | blocked | button labels **met**; the page actually opening and a successful Try again were not proved. User's decision to stop, recorded as such, not a defect |
| AC-30 nothing asks me to click the pill | blocked | pre-dictation half **met**: no pill on any error, no action on the pill. Mid-dictation half is milestone 4's, and see `finding-microphone-dies-mid-dictation.md` |
| AC-31 uses whichever microphone Windows is set to | blocked | "nowhere to choose one" **met**: no such control exists. Changing the Windows default and dictating again was not exercised |
| AC-32 an error clears itself once fixed | met | `AC-32-error-clears-itself.md` |

Met: 11. Blocked: 21. **Not met: 0.**

## Surfaces the record specced

Tables, read out of the live `echoscribe.sqlite3`: **3 specced, 3 present.**
`dictation`, `dictation_setting`, `deepgram_credential`, all created by the one
migration, all carrying `account_id`.

Commands: **13 specced, 9 present.** Four are absent.

| Command | Present | Note |
|---|---|---|
| `save_deepgram_key` | yes | |
| `get_deepgram_key_info` | yes | |
| `clear_deepgram_key` | yes | |
| `get_dictation_state` | yes | |
| `open_microphone_privacy_settings` | yes | |
| `open_deepgram_signup` | yes | |
| `open_deepgram_console` | yes | |
| `retry_dictation` | yes | |
| `get_hotkey` / `set_hotkey` | no | milestone 5 |
| `get_dictation_sounds` / `set_dictation_sounds` | no | milestone 5 |
| `pill_drag_finished` | **no, and not coming** | see below |

Events: **6 specced, 4 present**, and one present that is not specced.
`dictation:opened`, `dictation:level`, `dictation:closed`, `dictation:error`
all exist. `dictation:text` and `dictation:blocked` are milestone 4's.

## Two places the record no longer matches the build

Neither is a defect. Both are the record being out of date, which only a
surface check finds.

1. **`pill_drag_finished()` does not exist and should not.** The record's
   interface surface still says "the interface reports that a drag finished".
   The AC-27 fix moved the whole drag into Rust: the web view is out of the
   input path entirely, so the interface never sees the mouse and cannot report
   anything. `pill_mouse.rs` runs the drag and calls `remember_pill_spot`
   directly. This is a better answer than the record's, and the record has not
   caught up.

2. **`dictation:needs_key` exists and is not in the record's event list.** It
   is what AC-9 rides on. Milestone 3 added it; the record's Interface surface
   section was never amended to name it.

Route both to `/sync`, or `/architect` if the drag change is judged to need
ratifying.

## Found outside the criteria

1. **The sound fallback does not fire.** Reproduced three ways. With both
   device sounds set to `(None)`, the person hears nothing at all on open and
   nothing on close, and the app's own log still says it took the preferred
   name. `PlaySoundW` returning true means "Windows accepted the name", not
   "a sound was heard", so `play_first_that_works` never reaches
   `SystemAsterisk`. Full write-up and the isolating test in
   `AC-2-sound-fallback-never-fires.md`. This is the one thing in the sitting
   that is straightforwardly broken.

   It also makes record 0002's Still open section wrong where it says the
   fallback "is the only thing keeping the open and close audible" on such a
   machine. It is not keeping anything audible.

2. **On this machine `SystemAsterisk` and `SystemExclamation` are the same
   file**, `Windows Background.wav`. So even once the fallback fires, the open
   and close sounds would be identical here, and the record's build plan
   expects to "hear the two alert sounds".

3. **The microphone can die mid dictation with the pill still saying
   `MIC OPEN`.** Turning the desktop-apps consent switch off while the pill was
   open killed the stream. The pill stayed up, the bars went flat, and nothing
   told the person. A dead microphone looks exactly like a quiet room, because
   the meter is designed to read flat at silence. AC-30's mid-dictation half is
   milestone 4's by the record, so this is not scored against it, but the
   record's AC-30 list does not currently include the device failing after a
   successful open. See `finding-microphone-dies-mid-dictation.md`.

4. **The Try again busy state is effectively unreachable on a fast failure.**
   `mic-error.js` sets "Opening microphone" and awaits; when there is no device
   the failure returns immediately and the state is never seen. Not a defect,
   but the drawn state in `design/registry.md` is dead in that case.

5. **Disabling every recording device is not fully reversible by re-enabling
   one.** Windows promotes a new default when the old one is disabled and does
   not put it back. During the sitting this left Stereomix as the default
   capture device, which made the microphone appear dead system wide until it
   was set back by hand. Worth knowing for any future test that touches
   recording devices, and worth writing into the step 2a test instructions.

## Not checked, and why

- **Everything in milestones 4 and 5.** Transcription, typing at the cursor,
  the password field refusal, reconnect, history, the settings screen, the
  hotkey chooser, the sound switch, the masked key display. Not built, by
  design.
- **Anything needing a second monitor.** AC-23's focused-screen clause,
  AC-24's cross-screen clause, AC-25 and AC-26. The machine had one monitor
  and the user asked not to be asked again. Blocked for the second run running.
- **Two of the four microphone error kinds.**
  `microphone_in_use_by_another_app` needed an app that takes the microphone
  exclusively; nothing on the machine could. `microphone_unavailable` as a
  visible catch-all could not be forced with the switches allowing and a device
  present.
- **AC-29's two remaining clauses**, the privacy page actually opening and a
  successful Try again. The user chose to stop testing and considers them
  working. Their decision, recorded as such, not as a defect.
- **AC-9's link-out and AC-11**, both on the user's own build-session
  observation rather than this run's, at their direction.
- **AC-31's default-change half.** Not exercised.
- **AC-23 in full.** A dragged position was already stored, so the first-open
  case could not recur without resetting the row, which was not done.

## Heavy weight sign off

Not sought. Record 0002 is heavy and needs the user's own written approval in
this report, but that belongs to a pass. Nine criteria belong to milestones
that do not exist yet, so there is nothing to sign off for the feature as a
whole. The natural moment is after milestone 5, when all 32 can be judged
together.

## Hand-off

The plan row's verify boxes stay unticked, because a tick means a pass.

What this changes: milestones 1, 2, 2a, 2b, 3 and 3a have all now been driven
live, nothing failed, and the two faults that stopped the last run are proved
fixed. Milestone 4 is not blocked by anything found here.

Next, in order:

1. `/debug` on the sound fallback. It is the one broken thing, and it is small.
2. `/sync` on the two surface drifts, `pill_drag_finished` and
   `dictation:needs_key`.
3. `/architect` to hand milestone 4 the mid-dictation device failure, which its
   AC-30 list does not currently include.
4. `/develop` milestone 4.
