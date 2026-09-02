# AC-8's silence half and AC-16, both live. 2026-08-31

Build: commit `3b4fb62`.

## AC-8: the 30 second silence cap, now armed, closes the pill on its own

Milestone 4 armed the cap that milestone 2 built unarmed. Live: dictation
opened on the scratch box, total silence in the room, hands off everything.
The harness polled the pill window continuously.

**The pill closed on its own 30,237ms after opening.** No tap, no click. The
box held zero bytes afterwards: nothing was typed by silence. The five minute
half was met on 2026-08-30 (`AC-8-five-minute-cap.md`); with the silence half
now observed, AC-8 is met in full.

## AC-16: signed out, the hotkey does nothing at all

The user signed out in the EchoScribe window. Three synthetic Ctrl double
taps, two seconds apart, each followed by a poll for the pill window:

```text
tap 1 -> NOTFOUND
tap 2 -> NOTFOUND
tap 3 -> NOTFOUND
```

stderr gained no line of any kind during the taps: the hook swallowed them
without reaction. The user's own report for the same interval: "Nothing at
all", no sound, no pill, no window, no flicker. Met.
