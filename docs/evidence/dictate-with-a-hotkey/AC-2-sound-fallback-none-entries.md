# AC-2: the sound fallback fires on a scheme with (None) device sounds. 2026-08-31

The re-run the eighth amendment owed. On 2026-08-30 this exact test failed:
both device sounds set to `(None)` produced silence on open and close, because
`PlaySoundW` reports success for an entry that is present and empty. Fixed in
`661ce2f`. This is the first time the fixed build has been heard on this
machine.

Build: commit `3b4fb62`, clean tree, `npm run tauri dev`.

## Method, all of it mechanical except the ears

1. Baseline read from the registry before anything was touched:
   `DeviceConnect = C:\WINDOWS\media\Windows Hardware Insert.wav`,
   `DeviceDisconnect = C:\WINDOWS\media\Windows Hardware Remove.wav`,
   `SystemAsterisk = SystemExclamation = C:\WINDOWS\media\Windows Background.wav`.
   That last line matters: the record's corrected pass is one sound heard
   twice, because both alert names resolve to the same file here.
2. Both device entries set to the empty string, which is what `(None)` on the
   Windows sound page writes. Verified by reading them back: `[]` and `[]`.
3. The app restarted, so the once-per-run stderr latch would print fresh.
4. One dictation opened and closed by synthetic double taps. The pill
   appeared 219ms after the second tap's press.
5. Both entries restored to the exact baseline paths and read back to prove
   it.

## Result: pass, both halves

The mechanical half, from the app's own stderr on the fresh run:

```text
dictate: open sound -> alert fallback
dictate: close sound -> alert fallback
```

The audible half, the user's own report: **one sound, twice**. The same alert
sound on open and again on close. Exactly the corrected pass the eighth
amendment wrote down. Silence, the old failure, did not recur.

For contrast, the same stderr lines on the normal scheme earlier in this
sitting read `device event name` on both, and the user heard the hardware
insert sound on open and the remove sound on close, both runs.

## What this does not prove

The deleted file case. An entry naming a `.wav` that no longer exists still
plays silence and reports success, and the check the eighth amendment ordered
built before this re-verify was never built: `sound.rs` names it a "known
gap, deliberately not covered here", and `classify` accepts any non empty
entry without resolving a path. That half is **blocked on `/develop`**, not
tested, and this file must not be read as covering it.
