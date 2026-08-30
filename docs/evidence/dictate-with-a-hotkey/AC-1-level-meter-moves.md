# AC-1: the level meter moves with the voice. First live observation

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree. This had never been
watched before.

AC-1: "double tapping Ctrl opens the microphone and the floating pill appears
with a waveform that moves with my voice, within one second of the second tap."

## What was done

Notepad focused with a caret in it. Double tapped Ctrl. Spoke aloud for about
five seconds, then went quiet for three.

## What happened

- The pill appeared, and the user reported it as instant, inside the one
  second AC-1 allows.
- The plug-in chime played on open.
- **The meter bars moved while the user spoke.**
- **The bars flattened when the user went quiet.**
- The pill read `MIC OPEN`.

That is the whole of the loudness path proved end to end for the first time:
`microphone.rs` computing a number off the live buffer every 60 ms,
`mod.rs::level_sink` emitting `dictation:level` to the pill window, and
`pill.js` drawing 18 bars from it. No audio reaches the interface; the event
carries one number.

## The clause that is still not proved

AC-1 also says the pill appears "on the same screen as the window I am typing
into". Not observed. The machine had only one monitor connected during this
sitting, so there was no other screen for it to get wrong. Blocked, in neither
direction, for the second run running.

## Outcome

AC-1, the waveform and timing clauses: **met**.
AC-1, the focused-screen clause: **blocked**. One monitor only.
