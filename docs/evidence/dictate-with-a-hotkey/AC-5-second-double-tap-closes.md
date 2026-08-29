# AC-5: double tapping Ctrl again closes the pill and plays the closing sound

Live run, 2026-08-29.

## Outcome: not met

It works in the ordinary case. It stops working after the person touches
the pill, which is an action AC-24 explicitly invites.

## What worked

Step 3a. Pill open, double tap of the right Ctrl. The pill disappeared and
the closing sound played. User confirmed this behaved as specified.

## What failed

Step 5. The user dragged the pill to a new spot, let go, then double
tapped Ctrl.

User: "I double tap control, and the pill refused to disappear."

The user then narrowed it down themselves:

User: "when I click on the pill to drag it and release my hand without
clicking on any other place the control doesn't work."

So after clicking or dragging the pill, the hotkey stops responding until
the person clicks somewhere else. Clicking elsewhere restores it.

User, after clicking away: "Now the ctrl is working again."

## Why this is a fail and not a footnote

AC-24 tells the person to drag the pill. AC-5 says the second double tap
closes it. Straight after doing the first, the second does not work. There
is no message and nothing on screen to say why, so from the outside the
hotkey has simply stopped.

This is almost certainly the same underlying defect as AC-27: the pill
takes focus when it is clicked. It is recorded separately because it is a
separate promise to the user, and either could be fixed without the other.

Routed to `/debug`.
