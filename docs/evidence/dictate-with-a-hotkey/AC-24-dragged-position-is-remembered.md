# AC-24: drag the pill, and where you let go is where it opens next time

Live run, 2026-08-29. This criterion has three clauses. Two were proven.
One was not.

## Outcome: blocked

Two of three clauses met. The third was never observed, so the criterion
as a whole does not pass.

## Clause 1: it reopens where you dropped it. Met

Step 5. The user dragged the pill from bottom centre to the top left of
the larger monitor, let go, then closed and reopened it.

User: "When it reappears, it reappears exactly where I dragged it to."

## Clause 2: it survives closing and reopening the app. Met

Step 6. This session stopped `echoscribe.exe` and started a fresh
`npm run tauri dev`. The log shows a new process:

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.97s
Running `target\debug\echoscribe.exe`
```

The user then double tapped Ctrl.

User: "the dragged position survives a restart."

## Clause 3: the same relative spot on a different screen. Not observed

Step 7 part 3 asked the user to focus a window on the other monitor,
reopen the pill, and say which monitor it appeared on.

User: "The pill stayed in its position. on both monitors."

That does not answer which monitor it opened on. Asked to separate the two
readings, the user asked to move on instead. So this clause is unproven in
either direction. It is not recorded as a failure and it is not recorded
as a pass.

## How to finish this

With both monitors plugged in, drag the pill to an obvious spot on one
monitor, close it, focus a window on the other monitor, reopen, and note
which monitor it lands on and where.
