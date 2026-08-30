# AC-15, AC-28, AC-29, AC-31: the microphone errors, live

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree. The user performed
every action at the machine and reported what they saw. The consent switches
were read independently from the registry by this session at the moment each
error was on screen.

## Round 1: Windows blocking the microphone, machine-wide toggle

Setup: Settings > Privacy & security > Microphone > **Microphone access** OFF.
Then Notepad focused, double tap Ctrl.

What the user saw:

- The EchoScribe window **came to the front on its own**.
- Code: `MICROPHONE_BLOCKED_BY_WINDOWS`
- Sentence: "Windows is not letting EchoScribe use the microphone."
- Button: "Open Windows settings"
- **No pill appeared. No sound played.**

What this session read from the registry while that screen was up:

```
HKLM machine-wide    Deny
HKCU per-user        Allow
HKCU NonPackaged     Allow
```

App stderr:

```
dictate: the microphone did not open: Windows is not letting EchoScribe use the microphone. (BlockedByWindows)
```

That is step 2b proved against the real switches. cpal reported its catch-all,
`consent::refine` read the three switches, found one saying `Deny`, and
sharpened the kind. The sentence and the button match the record's table
exactly.

## Round 3: the desktop apps switch

Setup: **Let desktop apps access your microphone** OFF, master toggle back ON.

- Code: `MICROPHONE_BLOCKED_BY_WINDOWS`
- Sentence: "Windows is not letting EchoScribe use the microphone."
- Button: "Open Windows settings"
- No pill, no sound.

Registry read later in the sitting, while that state persisted:

```
HKLM machine-wide    Allow
HKCU per-user        Allow
HKCU NonPackaged     Deny
```

Step 2b's second case, the one the record names separately. Both the
machine-wide and the desktop-apps switch reach the same kind, which is what
"deny on any of the three" means.

## Round 4: no microphone found

Setup: `mmsys.cpl` > Recording > every device disabled, disabled devices shown.

- Code: `NO_MICROPHONE_FOUND`
- Sentence: "Windows cannot find a microphone."
- Button: "Try again"
- No pill, no sound.

App stderr:

```
dictate: the microphone did not open: Windows cannot find a microphone. (NoMicrophoneFound)
```

The replaced wording is confirmed in place. "No microphone is plugged in",
which the record rejected for sending a laptop user after a cable, is gone.

## Round 5: Try again on a failure that is still failing

With every device still disabled, the user pressed **Try again**.

- No busy state was visible. The failure returns immediately, so the busy
  state drawn in `mic-error.js` is on screen for a few milliseconds at most.
  Not a defect, but worth knowing: the drawn "Microphone error, waiting" state
  is effectively unreachable on a fast failure.
- The screen redrew showing the same failure, which is the drawn behaviour.

The user then re-enabled a microphone and pressed **Try again** again. The
screen showed `MICROPHONE_BLOCKED_BY_WINDOWS`. That looked wrong and was not:
this session read the switches and found `NonPackaged` still `Deny` from round
3. With a working device and a denied switch, blocked is the correct answer,
and `retry_dictation` going through the same `try_start` path is what produced
it. The app was right.

## The two kinds that were not exercised

- `microphone_in_use_by_another_app`. Nothing on this machine could take the
  microphone in exclusive mode.
- `microphone_unavailable`, the honest catch-all. With the consent switches
  allowing and a device present, there was no way to force it. Note that it is
  reached constantly underneath: it is what cpal actually returns for a privacy
  block, before `consent::refine` sharpens it.

## Outcome

- AC-15: **met** for every case exercised. The pill never appeared, no sound
  played, the cause was named correctly each time, and the blocked case
  carried the privacy page as its one action.
- AC-28: **met**. Window forward, code, one sentence, exactly one action, in
  all three rounds.
- AC-29, which action goes with which kind: **met**. Blocked got "Open Windows
  settings"; the other kind got "Try again".
- AC-29, "pressing it starts dictation there and then if the microphone now
  opens": **not proved**. The user chose to stop testing at this point and
  considers it working. Recorded as their decision, not as a defect.
- AC-29, the privacy page actually opening when the button is pressed: **not
  proved**, same decision.
- AC-31, "there is nowhere in EchoScribe to choose a microphone": **met**. No
  such control exists. The other half, changing the Windows default and
  dictating again, was not exercised.
