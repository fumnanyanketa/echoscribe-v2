# Finding: the microphone dies on its own, mid dictation. 2026-09-02

Found outside the criteria, three times in one verify sitting, on the build at
commit `3b4fb62` plus its uncommitted working set.

## What was seen

Each time, stderr:

```text
dictate: the microphone stream reported an error: A buffer underrun or overrun occurred.
dictate: the microphone died mid dictation: The microphone stopped working. (Unavailable)
```

| When | Circumstances |
|---|---|
| Run 1 | 30 seconds of silence, nobody speaking. Died at the end of the run. |
| Run 2 | 30 seconds of silence, nobody speaking. Died at the end of the run. |
| Run 3 | About 12 seconds into an open dictation. **The user was asked directly what they had done and answered: nothing.** |

The Windows default input is the built in Realtek microphone array, confirmed
by the user. No device was unplugged, no privacy switch was touched, and no
other application was knowingly holding the microphone.

## Why it matters

This is the app's one promise failing by itself. A person mid sentence loses
dictation, the pill says `MIC STOPPED`, and the EchoScribe window comes forward
saying "The microphone stopped working." with a Try again button. All of that
is the specified behaviour working correctly, which is exactly why it is easy to
miss: **the handling is right and the underlying event should not be happening.**

The record's own framing applies. Accuracy is the one thing this app must get
right, and a dictation that ends on its own part way through a sentence costs
more than a misheard word: the person has to notice, reopen, and work out where
they got to.

It also silently made AC-30 easier to prove than it should have been. This
sitting scored AC-30's mid dictation half met on a device death nobody staged.
That is a genuine observation of the criterion, and it is also a symptom.

## What it actually was, settled 2026-09-02 by `/debug`

**The microphone never died. The app ended the dictation itself, on a report
that meant the opposite.**

The audio library uses one callback for two different things, and the code read
both as the same thing:

- Most reports arrive as that library gives up on the device, and after them no
  more audio comes. Ending the dictation is right.
- One report, the buffer under or overrun, is a *notification*. The sound card
  lost a moment of audio, Windows flags that on the very next packet it hands
  over, and the library passes the flag on and then carries on reading that
  packet and every packet after it. Nothing has stopped.

The library says so in its own words: "causing a potential audio glitch", and
for its sibling kind, "audio will still play". It is also the only place on
Windows that can produce this exact sentence, which is why the message was the
same all three times.

### Measured, not argued

A spike opened the default input device the same way the app does, counted
frames, and deliberately refused to stop on an error. Same machine, same
`Mikrofonarray (Realtek Audio)`, F32 at 48000 Hz, 2 channels.

| Condition | Reports | Frames after the first report | Stream |
|---|---|---|---|
| Idle, 45 s | **0** | n/a | fine |
| Audio thread starved by a 250 ms stall in the callback, 15 s | **58** | **20,160** | still capturing |
| Audio thread starved by ordinary CPU load, 35 s | **88** | **1,429,632** | still capturing |

Then the same thing through the app's own microphone code, after the fix, as a
test anyone can re-run: **100 reports in 30 seconds, 0 reported deaths,
1,299,744 frames captured.**

### Every part of the pattern is accounted for

- **Nothing was done to provoke it.** Correct, and nothing needed to be. It
  takes CPU load, not a person.
- **Twice at the end of a 30 second silence run.** Silence is not the trigger:
  idle for 45 seconds produced none at all. Load is. That sitting was driven by
  a harness capturing screen frames, and the app is at its busiest as a run
  ends, closing the pill, tearing down the stream and playing a sound.
- **Once about 12 seconds in.** A glitch has no schedule. Under load the spike
  saw its first inside 5 seconds.
- **The handling looked right.** It is right, for a microphone that stopped.
  This one had not.
- **It was intermittent.** It tracks machine load, which nobody was watching.

### Answers to what was not yet known

- *This app's audio path, the device, or something else holding it?* None of
  those. The glitch is the machine being busy; the lost dictation was this
  app's own reading of it. The app's audio callback was never the stall: it
  hands each chunk to a queue that drops rather than waits.
- *Does rapid open and close cycling make it more likely?* Not directly. It
  raises CPU load, which does.
- *Does it happen on a machine not driven by a harness?* Yes, on any busy
  machine. The harness only supplied the load.
- *Does it correlate with 30 seconds of true silence?* No. Proven: 45 seconds
  idle, zero reports.

### The second observation is answered too

The finding worried that a revoked permission surfacing as an under or overrun
would hand the person the wrong next step. It cannot now: an under or overrun no
longer ends the dictation at all, so it cannot route anybody anywhere. A real
revocation was already proven on 2026-08-30 to arrive as `Access is denied.`,
which is one of the reports that genuinely means the device is finished.

### The near miss the fix had to avoid

Changing the Windows default microphone mid dictation also leaves that library's
loop running, so it looks superficially like a glitch. It is not: the old device
stays attached, so the person would go on dictating into the microphone they
just stopped using. It reports as the stream being invalidated, or as the device
being gone, and both still end the dictation. AC-31 depends on that, and a test
now pins it.

## The fix

One file, `src-tauri/src/dictate/microphone.rs`. A named question, asked before
any death is reported: has the stream actually stopped capturing? Only the under
or overrun answers no. Everything else ends the dictation exactly as before, so
the settlement of 2026-08-30 is untouched.

A glitch now goes to stderr and nowhere else, saying plainly that dictation
continues, rate limited to the first and then every fiftieth, because a busy
machine can report a hundred in half a minute and a log nobody can read is not
a log. Nothing a person can see changed, which is deliberate: making a glitch
visible would be a new promise, and that is the user's to make, not this fix's.

## Residue

**No bad data.** Nothing was written while the bug was live and nothing needs
correcting. Audio is transient, the `dictation` table has no writer yet, and
the words typed before each cut were correct words that stayed where they were
typed.

**One wrong claim in the record of it.** The 2026-09-02 verify report scores
AC-30 as met, saying its mid dictation half was proved "on a real, unstaged
device death". There was no device death. That half of AC-30 is unproven and
still needs its own live proof: with the pill up and the meter moving, switch
Windows' desktop-apps microphone access off, which is exactly what milestone
4's step already asks for. `/check verify` owns that row, so `/debug` has
flagged it rather than edited it.

> **Closed the same day, 2026-09-02, by `/check verify`.** The row is corrected
> and the half is proved, on the fixed build at `08cd6e6`, by a person switching
> Windows microphone access off with the pill up and a transcript growing on it.
> Run twice, once through each of the two switches. Evidence:
> `AC-30-mid-dictation-device-death.md`.

**A gap this fix does not close, and did not open.** If a device ever stopped
delivering audio while reporting nothing at all, the pill would still read
`MIC OPEN` over a dead microphone, because a flat meter and a quiet room look
identical. AC-8's silence cap closes it after 30 seconds, so it is bounded, not
silent forever. Unchanged by this fix and worth knowing.

## How to check it

From `src-tauri/`:

```bash
cargo test                                         # 162 pass
cargo test -- --ignored a_load_starved_microphone   # opens the real mic, 40 s
```

The second one starves the audio thread the way that sitting did, and asserts
the two things that were false that day: nothing reports the microphone dead,
and audio keeps arriving to the end. Its stderr shows the glitches happening
while the dictation survives them.

The half this cannot prove is that a *real* death still ends a dictation. That
needs the app: start dictating, and with the pill up switch Windows' desktop
apps microphone access off. The pill must still say `MIC STOPPED` and the
EchoScribe window must still come forward with Open Windows settings.

## Routing

Done. `/debug`, 2026-09-02: reproduced on demand, cause proven, smallest fix
applied to one file, 162 tests plus one live test green.

Not a wrong decision. Record 0002's settlement of 2026-08-30 says "the
microphone dying after it opened" and means it. A glitch is not that. Nothing
in the record needs changing and `/architect` is not owed anything here.

Next: `/test` for the regression test, though three unit tests and one live
test landed with the fix. Then `/check verify` owes AC-30's mid dictation half
a real proof, because the one in the report was this bug.
