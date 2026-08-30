# AC-27: the pill no longer takes focus. Re-proved live, 2026-08-30

Replaces the failure recorded in `AC-27-clicking-the-pill-steals-focus.md`
(2026-08-29). Build: commit `5ffc7f2`, clean tree. The user performed every
action and reported what they saw.

## The setup

Notepad open with the word `hello` typed and the caret blinking in it. Pill
opened by double tapping Ctrl. **Notepad was never clicked again for the whole
of this test**, which is what makes it a real test: every keystroke below had
to reach Notepad on its own.

## What was done, and what happened

| Action | Then typed | Landed in Notepad |
|---|---|---|
| One click on the pill's grip, no mouse movement | `abc` | yes |
| One click on the middle of the pill, on the bars, no mouse movement | `def` | yes |
| Press on the grip, drag the pill a hand's width across the screen, release | `ghi` | yes |

Notepad's title bar stayed active-looking throughout all three, including
during the drag. The pill followed the mouse smoothly.

Notepad ended up reading `helloabcdefghi`.

## Why this is the whole of AC-27

The 2026-08-29 run failed on a plain single click with no drag at all, and the
keystrokes that followed were lost. All three of the record's layers were
exercised here: the click on the interactive grip, the click on an inert part
of the window, and the drag. None of them moved the focused window.

## The knock-on that came with it, AC-5

The 2026-08-29 run also found the hotkey went dead after the pill was touched,
until the person clicked somewhere else. Immediately after the drag above, with
**no click anywhere at all**, a double tap of Ctrl closed the pill on the first
try and the closing sound played. See `AC-5-hotkey-after-touching-the-pill.md`.

## Outcome

AC-27: **met**.
