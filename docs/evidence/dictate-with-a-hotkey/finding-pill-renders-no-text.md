# Finding: the pill renders no text at all. Two criteria fail on one cause. 2026-08-31

Found live in the verify sitting, in two places, by the only instrument that
could find it: a person watching the pill while the machine checks passed.

## What was seen

1. **AC-33**: through a full two sentence dictation the pill never showed the
   grey unfinished wording. The user, asked directly: "the pill remained the
   same. have we developed the part of the pill that is supposed to show
   text?"
2. **AC-20's showing half**: on a live password refusal the pill showed
   neither `BLOCKED_PASSWORD_FIELD` nor its sentence. "It didn't type and
   just disappeared."

## The cause

The Rust side does its part. All eight specced events exist and fire:
`transcribe.rs` emits `dictation:interim` (line 461), `dictation:text` (line
493) and `dictation:blocked` (line 499), confirmed by grep and, for the
stream, by the fact that finalised phrases were typed all afternoon.

The pill's interface does not. `src/dictate/pill.js` registers exactly three
listeners: `dictation:opened`, `dictation:level`, `dictation:closed`. There
is no listener for `interim`, `text`, `blocked` or `error`, and nothing in
`pill.html` renders a transcript line. The events fire into a void.

The design side exists: `design/registry.md` draws the transcript line (final
ink, interim grey) and the Error pill, and `src/styles.css` already carries
`--color-ink-interim` and the transcript line's text size. What is missing is
purely the pill page reading the events and drawing them.

## What this fails, and what it does not

- **AC-33: not met.** The visible half never happened. The protective halves
  held everywhere they were probed: interim wording was never typed (every
  landed byte was final wording), never stored (the `dictation` table held 0
  rows all day), and wording still unfinalised at an abrupt stop was
  discarded.
- **AC-20: not met on its showing half.** The refusal itself is solid, twice
  proved; see `AC-20-password-refusal-live.md`.
- Also worth naming: any mid dictation error the record wants said on the
  pill in words (AC-13, AC-14, AC-30, `MIC STOPPED`) has no path to a person
  today for the same reason. Those criteria are blocked on the unbuilt
  "part a person reads" plan row anyway, but the fix for this finding and
  the build of that row overlap and should know about each other.

## Routing

`/develop`, against record 0002. This sitting changed no code. The registry's
transcript line and Error pill rows are drawn, so no `/canvas` work is owed
for AC-33 and AC-20 specifically; the four undrawn states named by the plan's
"part a person reads" row remain owed to `/canvas` as before.
