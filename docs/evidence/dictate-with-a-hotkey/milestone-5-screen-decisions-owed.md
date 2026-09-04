# Milestone 5, the settings screen: what the gate found owed, and what the user answered

**Date:** 2026-09-04
**Found by:** `/develop`'s gate, before any code was written
**Answered by:** the user, the same day, in the same sitting
**Status:** answered in the conversation and built. **`/architect` owes record
0002 an amendment carrying the two sentences**, and `/canvas` owes
`design/registry.md` three rows. Neither is optional: gate option 3 is "answered
here, and then it goes into the record, not just the chat".

This is the same shape as
[milestone-4-decisions-owed.md](milestone-4-decisions-owed.md), which the ninth
amendment carried in the same way.

## What the gate checked

Every value the screen must display or produce, against the record and
`design/registry.md`. Most had a named source and are not listed here: the
chosen hotkey and the two allowed values (`get_hotkey`), the wording per row,
the keycap, the `CHOSEN` badge, the switch label, the words On and Off, the
switch's value (`get_dictation_sounds`), both error codes and both error
sentences (record 0002's fourteenth amendment), the rule that no control is
drawn at all on a failed read, and the rule that the shown choice does not move
when a write is refused.

Six had no named source. Four were found before the build, two during it, when
the comp was read for the surface's own frame.

## 1. The caption beneath the two hotkey rows

`design/registry.md` "Hotkey choice" says there is one caption, once and not
per row, and says what it must carry: double tap to start, double tap again to
stop, and that left or right counts for each (AC-19). It does not fix the words.

**Answered:** "Double tap to start dictating, double tap again to stop. The
left and right keys both count."

Built in `src/dictate/dictation-settings.js` as `HOTKEY_CAPTION`.

## 2. The caption beneath the sound switch

`design/registry.md` "Sound switch" says there is one caption and what it must
carry: what AC-21 and AGENTS.md both require, and what a person would otherwise
reasonably fear, that the pill still appears when the microphone is open. It
does not fix the words.

**Answered:** "Turning sounds off silences the start and stop sounds only. The
pill still appears whenever the microphone is open."

Built as `SOUND_CAPTION`. It carries the AGENTS.md hard limit, that the
microphone never opens without something visible saying so, which is the reason
AC-21's switch silences sounds and nothing else.

## 3. What pressing the parent "Settings" rail item opens

Record 0004's AC-2 says every item in the rail opens a screen and none does
nothing. The rail holds three destinations, `settings`, `settings.dictation`
and `settings.transcription`, and neither record says what the parent opens.
Before this build it opened nothing.

**Answered:** it opens Dictation. The rail then marks Dictation as the one
showing with Settings as the section it is inside, which is the `--within`
state `setActive` already draws.

Built in `src/shell/dashboard.js` as `resolve`, which reads the sub-sections off
what `get_rail()` returned rather than off a shape assumed in the interface. So
the interface still cannot invent a destination.

## 4. Whether the Transcription surface is in this build

The plan row carries AC-12's visible half, the masked Deepgram key, which
`design/registry.md` puts on the Transcription surface as `Secret field`. The
build was scoped to Settings, Dictation.

**Answered:** Dictation only. `Secret field`'s "replace and remove" and its
"locally stored" wording are three more unnamed decisions, so it goes to
`/architect` first rather than riding along.

**Left open by that answer, and reported rather than worked around:** AC-12's
visible half, and record 0004's AC-2 for the `settings.transcription` item,
which still opens an empty surface.

## 5. The heading on the white surface

Found during the build, in the comp. `design/references/echoscribe-v2-design.html`
opens the Settings artboard with a 19px semibold heading reading "Dictation".
`design/registry.md` registers no heading component for this surface, and its
own rule is that a component not on the list does not exist.

**Answered:** build no heading. The registry wins; the comp does not override
it. The surface keeps the accessible name `dashboard.js` already gives it, from
the rail item that opened it.

**`/canvas` owes `design/registry.md` one row**: either a heading component for
this surface, or a note that there deliberately is none and why. Until then the
comp and the registry disagree in writing and the next build has to re-derive
this.

## 6. The label on the hotkey setting

Also found in the comp, which labels that control "Hotkey" above it, the way
`Sound switch` is labelled "Dictation sounds". The registry's `Hotkey choice`
row never mentions a label. Without one the two rows are a radio group with no
accessible name, which the accessibility checklist treats as a required item.

**Answered:** use the comp's own word, "Hotkey". It is not invented by the
build, and `Setting row`'s own shape in the registry is a label then a control,
so both settings on this surface take that shape.

Built as `.dset__label`, and the radio group is named by it through
`aria-labelledby` and described by its caption through `aria-describedby`.

## What each skill owes

| Skill | What it owes |
|---|---|
| `/architect` | An amendment to record 0002 carrying items 1 and 2, the two caption sentences, as Value sourcing rows. Items 3 to 6 are design or scope and belong below. Item 3 also touches record 0004's AC-2 and may want a line there. |
| `/canvas` | Three registry rows: the read failure state the fourteenth amendment already named as owed, the heading question in item 5, and the `Hotkey choice` label in item 6. |
| `/architect` | Record 0004's Still open already holds "what the Transcription section holds beyond the saved key row". Item 4 is that question arriving, plus `Secret field`'s replace, remove and wording. |

## What was deliberately not built, and why

- **`Device row`.** `design/registry.md` draws it on this surface, with a
  device name, a connection badge and a live input level. None of those three
  values has a source named in record 0002, and no acceptance criterion asks
  for it. Same reasoning as the pill's elapsed and word count chip: drawn, and
  waiting on `/architect` to name where its values come from.
- **Any loading state.** Both reads are local SQLite in the same process. The
  registry draws no loading state for this surface and a build may not invent
  one, so the surface is empty for the frame before the reads come back.
- **Any busy state on either control.** Same reason. The registry draws none,
  and the shown value does not move until the write comes back, which for a
  local write is imperceptible.
