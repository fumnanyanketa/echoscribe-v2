# Verify: Dictate with a hotkey, milestone 1 ("The hotkey and the pill")

**Live interactive run, 2026-08-29.** Replaces the BLOCKED report of the
same date. The app was driven for real on the user's own machine, with two
monitors of different size and DPI, one of which was physically unplugged
during the run. The user performed every action and reported what they saw
and heard. Nothing below was inferred from reading the code.

Approach bar: Tracer bullet, from docs/plan.md and AGENTS.md. The whole
path must work end to end for real; only breadth may be faked. This is
milestone 1 of 5, so the slice under test is the hotkey and the pill, with
no microphone, no network and no transcription.

Weight: heavy.

## Verdict: FAIL

The pill appears, it is placed correctly, it remembers where you put it,
it survives a restart, it survives a monitor being unplugged, and both
sounds are the right ones. That is most of the milestone working.

It fails on the one thing record 0002 lists as a hard refusal in its Risk
section. **Clicking the pill takes the typing cursor away from the app you
are in, and the keystrokes that follow are lost.** A single click is
enough; dragging is not required. As a knock-on, the hotkey stops
responding after the pill is clicked, until you click somewhere else.

AC-24 tells the person to drag the pill. Doing so currently costs them
their place in whatever they were typing. That is the core promise of the
app broken by a normal action the feature invites.

## Criteria in milestone 1 scope

| Criterion | Outcome | Evidence |
|---|---|---|
| AC-1 pill appears within 1s of the second tap, on the focused screen | blocked | `AC-1-pill-appears.md`. Appears, and fast. The focused screen clause was never observed |
| AC-2 device connect and disconnect sounds from the machine's own scheme | met | `AC-2-sounds.md`, `M1-live-run-log.txt` |
| AC-5 second double tap closes the pill, closing sound plays | **not met** | `AC-5-second-double-tap-closes.md`. Works normally, stops working after the pill is clicked |
| AC-6 Ctrl+C then Ctrl+V does not start dictation | met | `AC-6-other-key-between.md` |
| AC-16 the hotkey does nothing when nobody is signed in | met | `AC-16-nothing-when-signed-out.md` |
| AC-19 partial: either side of the keyboard counts | met | `AC-19-either-side-of-the-keyboard.md` |
| AC-23 first open is bottom centre, above the taskbar, fully on screen | met | `AC-23-first-open-position.md` |
| AC-24 dragged position is remembered, across restarts and across screens | blocked | `AC-24-dragged-position-is-remembered.md`. Two clauses met, the cross screen clause never observed |
| AC-25 the pill stays put when focus moves to another screen | met | `AC-25-pill-stays-put.md` |
| AC-26 screen unplugged, the pill still opens fully on screen | met | `AC-26-monitor-unplugged.md` |
| AC-27 clicking or dragging the pill never moves the typing cursor | **not met** | `AC-27-clicking-the-pill-steals-focus.md` |

Two failed. Two blocked. Seven met.

## Surfaces specced for milestone 1

Specced: 4. Present: 4. Missing: none.

| Surface | Present |
|---|---|
| `src-tauri/src/dictate/` Rust module | yes |
| `src/dictate/` interface | yes |
| `src-tauri/capabilities/pill.json` | yes |
| Record 0002's three tables via one migration | yes |

Unchanged from the earlier report. The migration was additionally
exercised for real this run: the app opened the live
`echoscribe.sqlite3`, read the account's settings, saved a dragged pill
position, and read it back after a full restart.

## Findings, worst first

1. **The pill takes focus when clicked.** AC-27. Words typed after
   touching the pill never reach the app the person was typing into. A
   single click does it. Record 0002 treats this as a hard refusal.
   Route: `/architect` first, then `/debug`.
2. **The hotkey stops working after the pill is clicked.** AC-5. It comes
   back once the person clicks somewhere else. Nothing on screen explains
   the pause, so from outside the hotkey has simply died. Almost certainly
   the same root cause as finding 1, but a separate promise to the user.
   Route: `/debug`.

## The two spikes owed to record 0002

Both are now answered. See `spikes-owed-to-record-0002.md`.

1. **Which sounds play.** The scheme's `DeviceConnect` and
   `DeviceDisconnect`. The `SystemAsterisk` / `SystemExclamation` fallback
   was never reached. Confirmed by the stderr lines in both runs.
2. **Can the pill take focus while dragged.** Yes it can, and it does.
   This is the opposite of what record 0002 assumes.

**These need an `/architect` edit to land in record 0002.** This skill
does not edit decision records. Spike 2 in particular is not a tidy up:
the record currently treats the non-activating window style as settling
AC-27, and the live run says it does not, so the record needs to say what
the approach is before anyone starts fixing it.

## Found outside the criteria

- The pill does not close if the double tap comes straight after touching
  it. Covered above as the AC-5 failure.
- Run 1's log ends with "process didn't exit successfully (exit code: 1)".
  That is this session stopping the app with taskkill to test the restart
  in AC-24. It is not a crash.

## Not checked, and why

- **AC-1's focused screen clause** and **AC-24's cross screen clause**.
  Both need the same observation: focus a window on the other monitor,
  open the pill, and note which monitor it lands on. Step 7 was run but
  the answer given did not separate the two readings, and the user asked
  to move on rather than repeat it. Recorded as blocked, in neither
  direction.
- **AC-26's resolution change half.** Only the unplug half was run.
- **The AC-16 signed out screen** was not independently seen by this
  session. The user confirmed they followed the step, which included
  waiting for that screen.
- **Everything past milestone 1**: the microphone, the waveform, the
  Deepgram key, transcription, typing at the cursor, history, the settings
  screen. None of it exists yet, by design.
- **`sound.rs`'s fallback path.** It was never reached, so it is unproven.
- **Prettier on the interface files.** Not installed yet, per AGENTS.md.

## Heavy weight sign off

Not sought. A heavy feature needs the user's own written approval in this
report, but that step belongs to a pass. This run failed, so there is
nothing to sign off yet.

## Hand-off

Verdict is FAIL. The plan row 2 "hotkey and pill" verify box stays
unticked.

Next, in order:

1. `/architect` to land both spike answers in record 0002 and decide what
   AC-27 does now that the non-activating window style is not enough.
2. `/debug` on the focus theft, with the AC-5 hotkey stall as the same
   investigation.
3. Re-run `/check verify` after the fix, and finish the two blocked
   clauses at the same time. Both need only one cross monitor open.

Do not start `/check review` or `/warden` on the assumption milestone 1
is verified. It is not.
