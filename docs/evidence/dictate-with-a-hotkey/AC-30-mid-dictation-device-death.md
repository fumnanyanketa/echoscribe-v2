# AC-30's mid dictation half, staged for real. 2026-09-02

**Live interactive run**, the sixth. One criterion, one sitting. The user was at
the keyboard and threw the switch themselves both times.

This exists because the 2026-09-02 report scored AC-30's mid dictation half on
a device death that never happened. That ending was the buffer under or overrun
glitch, found and fixed the same day, evidence in
`finding-microphone-dies-on-its-own.md`. A dictation the app ended by mistake is
not a microphone dying, so the criterion had never been exercised. It has now.

Build: commit `08cd6e6`, clean tree, so the microphone fix in `1a3b5ff` is in
it. **Compiled and launched by this sitting**, not inherited: an instance
running since 15:55, from before those commits landed, was stopped first with
the user's approval. Launched 16:25:11, `cargo` reporting `Finished dev profile`
then `Running target\debug\echoscribe.exe`. Its stderr was captured to a file
for the whole sitting, which is where the lines below come from.

Nothing here was inferred from reading the code. The pill was photographed by
its own screen rectangle about four times a second, the foreground window was
read by handle, the three consent switches were read from the registry before
and after each run, and the landed text was read out of the receiver's edit
control by `WM_GETTEXT`.

## What AC-30 asks for, and what was seen

> **AC-30**: Nothing in dictation ever asks me to click the pill. When something
> goes wrong while the microphone is open, the pill says so in words and closes,
> and if there is anything I can do about it the EchoScribe window comes forward
> carrying that one action.

Run twice, against the two Windows switches that produce a real revocation. The
second run is the one record 0002's milestone 4 step names.

## Run 2: the desktop apps switch, the one the record names

Setup: a receiver window with one real Win32 edit control, focused. Double tap
of left Ctrl through the real Windows input queue at T0. Pill up 706 ms later.
The user spoke one sentence, watched it land, then switched **Let desktop apps
access your microphone** OFF in Windows Settings.

The microphone was genuinely capturing when the switch was thrown. Two
independent proofs, not one:

- The pill carried a live transcript that grew as the sentence was spoken: grey
  interim tail at 6,649 ms, hardening into final ink by 11,185 ms.
- The words reached the cursor. Read out of the receiver's edit control by
  handle afterwards: `The quick black fox jumps over the lazy brown dog.`, 50
  UTF-16 units.

Registry, read by this session:

```text
before  HKLM machine-wide=Allow  HKCU per-user=Allow  HKCU NonPackaged=Allow
after   HKLM machine-wide=Allow  HKCU per-user=Allow  HKCU NonPackaged=Deny
```

App stderr:

```text
dictate: the microphone stream reported an error: Access is denied. (os error -2147024891)
dictate: the microphone died mid dictation: Windows is not letting EchoScribe use the microphone. (BlockedByWindows)
```

That is a real death, not a glitch. `Access is denied.` is one of the reports
`stopped_capturing` lets through, which is the whole point of the fix: the
glitch no longer ends a dictation and this still does.

The frames, at 4 to 5 a second:

| Frame | What it shows |
|---|---|
| `frames/AC-30-desktop-apps-last-frame-live-15058ms.png` | the transcript line still live, caret at the end, meter still drawn |
| `frames/AC-30-desktop-apps-mic-stopped-first-frame-15294ms.png` | **`MIC STOPPED`** in warning ink, warning accent down the left edge, meter flat, the last words still beside it |
| `frames/AC-30-desktop-apps-mic-stopped-still-held-17210ms.png` | still `MIC STOPPED`, 1,916 ms after it first appeared |

The pill went invisible at 17,414 ms. **The hold measures 2,120 ms** from the
first `MIC STOPPED` frame, and at most 2,356 ms from the last live one. The
eleventh amendment's 2 seconds, plus the closing work after it.

Then the window:

- `frames/AC-30-desktop-apps-window-forward-open-windows-settings.png`: the
  EchoScribe window showing `MICROPHONE_BLOCKED_BY_WINDOWS`, "Windows is not
  letting EchoScribe use the microphone.", and exactly one action, **Open
  Windows settings**.
- **The coming forward is mechanical here, not judged by eye.** The foreground
  window was read by handle after the ending and *was* the EchoScribe window.
  It was not before: the foreground log shows the receiver box until 14,794 ms,
  then the Settings app when the user clicked the toggle, then EchoScribe.

The action is right for the cause. The person who just switched microphone
access off is sent to the page that undoes it, which is what the fifth
amendment argues for in as many words. A Try again there would be a door that
cannot open.

**Nothing on the pill asked to be clicked**, in this or any of the 74 frames.
The `MIC STOPPED` pill carries no code and no action at all; both belong to the
window that follows.

## Run 1: the machine-wide switch

The first run of the sitting. Same shape, one difference: the user threw the
master **Microphone access** switch rather than the desktop apps one. The
registry read after it says so plainly, `HKLM machine-wide=Deny` with
`NonPackaged` still `Allow`, which is why run 2 was done at all. Recorded here
rather than discarded, because it is the same criterion proved through the
other switch, and record 0002's "deny on any of the three" says both must reach
the same kind. They did.

```text
dictate: the microphone stream reported an error: Access is denied. (os error -2147024891)
dictate: the microphone died mid dictation: Windows is not letting EchoScribe use the microphone. (BlockedByWindows)
dictate: close sound -> device event name
```

- `frames/AC-30-machine-wide-last-frame-live-65830ms.png`: transcript live,
  meter moving.
- `frames/AC-30-machine-wide-mic-stopped-first-frame-66032ms.png`: `MIC
  STOPPED`, meter flat.
- Pill invisible at 68,033 ms. **Hold between 2,001 and 2,203 ms.**
- `frames/AC-30-machine-wide-window-forward-open-windows-settings.png`: the same
  window, the same code, the same sentence, the same one action. Foreground read
  by handle: the EchoScribe window.

## Outcome

**AC-30: met.** Both halves. The mid dictation half is now proved on a
revocation a person performed deliberately, twice, through both of the switches
that can cause one, with the pill's words photographed, the hold measured, and
the window's arrival read by handle rather than by eye.

## What this sitting did not check

- **The closing sound was not heard.** stderr says `close sound -> device event
  name` once, which is the app reporting what it asked Windows for, not proof a
  person heard anything. That distinction is standing rule 14, and it is exactly
  the mistake `sound.rs` once made. AC-2 was scored elsewhere and is not
  re-scored here.
- **Open Windows settings was not pressed.** Whether it opens the privacy page
  is AC-29's clause and stays untested at the user's standing decision.
- **The error was not cleared afterwards.** AC-32's path costs nothing extra in
  a sitting like this one, but AC-32 is not this sitting's row and its standing
  evidence is untouched.
- Nothing else in the 2026-09-02 report was re-exercised. The other 32 rows
  stand as they were.

## One correction to this sitting's own method

The phase script read the landed text through the focused control, and after the
ending that read is worthless: the EchoScribe window comes forward and covers
the receiver box, so the read returns the wrong window's text. Both runs printed
`[EchoScribe]`, 10 units, which is not what landed anywhere. The receiver's edit
control was then read directly by handle, needing no focus, and holds the 50
characters quoted above. The bad reading is named here rather than quietly
dropped, because it was in the run output and someone re-reading the transcript
would otherwise trip over it.

## How to check this yourself

From the repository root, with the app running:

1. Open Windows Settings, Privacy & security, Microphone. Leave **Let desktop
   apps access your microphone** on.
2. Put the cursor in any text field and double tap Ctrl. Speak a sentence and
   watch the words land.
3. With the pill still up, switch that toggle off.
4. The pill must say `MIC STOPPED`, hold it about two seconds, and close. The
   EchoScribe window must then be in front of you saying
   `MICROPHONE_BLOCKED_BY_WINDOWS` with one button, Open Windows settings.
5. Switch the toggle back on.
