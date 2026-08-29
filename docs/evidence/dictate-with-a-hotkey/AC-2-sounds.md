# AC-2: the machine's own device plugged in and unplugged sounds

Live run, 2026-08-29.

## Outcome: met

## What was observed

Step 2, opening the pill. User: "it made a sound when it came up."

Step 3a, closing the pill with the right Ctrl. User confirmed the close
behaved as specified, which included the closing sound.

## Which sounds actually played

This is the spike record 0002 left open. `sound.rs` prints one stderr line
the first time each sound is asked for, saying which name won. Both lines
appeared in the live run:

```
dictate: open sound  -> device event name
dictate: close sound -> device event name
```

"device event name" means `DeviceConnect` on open and `DeviceDisconnect`
on close. These are the machine's own sound scheme entries for a device
being plugged in and unplugged, which is exactly what AC-2 asks for.

The `SystemAsterisk` / `SystemExclamation` fallback was never reached.

Raw capture: `M1-live-run-log.txt`.

## Note

No sound file ships with the app. The code only ever names a scheme entry
and asks Windows to play it.
