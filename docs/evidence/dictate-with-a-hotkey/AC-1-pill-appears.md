# AC-1 (milestone 1 slice): the pill appears within one second of the second tap, on the focused screen

Live run, 2026-08-29. Observed by the user at their own machine and
reported to this session. Milestone 1 has no microphone, so only the pill
half of AC-1 is in scope here. The waveform is a later milestone.

## Outcome: blocked

One clause was seen. The other was not, so this does not round up to met.

## What was observed

Step 2. Signed in, a window on the larger monitor focused, one double tap
of Ctrl.

User: "Yes. It still appeared, and it appeared fast."

So the pill does appear, and it appears well inside a second.

## What was not observed

"On the same screen as the window I am typing into."

Step 7 part 3 asked the user to focus a window on the other monitor and
reopen the pill, then say which monitor it appeared on. The answer given
was "the pill stayed in its position on both monitors", which does not
say which monitor it opened on. The user asked to move on rather than
repeat the step, so the question stands unanswered.

This matters because a single focused monitor cannot tell "the focused
screen" apart from "the primary screen". Only the cross monitor case can.
Until that is run, the focused screen clause is unproven.

## How to finish this

With two monitors plugged in, focus a window on the monitor the pill is
not currently opening on, double tap Ctrl, and note which monitor the pill
lands on.
