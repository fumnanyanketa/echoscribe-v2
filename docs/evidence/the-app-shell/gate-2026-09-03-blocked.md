# The app shell: what `/develop`'s gate found, and why no code was written

**Written by `/develop` on 2026-09-03,** before any code for record 0004,
"the app shell", was written. `/develop` does not edit decision records or
`design/`, so the items below are written here instead.

**Nothing was built.** The gate found one owed item, the user was offered
three ways forward on 2026-09-03, and chose to have it drawn and recorded
before the build rather than after. So this run stops here, and the build
resumes once `/canvas` and `/architect` have each done one small thing,
named at the foot of this file.

The user also chose, the same day, that the run covers **record 0004 only**
and not record 0002's settings screen behind it. That is what record 0004's
own build plan already says, and it is written down here because it decides
what "done" means for the next run: at the end of it the rail's items will
go active and the white surface will be empty, so AC-2 is only fully
answerable once the settings screen lands on the row after.

---

## What the gate found

Record 0004 has a `## Value sourcing` table, so the input coverage test was
mostly a lookup. Every value the three milestones must produce, compute or
display, against the source the record names for it:

```text
dashboard shown at all                  AC-1, AC-7    auth state + deepgram_credential row
dashboard label and page                AC-1          this record (src-tauri/src/shell/, src/shell/)
first-open size 1200x800                AC-3          this record
smallest size 960x640                   AC-3, AC-5    this record
remembered size                         AC-4          shell_window.width / height
remembered place                        AC-4          shell_window.center_x / center_y, fractions
which screen it opens on                AC-4, AC-5    shell_window.screen_name, primary as fallback
that screen's working area              AC-3, AC-5    asked of Windows at open
the clamp on read                       AC-5          this record, the pill's own clamp
updated_at clock                        data model    derivation: sign_in::clock::now_iso8601
when the small window is shown          AC-1, AC-6    this record
error clears, small window hides        AC-6          this record, AC-32 read for two windows
sign out closes the dashboard           AC-7          this record
rail sections and the landing           AC-1, AC-2    this record: Settings; Dictation, Transcription
rail wording                            AC-2          design/registry.md
account block: initials, name, mail
  address, signed-in date               AC-2 (block)  record 0003 AC-5 and AC-7, AccountView
the "working offline" sign              record 0003   NO SOURCE NAMED
                                        AC-14
```

One value with no named source. It is not one of record 0004's own seven
criteria, which is exactly why it was nearly missed.

---

## The owed item: where the "working offline" sign lives

**What is owed.** Record 0003's AC-14 says that when the app opens with no
internet, or with Clerk unreachable, "the app shows it is working offline".
Today that sign is one line inside `mountSignedIn` in
`src/sign-in/sign-in.js`: a `!` marker and the words "Working offline",
styled by `.signed-in__offline` in `src/sign-in/sign-in.css`.

**Why this build breaks it.** Record 0004 requires `mountSignedIn` to be
deleted in the same change that lands the rail, under standing rule 11, and
its Sign out to move to the account block. It says nothing about the offline
sign. And by record 0004's own invariant the small window is hidden whenever
a person is signed in with a key saved, so there is no screen left for that
line to be on. Deleting `mountSignedIn` therefore takes record 0003's AC-14
from met to unmet, silently, unless the sign has a new home.

**Why `/develop` cannot simply build it.** `design/registry.md` draws no
working-offline component anywhere, for the rail, the account block or the
white surface. Its own opening rule settles what that means: "A component
not on this list does not exist. If a screen needs one that is not here,
that is a design decision: stop, run `/canvas`, add it here first, then
build it." The nearest drawn thing, the `Sign-in notice line`, lives on the
sign-in window and is not the account block, so reusing it in the rail would
be a new row in the registry either way.

**What was offered, and what was chosen.** Three options on 2026-09-03:

| Option | Cost | Chosen |
|---|---|---|
| Draw it first, then build the whole shell in one go | One extra session before any of the shell exists | **Yes** |
| Build all three milestones now, flag the sign as owed | A person with no internet gets no sign of it; record 0003 is heavy weight and could not be marked done | No |
| Build milestones 1 and 2 only, rail after the design pass | Nothing regresses, but Sign out lives on the hidden small window, so a person could not sign out at all | No |

Nothing about the offline sign is assumed in any file. No code was written.

---

## Two things the gate resolved by reading, which `/architect` should carry

Neither of these is owed, and neither blocks the build. Both are places
where record 0004 says something the codebase does not bear out, so they are
written down before they are re-derived from scratch.

**1. Record 0004's "Still open" first item is answered, and the answer is
not the one the record expects.** It asks "whether Tauri 2 needs an explicit
permission entry per application command in a non default capability", and
tells the build to read it against `src-tauri/gen/schemas/`. Read on
2026-09-03: `acl-manifests.json` and `capabilities.json` contain no entry
for any of this app's own commands. Searching both for `get_auth_state` and
`save_deepgram_key` returns nothing. Tauri's permission list covers core
plugin commands only, so **there is no per-command grant to give or withhold
for an application command.**

The consequence matters for the record's Risk section, which says the
dashboard is "granted the narrowest set that lets the dashboard call the
commands named under Interface surface and nothing else". No such set
exists. `dashboard.json` will name the dashboard's window label and grant an
empty permission list: no `core:default`, and nothing for events, windows or
anything else. What actually stops the dashboard reaching a command it
should not is the refusal already in Rust, that every command on this
surface returns `not_signed_in` when nobody is signed in. That is a real
control and it is already built; the capability file is the second line, not
the first. The record's instruction "do not resolve it by granting
`core:default`" is honoured, and the reason it was right is unchanged.

**2. The account block needs a third command, and it already exists.**
Record 0004's Interface surface names two, `get_rail()` and `sign_out()`.
The account block also shows initials, name, the mail address and the
signed-in date. Their source is named, record 0003's AC-5 and AC-7, and the
only route they travel is the existing `get_auth_state`, which returns
exactly those four on its `AccountView` and never a token. So no new
command, no new stored value, and, per the finding above, no capability
widened. Record 0004's Interface surface owes one line naming
`get_auth_state` as the third command the dashboard calls, so that a person
reading that section is not left thinking the account block invents its
contents.

---

## What each skill owes before `/develop` resumes

**`/canvas`**, one registry row: the working-offline sign, where it lives in
the shell, and what a person reads. It is the account block's business by
elimination, since the rail's other contents are nav items and the white
surface belongs to whichever section is showing, but that is a design call
and not this file's. Everything such a row needs already exists as tokens:
the rail ground, `--color-ink-nav`, `--color-ink-on-dark-secondary` and
`--color-warning-on-dark` are all in `src/styles.css` and already audited in
`design/check-contrast.py`. Two signals with words as one of them, per
mandate rule 5; today's line already does that with its `!` marker.

**`/architect`**, an amendment to record 0004 carrying three things: where
the offline sign lives, once `/canvas` has drawn it, and the two findings
above. Record 0003 wants a pointer to it too, because AC-14's display moves
out of the sign-in feature and into the shell.

**`/develop`**, then, the whole of record 0004 in one run: the two windows
and `dashboard.json`, the remembered geometry with its migration and clamp,
and the rail, the surface, the account block with its Sign out and its
offline sign, and the deletion of `mountSignedIn`. Nothing in this run is
already built, so there is no resume point to find: it starts at milestone 1.
