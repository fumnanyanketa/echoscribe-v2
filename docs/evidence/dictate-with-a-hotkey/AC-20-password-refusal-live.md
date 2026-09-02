# AC-20: a real password field refused, twice. And the pill said nothing. 2026-08-31

Build: commit `3b4fb62`. The field: `<input type="password">` on a local page
in Google Chrome, a real password field to UI Automation. The user clicked
into it, dictation was started, the user spoke one sentence.

## The refusing half: met, twice, mechanically

Run 1: the pill closed by itself 5,522ms after opening, with no closing tap.
Run 2 (rerun so the user could watch the pill): closed by itself 4,694ms
after the taps. In both runs:

- stderr printed `dictate: dictation stopped: BLOCKED_PASSWORD_FIELD`.
- The foreground window before and after was the same Chrome window, read by
  handle: the EchoScribe window did not come forward, exactly as the record
  decided for this one ending.
- Nothing was typed anywhere. The password field is unreadable by design, but
  no stray text landed in any other window, and dictation stopped rather than
  skipping keystrokes, which is the Risk section's fixed behaviour.

## The showing half: not met

AC-20's own words: "the pill shows that dictation was blocked." The record's
ninth amendment fixes what it shows: `BLOCKED_PASSWORD_FIELD` and "EchoScribe
will not type into a password field.", in the registry's drawn Error pill
shape, no action.

The user watched the pill through run 2 for exactly this. Their report: "the
pill did not show anything. it didn't type and just disappeared."

The cause is in the interface, not the events: `transcribe.rs` emits
`dictation:blocked`, but `src/dictate/pill.js` listens only to
`dictation:opened`, `dictation:level` and `dictation:closed`. No listener for
`dictation:blocked` exists, so the pill cannot show the refusal. The same
root cause as AC-33's missing grey line; see
`finding-pill-renders-no-text.md`.

Routed to `/develop`. Nothing was changed in this sitting.
