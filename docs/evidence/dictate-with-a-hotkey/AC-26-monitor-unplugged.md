# AC-26: with a screen unplugged, the pill still opens fully on screen

Live run, 2026-08-29. Record 0002 calls this "the part most likely to be
wrong", so it was run for real rather than reasoned about.

## Outcome: met

## What was observed

Step 8. The pill's remembered spot was the top left of the larger monitor,
from the step 5 drag. With the pill closed, the user physically unplugged
that larger monitor, clicked into a window on the remaining monitor, then
double tapped Ctrl.

User: "The pill appears fully on screen."

Nothing off the edge, nothing missing, nothing stranded on a screen that
is no longer there.

## Scope of this evidence

The two monitors are different sizes and different DPI, so the remembered
fraction was being applied to a screen it was not recorded on. That is the
case the criterion is about.

The resolution change half of AC-26 was not tested. Only the unplug half
was.
