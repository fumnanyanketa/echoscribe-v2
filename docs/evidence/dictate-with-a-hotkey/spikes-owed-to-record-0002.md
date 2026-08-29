# The two spikes record 0002 left open, now answered

Both come from the live run of 2026-08-29. Record 0002 lists them in its
Still open section. This file is the answer. This skill does not edit
decision records, so landing these needs an `/architect` edit.

## Spike 1: which pair of system sounds actually plays

**Answer: the scheme's device event names. The fallback was never used.**

`sound.rs` tries `DeviceConnect` and `DeviceDisconnect` first, and falls
back to `SystemAsterisk` and `SystemExclamation` if the scheme has nothing
under those names. It prints one stderr line the first time each sound is
asked for.

Both lines appeared in the live run:

```
dictate: open sound  -> device event name
dictate: close sound -> device event name
```

So AC-2 is being met by the sounds it actually names, on this machine's
sound scheme. The fallback path exists but has not been exercised, and is
therefore still unproven.

## Spike 2: can the pill take focus while dragged

**Answer: yes, it can, and it does. This is the opposite of what the
record assumes.**

Two observations, both from the user at the machine:

- Caret in Notepad after typing `hello`. Pill dragged, let go, then
  `world` typed with no click in between. `world` never reached Notepad.
- Caret in Notepad. Pill single clicked, no drag. `x` typed. `x` never
  reached Notepad.

A third symptom points the same way: after the pill is clicked, the double
tap Ctrl hotkey stops working until the person clicks somewhere else.

`pill_window.rs` sets `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW` for exactly
this reason, and the record treats a non-activating pill as settled. On
this machine it is not behaving that way. Record 0002's Risk section
treats a pill that steals focus as a hard refusal, so this is not a small
correction to the record.

Full detail: `AC-27-clicking-the-pill-steals-focus.md`.

## What needs to happen with these

`/architect` should edit record 0002:

1. Close spike 1 with the device event name answer.
2. Close spike 2 with the answer no, and decide what follows. The record
   currently assumes the non-activating window style settles AC-27. The
   live run says it does not, so the record needs to say what the
   approach now is before `/debug` starts.
