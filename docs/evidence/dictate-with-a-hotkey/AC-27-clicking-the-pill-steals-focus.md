# AC-27: clicking or dragging the pill never moves the typing cursor

Live run, 2026-08-29.

## Outcome: not met

This is the headline failure of the run. The pill takes focus away from
the app the person is typing into, and their keystrokes are lost.

## What was observed, first time

Step 5. The user opened Notepad on the larger monitor, typed `hello`, and
left the caret blinking there. They double tapped Ctrl to open the pill,
dragged the pill to the top left, let go, and then, without clicking
anywhere, typed `world`.

User: "no world did not land in the notepad after hello."

The word went somewhere other than the app the person was typing into.

## What was observed, second time, without a drag

Step 5b. The user clicked into Notepad so the caret was blinking there,
then single clicked the pill once, with no drag at all, and typed `x`.

User: "When I type x, it doesn't land in notes."

So a plain click is enough. Dragging is not required.

## The corroborating symptom

The user found this themselves while working through step 5:

User: "when I click on the pill to drag it and release my hand without
clicking on any other place the control doesn't work."

And after clicking away from the pill:

User: "Now the ctrl is working again."

So after the pill is clicked, the hotkey also stops responding until the
person clicks something else. That is recorded separately as the AC-5
failure. Both point the same way: the pill is being activated when it is
clicked.

## Why this is serious

Record 0002 puts this in its Risk section as a hard refusal, not a nice to
have. The whole point of EchoScribe is that words land where your cursor
already was, in the app you were already in. A pill that takes the cursor
when touched breaks that promise directly, and AC-24 tells the person to
touch it.

The intent is in the code. `pill_window.rs` sets `WS_EX_NOACTIVATE` and
`WS_EX_TOOLWINDOW` on the window and comments that this is exactly so
clicking it never moves the caret. On this machine that is not producing
the promised behaviour. This skill does not diagnose why and does not fix
it.

Routed to `/debug`.

## The spike this answers

Record 0002 left open: confirmation that the pill genuinely cannot take
focus while dragged. It is now confirmed, and the answer is no. The pill
can take focus, on a click as well as on a drag. See
`spikes-owed-to-record-0002.md`.
