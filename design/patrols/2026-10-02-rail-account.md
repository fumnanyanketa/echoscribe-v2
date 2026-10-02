# Drift patrol, 2026-10-02: the rail's account foot

Scope: the account block at the foot of the nav rail, raised by the user
from live screenshots. They said the sidebar looks scattered. They are
right, and this report says why.

## What the comp draws

Two different things, in two different places:

- **Rail foot** (comp line 253): one row only. The 26px violet initials
  disc and the display name. Nothing else.
- **ACCOUNT card** (comp lines 551 to 564): on the white Settings surface,
  under a mono `ACCOUNT` eyebrow. A bordered card (`#e7e9ef` edge, 10px
  radius, 16px padding, 12px gaps) holding: a 38px dark disc with the name
  stacked over the mono email beside it, then the violet dot with "Signed
  in since 2 Feb", then the Sign out button.

## What is built

The rail foot carries the card's whole contents: offline row slot,
initials and name, email, signed-in line, Sign out. That placement is the
user's own decision 4 of the 2026-10-01 patrol ("sign out stays in the
rail foot"), so it is not drift by itself. What nobody did is draw the
rail arrangement of that content, and the build shows it:

| Where | Rule broken | Kind | Fix | Severity |
|---|---|---|---|---|
| `src/shell/dashboard.css:253` (`.account__details`) | No composition exists for email and date in the rail. The comp nests name over email in one column beside the disc; the build puts them in a separate flush-left block, so the name indents past the disc and the email does not. Mixed alignments read as scatter. | gap | Decide placement (question put to the user below), then draw it and route to `/develop` | breaks the look |
| `src/shell/account-block.js:99` (`formatDate`) | "Signed in since September 4, 2026" wraps to two lines inside the 232px rail, and the 7px dot centres against both. The comp's card is wide and its date is short ("2 Feb"), so the comp never met this. | gap | Follows from the same decision. In a card there is room; in the rail the date needs the comp's short form | breaks the look |
| `src/shell/dashboard.css:176` (`.account`) | Four unrelated pieces (identity, details, button) sit in one column with one gap value and no grouping, which is the card's contents without the card. | gap | Same decision | breaks the look |

All three are one finding wearing three faces: **decision 4 kept the
content in the rail, and the comp's layout for that content assumes the
wide white card.** No token was ignored. The system itself has a hole
where "account block, in the rail, with email, date and Sign out" should
be drawn.

## The decision owed (user's, not this patrol's)

Two honest ways out. Both were put to the user on 2026-10-02:

- **A. Follow the comp.** Rail foot returns to disc plus name only. Email,
  signed-in line and Sign out move to the comp's ACCOUNT card on a white
  settings surface. This reopens decision 4 of 2026-10-01, which only the
  user can do. Where the card lands (the comp draws it under the Deepgram
  key, which maps to the Transcription surface, or a new Account
  destination) may need `/architect` if a new nav destination is wanted.
- **B. Keep decision 4, draw the rail version properly.** Adopt the card's
  internal anatomy inside the rail: name stacked over the mono email
  beside the disc, the signed-in line with the comp's short date form so
  it never wraps, Sign out last, and one consistent left edge for all of
  it.

Registry: 0 added. The `Account block` row ("Initials, name, and on
Settings the mail address and signed-in date") already gestures at the
comp's split and was never updated when decision 4 kept everything in the
rail; it is updated once the user chooses.

System changes proposed: whichever of A or B the user picks gets drawn in
`design/registry.md` first, then routed to `/develop`. This patrol edited
no code.

## Second finding, same day: the sub-nav never collapses

Raised by the user after the card landed: the Settings sub-sections are
always visible in the rail. The comp disagrees with the build. Its History
artboard draws no sub-nav under a resting Settings (lines 249 to 252), and
its Settings artboard draws the sub-nav open under an active Settings
(lines 427 to 434). Permanent visibility is drift, kind: discipline
against the comp, severity: breaks the look, found by the user. Fix: the
sub-nav shows only while its section is the one showing. Registry's
`Section sub-nav` row corrected; routed to `/develop`.

Amended after the user tried it: a second press on Settings folds the
sub-nav back without changing the screen, and the next press unfolds it.
The user asked for the fold in so many words, so it is their decision,
recorded on the registry row.

## Outcome, same day

The user chose **A, follow the comp**. Registry updated: `Account block`
corrected back to the comp's one row (disc and name, offline row stays),
and `Account card` registered on the Settings, Transcription surface at
the comp's values. Decision 4 of the 2026-10-01 patrol is reversed by its
own author. The build is routed to `/develop`.
