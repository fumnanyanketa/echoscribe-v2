# AC-8: the 5 minute cap fires. First time it has ever run

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree.

## What was done

The user noted the clock, opened the pill by double tapping Ctrl with Notepad
focused, and then left the machine completely alone for six minutes. No key
was pressed and nothing was clicked for the whole of it.

## What happened

- The pill **stayed open past 30 seconds**, which is the correct behaviour.
  The silence cap is deliberately unarmed until milestone 4, so a close at 30
  seconds would have been a fault.
- The pill **closed on its own at roughly five minutes**.
- No closing sound was heard. That is not a second fault: `DeviceDisconnect`
  was still set to `(None)` in the registry at that moment, left over from the
  fallback test, and the fallback does not fire. See
  `AC-2-sound-fallback-never-fires.md`. The cap close path calls the same
  `sound::play_close` as a manual close.

## What this proves

`limits.rs::Deadlines::without_deepgram` had only ever been exercised by unit
tests, which fast-forward the clock. This is the first time the real 5 minute
deadline has run against a real clock, with `level_sink` checking it on each
60 ms tick and sending `Command::CloseBecause` once.

## Outcome

AC-8, the 5 minute half: **met**.
AC-8, the 30 second silence half: **blocked, by design**. It is built, unit
tested and deliberately unarmed until milestone 4, because the record sources
silence as "no final wording from Deepgram" and Deepgram is not connected yet.
Arming it on loudness would be a different promise. This is not a defect and
must not be read as one.
