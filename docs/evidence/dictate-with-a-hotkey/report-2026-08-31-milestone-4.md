# Verify: Dictate with a hotkey, record 0002. 2026-08-31

**Live interactive run**, the fourth. The user was at the keyboard for every
ear and voice moment; everything scriptable was driven by a harness through
the real Windows input queue, and every comparison was mechanical: landed
text read by `WM_GETTEXT` and compared as bytes, the foreground window read
by handle, the pill window found by title and timed by polling, the database
read by SQL, the registry read and restored by script, stderr read from the
dev process. Nothing below was inferred from reading the code, except where a
finding's cause is named and said so.

Build: commit `3b4fb62`, clean tree, 149 tests passing with 1 ignored (the
step 4a spike), clippy clean at `-D warnings`. Three app restarts during the
run, one deliberate for the sound test.

Earlier reports preserved: `report-2026-08-29-milestone-1.md`,
`report-2026-08-30-milestones-1-3.md`.

Approach bar: **tracer bullet**, from `docs/plan.md` and AGENTS.md. The whole
path must work end to end for real. As of this sitting it does: hotkey,
microphone, Deepgram stream, and typed words at the cursor in another app,
all in one dictation, several times.

Weight: heavy.

## Verdict: FAIL

Two criteria are **not met**, and both fail on one cause: **the pill renders
no text.** `pill.js` listens to `opened`, `level` and `closed` and nothing
else, so AC-33's grey line and AC-20's `BLOCKED_PASSWORD_FIELD` message have
no way to reach a person. Both were exercised live and observed missing by
the user watching the pill. Everything protective behind both held: interim
wording was never typed or stored, and the password refusal refused,
mechanically proven, twice. See `finding-pill-renders-no-text.md`.

This is a narrower failure than the word FAIL suggests, and a more honest one
than BLOCKED would be: the tracer bullet fired end to end today, typing was
character perfect in both mechanisms across four scored dictations, and the
sound fallback made its first sound ever on this machine. But a pass requires
every criterion met, two were watched failing, and this report does not round.

## The criteria

| Criterion | Outcome | Evidence |
|---|---|---|
| AC-1 pill and moving waveform within 1s | blocked | timing and waveform **met**: pill in 126ms to 276ms across every open, measured (`step-1a-re-verify.md`, `step-4a-typing-proof.md`); waveform moved with voice, user confirmed. The focused-screen clause needs a second monitor |
| AC-2 the machine's own device sounds | met | `AC-2-sounds.md` (2026-08-30), re-heard both runs today; stderr `device event name` both ways; fallback pass `AC-2-sound-fallback-none-entries.md` |
| AC-3 words typed at the cursor while speaking | met | `step-4a-typing-proof.md`: both receivers, phrases mid speech, user confirmed; package cannot-tell path exercised via the unpackaged edit box |
| AC-4 typed text never taken back, only final wording | met | `step-4a-typing-proof.md`: join half byte-checked (no leading space, one space at joins); never-taken-back user confirmed; unfinalised wording discarded on abrupt stop (`AC-7-focus-follows-mid-dictation.md`) |
| AC-5 second double tap closes, closing sound | met | `step-1a-re-verify.md`: closed 325ms after taps fired straight after touching the pill, no other click |
| AC-6 Ctrl+C then Ctrl+V does not start | met | 2026-08-30 live, step B3 (`report-2026-08-30-milestones-1-3.md`), unchanged code path |
| AC-7 words follow focus to another window | met | `AC-7-focus-follows-mid-dictation.md`: split exactly at the mid dictation click, both landings character perfect |
| AC-8 30s silence or 5 minute cap | met | silence half: closed on its own at 30,237ms, nothing typed (`AC-8-16-silence-cap-and-signed-out.md`); 5 minute half `AC-8-five-minute-cap.md` |
| AC-9 no key saved, guided setup screen | met | `AC-9-10-deepgram-key.md` (2026-08-30) |
| AC-10 valid key checked, saved, dictation works, survives restart | met | save and restart halves 2026-08-30; the "dictation works" clause closed today: the saved key (`64ef`) streamed every dictation in this sitting |
| AC-11 invalid key rejected, nothing saved | met | user's build-session observation, as recorded 2026-08-30 |
| AC-12 Settings shows last four only | blocked | milestone 5. `key_last_four = 64ef` stored, key absent from the database |
| AC-13 saved key stops being accepted | blocked | kinds, codes, sentences, actions built and unit tested; the screens are the unbuilt "part a person reads" plan row |
| AC-14 connection drops, one reconnect | blocked | machinery built; not forced live, and its message is the same unbuilt row |
| AC-15 microphone unavailable, cause named, privacy link | met | `AC-15-28-29-microphone-errors-live.md` (2026-08-30); standing restated in `AC-15-mic-error-kinds-standing.md` |
| AC-16 hotkey does nothing signed out | met | `AC-8-16-silence-cap-and-signed-out.md`: three taps, no pill, no stderr line, user heard and saw nothing |
| AC-17 dictations saved, survive restart | blocked | milestone 5. `dictation` table present and holding 0 rows after a day of dictating |
| AC-18 second account sees nothing of the first | blocked | milestone 5 |
| AC-19 exactly two hotkeys, either side counts | blocked | settings surface is milestone 5; either-side clause met live 2026-08-30 |
| AC-20 password field: nothing typed, pill shows blocked | **not met** | refusing half met twice, mechanically (`AC-20-password-refusal-live.md`); the pill showed nothing, user watched (`finding-pill-renders-no-text.md`) |
| AC-21 one sound switch in Settings | blocked | milestone 5. `sounds_enabled` exists and is honoured |
| AC-22 no way to set any other hotkey | blocked | milestone 5, no settings surface to check |
| AC-23 first open bottom centre, focused screen | blocked | a dragged position is stored so first-open cannot recur without resetting the row; focused-screen clause needs a second monitor |
| AC-24 dragged position remembered, carries across screens | blocked | remembering half **met** again today, mechanically: drag of exactly 150px moved `pill_x` by exactly 150/1920 (`step-1a-re-verify.md`); carrying clause needs a second monitor |
| AC-25 pill stays put when focus moves screens | blocked | one monitor |
| AC-26 screen unplugged or resized | blocked | one monitor |
| AC-27 clicking or dragging never moves my cursor | met | `step-1a-re-verify.md`: inert click, grip click, 150px drag, all mid dictation, foreground window never changed, every typed character landed in the editor |
| AC-28 window forward with code, sentence, one action | met | `AC-15-28-29-microphone-errors-live.md` (2026-08-30) |
| AC-29 blocked opens privacy page, others Try again | blocked | labels met 2026-08-30; the page opening and a successful Try again stay untested at the user's standing decision, reconfirmed this sitting |
| AC-30 nothing asks me to click the pill | blocked | pre-dictation half met 2026-08-30; mid dictation half is the unbuilt "part a person reads" row, and this sitting's finding shows the pill currently could not say anything even if asked |
| AC-31 uses whichever microphone Windows is set to | blocked | "nowhere to choose one" met: no such control exists. The default-change half was attempted, voided by an instruction failure (nobody spoke), and then skipped at the user's decision |
| AC-32 an error clears itself once fixed | met | `AC-32-error-clears-itself.md` (2026-08-30) |
| AC-33 unfinished wording on the pill, grey, never typed | **not met** | the grey line never appeared, user watched (`finding-pill-renders-no-text.md`); protective halves held: only final wording ever landed, nothing stored |

**Met: 15. Not met: 2. Blocked: 16.**

Of the sixteen blocks: six are milestone 5 (AC-12, 17, 18, 19, 21, 22), three
are the unbuilt "part a person reads" plan row (AC-13, 14, 30), four need a
second monitor (AC-25, 26, and clauses of AC-1, 23, 24), and three are user
decisions or machine limitations standing from 2026-08-30 (AC-29, AC-31's
default half, and the two unforcible microphone kinds inside AC-15's family,
restated in `AC-15-mic-error-kinds-standing.md`).

## Surfaces the record specced

Tables: **3 specced, 3 present**, all carrying `account_id`, read live.
Commands: **12 specced** (13 minus `pill_drag_finished`, removed by the
seventh amendment), **8 present**; the four absent are the milestone 5
hotkey and sound settings pairs, unchanged from the last report.
Events: **8 specced, 8 present** on the Rust side, `dictation:text`,
`dictation:interim` and `dictation:blocked` new since milestone 4, confirmed
by grep and, for text and interim, by behaviour. The pill's interface reads
three of the eight, which is the sitting's headline finding, an interface
gap and not a missing surface.

## Found outside the criteria

1. **The pill renders no text of any kind.** The cause behind both not-met
   criteria, and it also forecloses every future pill message until fixed.
   `finding-pill-renders-no-text.md`. To `/develop`.
2. **The sound fallback's deleted-file check was never built.** The eighth
   amendment ordered it built by `/develop` before this re-verify; `sound.rs`
   calls it a known gap in its own header. The `(None)` half passed live; the
   deleted-file half has nothing to exercise. To `/develop`, step 1a.
3. **A mid dictation focus move puts one join space at the start of the new
   window's text.** Rule conformant (the space precedes every phrase after
   the dictation's first) and mildly surprising to a person. Recorded for the
   record's owner to weigh, not a defect.
4. **The join space lands in whichever window is focused when the next
   phrase arrives**, which is the same fact as 3 seen from the old window: a
   sentence split across windows leaves the first window with no trailing
   space. Same status.
5. **Deepgram inserted one plausible word substitution in one of five
   dictations** (`markets` for `market`). Accuracy note for the record's
   "accuracy is the one thing" line; not a typing fault, attribution in
   `step-4a-typing-proof.md`.

## Not checked, and why

- Everything blocked in the table above, for the reasons given there.
- **The password cannot-tell fallback** (typing proceeding where UI
  Automation cannot answer `IsPassword`): could not be forced on this
  machine; the rule is unit tested. The coverage spike in Still open also
  remains owed, and it needs the user and a list of real applications.
- **Both original spikes in Still open**: the deliberately narrow key at the
  stream, and `IsPassword` coverage. Both need the user, both unchanged.
- **AC-14 live** (a forced connection drop): not attempted; its visible half
  is unbuilt anyway.
- **The four microphone error kinds live on the fixed build**: two stand
  proved from 2026-08-30, two remain unforcible here
  (`AC-15-mic-error-kinds-standing.md`).

## Heavy weight sign off

Not sought. Record 0002 is heavy and the feature needs the user's own written
approval in this report, but that belongs to a pass. Two criteria failed and
nine more belong to surfaces that do not exist yet. The natural moment is
after milestone 5 and the fixes, when all 33 can be judged together.

## Hand-off

The plan row's verify boxes stay unticked, because a tick means a pass.

Next, in order:

1. `/develop` on the pill's text rendering: listeners for `dictation:text`,
   `dictation:interim` and `dictation:blocked`, drawing the registry's
   transcript line and Error pill. This alone clears both not-mets.
2. `/develop` on the sound deleted-file check, step 1a's outstanding piece,
   then one short `/check verify` pass to hear it.
3. `/canvas` then `/develop` on the "part a person reads" row, which unblocks
   AC-13, AC-14 and AC-30 and overlaps with fix 1.
4. Milestone 5, which unblocks six more.
5. A second monitor, whenever one exists, for the four screen clauses.
