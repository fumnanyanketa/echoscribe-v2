# Verify: the app shell, 2026-09-03

Record: [0004-the-app-shell](../../decisions/0004-the-app-shell.md), 7 criteria,
medium weight, In progress.

Approach bar: tracer bullet, from AGENTS.md's Build approach. The shell's plan
row records no override of its own. Under that bar the whole path has to work
for real and only breadth may be missing, so an empty white surface behind a
working rail is the plan and not a defect, and a window that will not come back
where it was left is a defect.

**Verdict: FAIL.**

One criterion was observed not to match. AC-7's sign out closes the dashboard
and then leaves the app running with no window at all, so a person is not left
on the sign in screen and has no way back into the app. Four more criteria are
blocked, three of them for reasons that were expected and one that was not.

| Criterion | Outcome | Evidence |
|---|---|---|
| AC-1 Signed in with a key saved, the dashboard shows without clicking: dark rail, white surface, Settings open on Dictation | met | `AC-1-dashboard-at-rest.png`, plus the window probe below |
| AC-2 Every item in the rail opens a screen, none does nothing, none is greyed out | blocked | `AC-2-rail-settings-active.png`, `AC-2-rail-dictation-active.png`, `AC-2-rail-transcription-active.png`. Every item responds; the screen it would open is not built yet |
| AC-3 Opens at 1200x800 the first time, resizes down to 960x640 and no smaller | met | `AC-3-first-open-and-floor.txt`, `AC-3-first-open-1200x800.png`, `AC-3-floor-960x640-after-drag.png` |
| AC-4 Resize it, drag it, close the app, reopen: same size, same place, same screen | blocked | `AC-4-close-and-reopen.txt`. Met on one screen, to the pixel. "Same screen" needs two monitors |
| AC-5 A screen that is gone, or a size that no longer fits, still opens wholly on screen | blocked | not exercised. Needs two monitors of different sizes with one unplugged |
| AC-6 An error brings the small window forward and leaves the dashboard alone; clearing hides it and reveals the dashboard unraised | blocked | `AC-6-two-windows-and-the-reveal.txt`, `AC-6-1-before-the-error.png`, `AC-6-2-error-forward-dashboard-unmoved.png`, `AC-6-3-small-window-hidden-dashboard-revealed-unraised.png`. First half met; the self clearing trigger cannot be produced |
| AC-7 Signing out closes the dashboard and leaves me on the sign in screen; a second account gets its own window place | **not met** | `AC-7-sign-out-leaves-no-window.txt`, `AC-7-sign-in-screen-after-restart.png` |

Also exercised in the same sitting, at the request that started this run:
record 0002's **AC-32** is in the same position as AC-6. The behaviour it asks
for, that the window does not come to the front, does not hide itself and does
not move, was observed through the dismissal route and is cited in
`AC-6-two-windows-and-the-reveal.txt`. Its own trigger, an error clearing
itself, could not be produced. AC-32 stays record 0002's criterion and is not
scored here.

Surfaces specced: 13. Present: 13. Missing: none. Listed at the end.

## How this was run

The app was built from the current source and driven as a real user, on this
machine, at 3840x2160 with Windows display scaling at 200%. Nothing was judged
by reading code.

- `cargo build`, then the built `echoscribe.exe` started directly. Started and
  restarted nine times across the run.
- Windows was asked what was on screen through a per monitor DPI aware probe:
  every window's title, whether it is visible, its outer and client rectangles,
  its place in the z-order, which one has focus, and the working area of the
  screen holding it.
- Screens were captured with `PrintWindow` for a single window's own pixels and
  with a full desktop grab for the stacking shots.
- Every click and every drag was real mouse input through the ordinary Windows
  path, and the hotkey was two real Ctrl taps. **No window under test was ever
  moved, resized or closed by calling an API on it.** The helper that moves
  other applications out of the way refuses any window belonging to
  `echoscribe.exe`, for that reason.
- `cargo test`: 204 passed, 0 failed, 2 ignored. Supporting only. A passing test
  is not an observation and no criterion above is scored on one.

Because display scaling is 200%, every size the record states in logical pixels
appears doubled in the numbers: 1200x800 logical is 2400x1600 physical, and the
960x640 floor is 1920x1280 physical. The app got this right everywhere it was
checked.

## AC-1, met

Started with a key saved and a session, no clicking:

```
title      : EchoScribe          visible: True   client 1920x1280   [FOREGROUND]
title      : echoscribe          visible: False                (the small window)
title      : EchoScribe dictation visible: False               (the pill)
```

The dashboard is the window at rest and the small window is hidden, which is the
invariant the whole feature leans on. `AC-1-dashboard-at-rest.png` shows the
dark rail on the left, the white reading surface on the right, the brand lockup,
Settings with Dictation raised and active and Transcription resting, and the
account block pinned at the rail foot with initials, name, mail address, signed
in date and Sign out. The initials circle is the raised rail grey, not violet,
which is the second amendment's decision.

## AC-2, blocked, and here is exactly what is true

All three rail items were clicked, one at a time, with a real mouse.

- **Settings** goes active, both children go resting. `AC-2-rail-settings-active.png`
- **Dictation** goes active. `AC-2-rail-dictation-active.png`
- **Transcription** goes active. `AC-2-rail-transcription-active.png`

So: no item does nothing when clicked, and no item is greyed out. Those are
AC-2's second and third sentences and both are met.

AC-2's first sentence, that every item opens a screen, is not true today, and
this record already says why. The white surface stays empty because record
0002's milestone 5 settings screen is not built, `design/registry.md` draws no
component for a surface waiting for its screen, and a build may not invent one.
The rail is doing everything a rail can do until that screen lands. The rest
waits on that one row, and nothing more.

## AC-5, blocked, and deliberately not worked around

This machine has one monitor. AC-5 needs a second one of a different size, the
window left on it, and that monitor then unplugged. A stored row naming a
missing screen could have been written by hand to walk the same code path, and
that was not done: the record itself warns that the unplug half is the half that
fails and the half it will be tempting to skip, so a hand written row would be
the skip wearing the shape of a proof.

The clamp behind AC-5 has 12 unit tests in `geometry.rs` and they pass. That is
not evidence for AC-5 and is not counted as any.

## Found outside the criteria

Ranked by the harm each would do.

1. **The microphone will not open at all, so dictation is dead on this machine.**
   With Windows microphone access set to Allow, three hotkey presses each
   answered `MICROPHONE_UNAVAILABLE`, "The microphone could not be opened."
   Another program was then asked to open the default recording device and did,
   recording 677 ms of audio, from the same session, moments later. Four capture
   devices are present. So the device works for other software and EchoScribe
   cannot get at it. `AC-6-microphone-would-not-open.png` is what it says on
   screen. This is record 0002's ground, not this record's, and it is
   what blocks AC-6's second half. It is the most damaging thing in this report
   after AC-7, because it is the product's one job. Route: `/debug`.

2. **AC-7's sign out, above.** Route: `/debug`. A hypothesis worth starting from
   is in `AC-7-sign-out-leaves-no-window.txt`.

3. **The dashboard moved itself once, and the app remembered the wrong place.**
   On one launch the window opened at the remembered place, physical 1894,374,
   confirmed by a probe five seconds after start. Some minutes later it was at
   114,12, and the app had written that new position into `shell_window` as
   though the person had moved it. Nothing in this run had touched the window.
   It did not recur across eight later launches, and my own screen capture was
   ruled out by capturing again and re-checking the position, which did not
   move. So: observed once, unexplained, not reproduced. Worth knowing about,
   because a window that can move itself makes a remembered place a lie, which
   is the exact failure AC-4 and AC-5 exist to prevent. Route: `/debug`, only if
   it recurs; there is nothing to reproduce today.

4. **A width can come back half a logical pixel different.** A window left at
   1130.5 logical pixels wide is stored as the integer 1131 and returns as 1131.
   One physical pixel at this scaling. Invisible, inherent to an integer column,
   written down so nobody later reads "identical" and finds otherwise.

5. **A first ever open writes no row.** The row appears only once the person has
   moved or resized the window. That is what the record's data rules say, so it
   is correct; it is noted because it looks like a missing write until you check.

6. **Not the app: your Deepgram API key is sitting in an unsaved Notepad
   document on this desktop**, and the window's title carries part of the key
   itself, so it went into this session's logs when I enumerated windows. I did
   not open, capture or close that document, and I have not repeated the value.
   AGENTS.md holds that the key is never logged or shown in full; that rule is
   about the app, and this is outside it, but it is the same key. Worth moving it
   somewhere that is not a window title.

## Not checked, and why

- **AC-4's same screen half and all of AC-5.** One monitor on this machine.
- **AC-6's and record 0002's AC-32's self clearing trigger.** The microphone
  will not open, so `dictation:opened` cannot fire. Finding 1.
- **AC-7's second account half.** No second Clerk account's credentials.
- **Record 0003's AC-14, the offline row.** This record's milestone 3 is where it
  is proved, and it was not part of this run. The code for it is present in
  `src/shell/account-block.js` and the dashboard's two event listeners are
  registered before anything is asked of Rust, which is the fix the build
  session made. Neither was exercised. It belongs to record 0003's evidence set.
- **The Deepgram key setup screen shown to a signed in person with no key**, the
  second amendment's replacement for the deleted `mountSignedIn`. Reaching it
  means clearing a saved key, which was not in this run's remit.
- **Whether closing the dashboard while an error is up behaves.** Only the two
  orders in the record were exercised.

## Surfaces the record names, all present

| Surface | Present |
|---|---|
| `src-tauri/src/shell/mod.rs`, the one place that decides which window is at rest | yes, 432 lines |
| `src-tauri/src/shell/dashboard_window.rs` | yes, 200 lines |
| `src-tauri/src/shell/geometry.rs`, the clamp and the screen match | yes, 335 lines, 12 tests |
| `src-tauri/src/shell/store.rs`, the table and its migration | yes, 377 lines, 8 tests |
| `src-tauri/src/shell/rail.rs`, the fixed rail | yes, 134 lines, 4 tests |
| `src/shell/dashboard.html`, `dashboard.js`, `rail.js`, `account-block.js`, `dashboard.css` | yes, all five |
| `shell_window` table, 7 columns as specced | yes: `account_id, width, height, center_x, center_y, screen_name, updated_at`, read out of the live database |
| `src-tauri/capabilities/dashboard.json` | yes, scoped to `["dashboard"]`, granting `core:event:allow-listen` and `core:event:allow-unlisten` and nothing else. Not granted `core:default` |
| `get_rail()` registered as a command | yes, `src-tauri/src/lib.rs:44` |
| `mountSignedIn` and the `.signed-in` rules deleted | yes, no occurrence anywhere in `src/` |
| `MIC_ERROR_KINDS` and `isDeepgramErrorKind` deleted | yes, no occurrence anywhere in `src/` |
| The clearing table's one copy, `dictate/error_screen.rs` with `screen_for`, and the `screen` field crossing on `dictation:error` | yes, and proven live: the interface mounted the microphone error screen without inspecting a kind |
| `dictation:key_saved` and `dictation:key_cleared` emitted | yes, `dictate/deepgram_key.rs:399` and `:464` |

## What this run changed on your machine, and what was put back

- The `shell_window` row was deleted to reach AC-3's first ever open, with your
  permission, and **has been restored** to the values it held before this run:
  960x640, centre 0.7466145833333333 and 0.5084786821705426, `\\.\DISPLAY1`.
  A full copy of the database was taken first and is in this session's
  scratchpad.
- Windows microphone access was switched to Deny and **is back to Allow**.
- Visual Studio Code, Claude, two Chrome windows, Notepad and Settings were
  minimized so the dashboard was genuinely on screen, and **all are restored**.
  A `cmd` window titled TYPING TARGET stood in for the app a person was working
  in and has been closed.
- The app is running, signed out, on the sign in screen. **You will need to sign
  back in**, which was the cost you agreed to for AC-7.
- No application code, decision record or plan row was edited. `docs/plan.md`'s
  verify box for this row is **not** ticked, because this run did not pass.
