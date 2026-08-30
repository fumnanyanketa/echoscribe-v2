# AC-32: an error screen clears itself once the cause is fixed

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree.

AC-32: "If I fix what an error told me was wrong and then dictate
successfully, the EchoScribe window is no longer showing that error. It clears
on its own, quietly: the window does not come to the front, does not hide
itself, and does not move."

## The microphone half, observed first hand this sitting

Starting state: the EchoScribe window showing `MICROPHONE_BLOCKED_BY_WINDOWS`
from round 1 of the microphone error tests, with the machine-wide microphone
toggle off.

What the user did: turned **Microphone access** back **ON** in Windows
Settings. **The EchoScribe window was not clicked at any point.** Then clicked
into Notepad and double tapped Ctrl.

What the user saw:

- The pill appeared. The microphone opened.
- The EchoScribe window behind had **left the error screen**.
- It was showing the signed-in screen: "Signed in as fumnanya nketa",
  the account email, "Signed in since August 30, 2026", and a Sign out button.
  Confirmed by screenshot during the sitting.
- **The window did not jump to the front, did not move, and did not hide
  itself.**

That is the whole of step 3a: `dictation:opened` broadcast rather than sent to
the pill alone, and `src/main.js`'s listener leaving the error screen quietly.
The round trip the record describes, off to Windows and back with the hotkey,
was made for real.

## The key half

The three key errors clear when a key is accepted and saved. The user pasted
their real Deepgram key during this sitting and it was accepted; the setup
screen was gone afterwards and the hotkey opened the microphone from then on
(see `AC-9-10-deepgram-key.md`).

The user chose not to answer the separate question of what the window showed
in the instant after saving, and considers this working. Recorded as their
decision. The observable consequence, that the setup screen no longer stands
between the hotkey and the microphone, was confirmed first hand.

## The allowance half

`deepgram_no_allowance` clears on the first finalised words from Deepgram and
belongs to milestone 4. Not built, not tested, correctly absent: three source
guards in `mod.rs` keep that kind out of the shell, and `src/main.js` never
names it.

## Outcome

AC-32, the microphone half: **met**, first hand.
AC-32, the key half: **met** on the user's own observation, with the
consequence confirmed first hand.
