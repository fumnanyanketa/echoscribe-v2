# AC-5, proved on its own: the hotkey after the pill has been touched

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree.

Record 0002's build plan step 1a asks for this one on its own rather than as a
consequence of the focus fix: "The hotkey still responding straight after the
pill has been touched, with no click anywhere else."

## What was done

Immediately after the drag in `AC-27-pill-no-longer-takes-focus.md`, with the
hand going straight from the mouse to the keyboard and **no click anywhere on
the machine in between**, the user double tapped Ctrl.

## What happened

- The pill disappeared.
- The closing sound played, the "device unplugged" one.
- It responded on the first double tap. No repeat was needed.

## Why it matters

On 2026-08-29 this failed. The hotkey went dead once the pill had been
clicked, and only came back when the person clicked somewhere else. Record
0002's Value sourcing calls that "a fault on this path in its own right,
whatever it looks like from outside", because the keyboard hook is machine
wide and must never read the foreground window.

`pill_mouse.rs` names the cause it fixed: Windows' own move loop is modal and
blocks the thread it runs on, so the drag now runs on `SetWindowPos` with
`SWP_NOACTIVATE` instead.

## Outcome

AC-5: **met**, including the case that failed before.
