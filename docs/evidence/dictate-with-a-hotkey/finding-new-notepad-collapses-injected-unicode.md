# Finding: the new Windows Notepad collapses injected text when it stalls

**Observed live, 2026-08-31**, during the /debug sitting that followed
milestone 4's first real run. The user performed every on-screen step and
supplied screenshots and pasted text; the terminal side ran under capture.
Build: commit `140dbe5` plus that day's uncommitted fixes.

This is the second of the two faults behind "dictation does not type
anything". The first, a startup crash in the encryption library selection, is
fixed in the same working tree and is not this file's subject.

## The verdict, in one paragraph

EchoScribe's typing is character-perfect. The same burst of key events that
arrives mangled in the new Windows Notepad arrives intact, 205 characters out
of 205, in a classic edit control (the Run dialog). The corruption happens
inside the receiving application: when the new Notepad stalls, every keystroke
that queued up during the stall is delivered as a copy of the newest queued
character instead of its own value. Runs like `,,,,,,,,,,,,,,,,,,,,` and 57
consecutive `h`s are that collapse, and their lengths match the text they
replaced. Windows accepts every event we send; acceptance was never the
problem, delivery inside the stalled app is.

## The evidence chain

Each step removed one suspect. All probe counts are from the temporary
counts-only diagnostic (grep `PROBE` under `src-tauri/src/dictate/`), which
never logs the words themselves.

1. **Deepgram exonerated.** Character-class histograms of each finalised
   phrase, taken before typing, matched the dictated sentences exactly, for
   example 33 letters, 9 spaces, 1 punctuation for "In the country of the
   blind, a one eyed man". The text handed to typing was correct.
2. **Windows' acceptance confirmed.** Every `SendInput` call reported every
   event accepted, across every run: 52/52, 86/86, 18/18, 40/40 live, and
   40+56+54+260 in each synthetic pass.
3. **EchoScribe exonerated entirely.** A test that types three fixed sentences
   through the identical code path, with EchoScribe not running, no keyboard
   hook, no microphone, no network, still produced the collapse in Notepad:
   a 28 character phrase landed as its first 3 characters plus 25 copies of
   its final comma.
4. **Pacing tried, and honestly judged insufficient.** Batches were cut from
   32 UTF-16 units to 8 with a 10 ms pause between calls. Notepad still
   collapsed a long phrase into `The` + 57 `h`s + dots. A backlog spanning
   seven paced batches means the app stalled through six pauses. Any receiver
   stall longer than the pause rebuilds the backlog, so pacing reduces the
   frequency and cannot reach zero.
5. **The receiver convicted.** The same paced burst into the Run dialog's
   classic edit control: all 205 characters correct, verified by exact string
   comparison, not by eye.

## Why this is a decision and not a bug fix

Record 0002's Risk section anticipated the class: "a small number of apps
handle fast simulated input badly. Those apps will need finding by use, not by
reading code." One has now been found by use, and it is the default text
editor of Windows 11, the first application most people will try dictation in.
A risk accepted as "a small number of apps" reads differently when the
Windows default is in the set.

No tuning of the current mechanism closes the gap: the collapse needs only a
receiver stall longer than our pause, and we cannot know when the receiver has
finished translating what we sent. Alternatives, clipboard paste and restore,
posting characters directly to the focused window, a per-application strategy,
or accepting and documenting the limitation, all change what AC-3 decided or
add behaviour the record does not describe. That routing is /architect's, per
/debug's own rule against papering over a flawed decision.

The pacing (8 units per call, 10 ms between calls) is kept as a mitigation for
marginal receivers, labelled as such in `typing.rs`.

## What was checked so it is not re-checked

- `send()` builds one down and one up per UTF-16 unit into a fresh vector, no
  index can drift, `wVk` stays zero, the character rides in `wScan`. Verified
  independently by the user before the synthetic runs.
- The audio format was eliminated the same way: device rate passed through,
  little-endian pairs pinned by test, Linear16 mono declared.
- The 403 to KeyNotAllowed mapping was never reached; spike 1 remains owed and
  untouched.

## Still open from the same sitting, parked deliberately

- The live run's stream oddities: a hello-shaped final arriving last, and the
  word "is" falling between two finals. Suspicion recorded, not proven: the
  SDK parses only four message shapes and `transcribe.rs` treats one
  unparseable message as a dropped connection, which would reconnect a healthy
  stream and lose in-flight words. Needs a live dictation with the terminal
  under capture.
- The missing space between consecutive finalised phrases, a one-line join fix
  in `transcribe.rs`, held during the investigation so every test ran against
  unchanged behaviour.
