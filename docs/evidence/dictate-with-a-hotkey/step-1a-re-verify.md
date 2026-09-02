# Step 1a re-verify: the pill's focus refusal, AC-5 on its own, and the drag stored. 2026-08-31

Live run against commit `3b4fb62`, clean tree, `npm run tauri dev`. The user
was at the keyboard and stayed silent; every action below was driven by a
scripted harness (synthetic mouse and keyboard through the same Windows input
queue a person uses), and every check was read mechanically: the foreground
window by handle, the landed text by `WM_GETTEXT`, the pill window by title,
the stored position by SQL. Nothing here was judged by eye.

Harness: `phase1.ps1` in the session scratchpad. Receiver: a WinForms window
holding a real Win32 edit control ("EchoScribe verify scratch box").

## What was proved, in order

The transcript, from the passing run:

```text
[1] bring the scratch box to the front and click into it
    fg=[WindowsForms10.Window.8.app.0.3a136c8_r8_ad1] [EchoScribe verify scratch box]
    box text cleared, now: []
[2] ctrl double tap, timing the pill
    pill appeared 199ms after the second tap's press
    pill rect 639,294,903,370 (264x76 in this process's coordinate space)
[3] click the inert bars at 799,332
    fg=[WindowsForms10.Window.8.app.0.3a136c8_r8_ad1] [EchoScribe verify scratch box]
    box now: [abcd]
[4] click the grip at 677,332, no drag
    fg=[WindowsForms10.Window.8.app.0.3a136c8_r8_ad1] [EchoScribe verify scratch box]
    box now: [abcdefgh]
[5] drag the grip 150px right
    fg=[WindowsForms10.Window.8.app.0.3a136c8_r8_ad1] [EchoScribe verify scratch box]
    pill rect now 789,294 (was 639,294)
    box now: [abcdefghijkl]
[6] ctrl double tap straight after touching the pill (AC-5)
    pill closed 325ms after the second tap
    final box text: [abcdefghijkl]
```

- **A single click on the pill, no drag** (steps 3 and 4, the inert bars and
  the grip): the foreground window never changed, and the four characters
  typed after each click landed in the edit box. String compared: the box
  held exactly `abcd`, then exactly `abcdefgh`.
- **The same after a drag** (step 5): the pill window moved exactly the 150
  pixels dragged (left edge 639 to 789), the foreground window never changed,
  and `ijkl` landed. Box held exactly `abcdefghijkl`.
- **The hotkey straight after the pill is touched, AC-5 proved on its own**
  (step 6): the second double tap arrived with no click anywhere after the
  drag, and the pill closed 325ms later. The closing sound played per stderr.
- **AC-1's one second, measured**: the pill appeared 199ms after the press of
  the second tap. In the first (aborted) run it was 276ms. Both are well
  inside the one second AC-1 allows. The waveform half of AC-1 is proved in
  the dictation phase of this sitting, not here.
- **The drag is stored** (AC-24's remembering half, re-proved against this
  build): `dictation_setting.pill_x` went from `0.4015625` to `0.4796875`,
  which is exactly 150/1920 of the working area, `pill_y` unchanged for a
  horizontal drag, `updated_at` refreshed to `2026-08-31T11:45:46Z`. Read by
  SQL before and after.
- **The web view is out of the mouse path**: stderr printed
  `dictate: pill web view kept out of the mouse path: WRY_WEBVIEW` at start.
- **The normal scheme sounds took the preferred names**: stderr printed
  `dictate: open sound -> device event name` and
  `dictate: close sound -> device event name`. The audible half is the
  user's report, recorded in the sitting log.

## Blocked in this sitting, and why

- **The cross monitor items.** AC-1's focused screen clause, AC-24's carrying
  clause, AC-25's other screen clause, and AC-26 all need a second monitor.
  The machine has one. Blocked, not worked around, the third sitting running.
- **The sound fallback's deleted file half.** The record's eighth amendment
  ordered a check that a scheme entry naming a deleted file is caught, built
  by `/develop` before this re-verify. It was never built: `sound.rs` says
  "Known gap, deliberately not covered here" in its own header, and
  `classify` accepts any non empty entry without looking for a file. There is
  nothing to exercise. Blocked on `/develop`.
- The fallback's `(None)` half was run separately; see
  `AC-2-sound-fallback-none-entries.md`.

## Honesty notes

- The first run of the harness was invalid and is not counted: a stray dialog
  overlapped the scratch box so focus never reached it, and the pill click
  coordinates were wrongly scaled by DPI, so the clicks missed the pill. Both
  were harness faults, found by the harness's own foreground checks, fixed,
  and rerun. The only values kept from run 1 are its independent pill timing
  (276ms) and the fact that dictation opened and closed cleanly.
- Synthetic input goes through the same input queue as a person's, and the
  activation behaviour it tests is the same. But it is not a physical hand:
  a touchscreen tap or a pen was not tested.
