# AC-2: the deleted file fallback, heard. 2026-09-02

The eighth amendment ordered one more piece of work into step 1a before the
sound half could be re-verified: a check that the sound file a scheme entry
names is actually there. It was built on 2026-08-31. This is it making a sound.

## What was done

`sound.rs` reads the machine's sound scheme fresh on every play, so no restart
is needed for the behaviour. The line that names which sound name won prints
once per run, so the app was restarted to get a fresh one.

1. The current values were read and saved:
   - `DeviceConnect\.Current` = `C:\WINDOWS\media\Windows Hardware Insert.wav`
   - `DeviceDisconnect\.Current` = `C:\WINDOWS\media\Windows Hardware Remove.wav`
2. `DeviceConnect` was pointed at
   `C:\WINDOWS\media\EchoScribe Verify Deleted File.wav`, confirmed absent from
   the disk. `DeviceDisconnect` was left alone, so the two sounds should differ.
3. The app was restarted, and one dictation was opened and closed.
4. Both registry values were restored and verified afterwards.

## What happened

stderr, from the app:

```text
dictate: open sound -> alert fallback
dictate: close sound -> device event name
```

The opening sound's scheme entry named a file that is not on the machine, so
the entry was ruled `Silent` before Windows was asked at all, and the alert
fallback `SystemAsterisk` was used instead. The closing sound's entry was
intact and took the device event name, as it always has.

**And a person heard it.** Asked what they actually heard, the user reported
two different sounds: a generic alert chime on the open, and the usual hardware
unplug sound on the close.

That last paragraph is the whole point of this file. `PlaySoundW` reporting
success is not evidence a person heard anything, which is the mistake this
module's header now documents at length and standing rule 14 exists for. A log
line saying the fallback was chosen would have been equally true if the
fallback itself had been silent. The ears are the instrument.

## What this closes and what it does not

**Met**: AC-2's fallback now has both of its holes proved shut on this machine.
The `(None)` entry half was heard on 2026-08-31
(`AC-2-sound-fallback-none-entries.md`); the deleted file half is this file.
Together with the normal pair heard on 2026-08-30 (`AC-2-sounds.md`) and again
throughout this sitting, AC-2 is met.

**Not tested**: the `CannotTell` path, where a path form this code fails to
resolve is played anyway and Windows gets the last word. The record accepts
that as wrong-but-safe by name, and it is unit tested. No natural example of
such a path form exists on this machine to try.

Nothing shipped with the app. No sound file was added, and the sound scheme was
read by the app and only ever written by this test, then put back.
