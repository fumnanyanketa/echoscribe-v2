# AC-2, the sound fallback: it does not fire, and both sounds go silent

**Observed live, 2026-08-30**, during the `/check verify` sitting. The user
performed every action and reported what they heard. Build: commit `5ffc7f2`,
clean tree.

## What was being tested

Record 0002's build plan step 1a, fifth item: "set both device sounds to None
in Windows, open and close the pill once, hear the two alert sounds, put the
scheme back."

The record's Still open section says the fallback "is kept, because on a
machine whose scheme has no device sounds it is the only thing keeping the
open and close audible, and that sound is half of how a person knows the
microphone is on."

## What happened

**Run 1, the scheme intact.** Pill opened and closed by double tapping Ctrl.
User heard the plug-in chime on open and the unplug chime on close. App
stderr:

```
dictate: open sound -> device event name
dictate: close sound -> device event name
```

**Run 2, both device sounds set to `(None)`.** `mmsys.cpl` > Sounds >
Program Events > Windows > Device Connect and Device Disconnect, each set to
`(None)`, applied. The app was restarted first, because `sound.rs::report_once`
prints only the first time each sound is asked for in a process.

User heard **no sound on open and no sound on close**. The pill still appeared
and disappeared normally. App stderr:

```
dictate: open sound -> device event name
dictate: close sound -> device event name
```

The preferred branch was taken both times. `SystemAsterisk` and
`SystemExclamation` were never reached.

**Run 3, isolating it.** Device Connect restored to
`Windows Hardware Insert.wav`, Device Disconnect left on `(None)`. User heard
a sound on open and **no sound on close**.

That rules out the machine's audio being off, a muted app, or a mistaken
setting. The silence tracks the scheme entry exactly, one event at a time.

## Why the fallback never runs

`sound.rs::play_alias` treats `PlaySoundW` returning true as proof a sound
played:

```rust
unsafe { PlaySoundW(alias, None, SND_ALIAS | SND_ASYNC | SND_NODEFAULT).as_bool() }
```

With `SND_NODEFAULT`, a scheme entry that exists but names no file resolves to
silence and the call still succeeds. So `play_first_that_works` sees success
on the preferred name and never tries the fallback. The return value says
"Windows accepted the name", not "the person heard something".

## What this costs

The fallback is dead code on any machine that has the two entries present and
empty, which is exactly the machine it exists for. On such a machine both
dictation sounds are silent while the sound switch is on, and nothing tells
the person why.

Record 0002 leans on that sound twice: in "What this makes harder", where the
two sounds are what covers a pill left on a screen the person is not looking
at, and in AC-21, where sounds being off is meant to be a choice the person
made.

The "no silent listening" rule in AGENTS.md still holds. The pill is visible
whenever the microphone is open, and that is the rule's own requirement.

## Outcome

- AC-2, on this machine's own scheme: **met**. Both sounds are the right ones,
  from the machine's scheme, nothing bundled.
- The fallback named in record 0002's build plan step 1a and its Still open
  section: **not met**. It does not fire, and the sounds are silent instead.
