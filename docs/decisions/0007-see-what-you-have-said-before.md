# 0007. See what you've said before

**Status:** In progress
**Date:** 2026-09-04
**Weight:** medium
**Plan row:** 5
**Supersedes:** nothing
**Amended:** 2026-09-04, the first. This record refused one thing rather than
deciding it, the row's Copy action, and left the count off the row for a
missing counting rule. The user settled both the same day and this amendment
carries both. **Copy is built**, on a reading of AGENTS.md's data rules that
was the user's to give and is written here as a distinction inside the rule
rather than an exception to it: the app never routes transcribed text through a
third place of its own accord, and a person taking their own text at their own
request is not the app routing it. Record 0002's clipboard refusal of
2026-08-31 stands untouched, because that was the app moving text invisibly as
a typing mechanism. **The count is built and it counts characters, not words**,
on the precedent record 0005 set when it counted its vocabulary budget in
characters. The same rule settles the pill's elapsed and count chip, which is
record 0002's and is carried there by its twentieth amendment on the same day,
so one answer unblocks the two things this record's Still open said it would.
This adds AC-14 and AC-15 and takes the count to 15. No existing criterion is
renumbered or reworded, AC-13 included: it says EchoScribe never puts a
transcript on the clipboard "on its own", and a person pressing Copy is not the
app acting on its own. AGENTS.md is owed the data rule's new wording, which is
written out below and which `/sync` applies, because this skill never edits
that file.

**Decided without the user, on their own instruction.** The same standing
instruction records 0005 and 0006 were written under, given again for this
session: the user was away, and `/architect`, `/canvas`, `/develop` and `/test`
were authorised to run end to end on plan row 5, deciding anything with no
named source, taking the most reversible option, and writing the reasoning down
so it can be ratified or reversed later. That is `/develop`'s gate option 3.
Every such decision is marked **decided on the user's behalf** where it is made,
and all of them are collected at the end under "Decided on the user's behalf".

The instruction carried three refusals with it: nothing that costs money,
nothing that installs software, nothing that widens `src-tauri/capabilities/`.
None of the three was reached. **One other thing is refused and recorded rather
than decided**, because it is not this record's to settle: the row's Copy
action, which AGENTS.md's data rules forbid in as many words. See "What is
refused, and why" below.

## In one line

A History section, first in the rail and the screen EchoScribe opens on, lists
every dictation this account has finished, newest first, with the text that was
typed, when it was, how long it took and the language it ran in, and a search
over that text. It buys the fifth and last thing the brief said to build, and it
costs nothing new in storage: record 0002's milestone 5 has been writing these
rows since 2026-09-02 and nothing has ever read them back.

## What this is for

Every dictation since record 0002's milestone 5 has been written to the
`dictation` table and then never seen again. `src-tauri/src/dictate/store.rs`
says so at the top of the file, in a sentence written before this record
existed: "Reading history back is plan row 5's own feature and is deliberately
not here." This is that feature.

It is plan row 5's `Done when:` exactly: open your history and see the text from
something you dictated earlier, even after closing and reopening the app.

It is also the last of the five things AGENTS.md lists under "What to build",
and the one record 0004 has been holding a place for: that record's landing
destination is Settings, on Dictation, and it says in as many words that the
landing "moves to History when plan row 5 is designed, by an amendment carried
in that record and here".

## Acceptance criteria

- **AC-1**: History is where EchoScribe opens. Signed in with a key saved, the
  dashboard lands on History without me clicking anything.
- **AC-2**: History shows my dictations, newest first, each with the text that
  was typed, when it was, how long it lasted, and which language it ran in.
- **AC-3**: A dictation I did earlier is still there after closing and
  reopening the app.
- **AC-4**: A dictation I have just finished is in the list the next time I
  open History.
- **AC-5**: Typing in the search field narrows the list to the dictations whose
  text contains what I typed, and says how many matched. Clearing it shows them
  all again.
- **AC-6**: A search that matches nothing says so, keeps what I typed, and
  changes nothing about my history.
- **AC-7**: With more dictations than one page holds, there is a way to see
  older ones, and using it adds them below the ones already shown without
  losing my place or my search.
- **AC-8**: With nothing dictated yet, History says so, shows the hotkey to
  press, says the words stay on this machine, and offers one way to change the
  hotkey.
- **AC-9**: My history is mine. A second account signing in on the same machine
  sees none of my dictations.
- **AC-10**: If my history cannot be read when the screen opens, the screen is
  one error line and draws no control at all: no list, no search field, nothing
  holding a guessed value.
- **AC-11**: A dictation in a language that reads right to left is shown the
  right way round, and a dictation in any script is shown exactly as it was
  typed.
- **AC-12**: What I dictated appears on this screen and nowhere else in
  EchoScribe: never in a message about something going wrong, and never in any
  log or file the app writes.
- **AC-13**: I can select the text of a dictation and copy it myself.
  EchoScribe never puts it on the clipboard on its own.
- **AC-14**: Every dictation on the screen says how many characters were
  typed, and that figure is the same one the pill showed me while I was
  dictating it. Added 2026-09-04 by the first amendment.
- **AC-15**: One action on a dictation puts its text, and nothing but its
  text, on my clipboard, and tells me on the button that it did. Nothing else
  in EchoScribe ever writes to the clipboard. Added 2026-09-04 by the first
  amendment.

## The decision

**A new feature that reads a table another feature owns, and writes nothing.**
`src-tauri/src/history/` opens its own connection to the same SQLite file, the
way the shell's store, the dictate feature's, the language feature's and the
vocabulary feature's all do, and reads `dictation`. It creates no table, runs no
migration, and never writes a row. That is not a compromise reached here: it is
what record 0002's own build already decided and left a note about in
`store.rs`, and it is the reason `dictation` is indexed on account and start
time, "because that is the only way plan row 5 will read it".

**The ordering that makes it safe is named rather than assumed.**
`history::init` runs after `dictate::init` in `src-tauri/src/lib.rs`, because
the dictate feature owns `dictation`'s schema and creates it. History creating
the table itself was considered and refused outright: two features running the
same `CREATE TABLE IF NOT EXISTS` is two places a column can be added and one
place it can be forgotten, and record 0006's `language` column already showed
how that migration has to be written. One owner, one creator, one migration.

**No new table, no new column, and no amendment to record 0002.** This is the
first feature in this project that stores nothing at all. Everything AC-2 shows
is already in a row: `text`, `started_at`, `duration_ms` and `language`. That is
worth saying out loud, because it is what makes this row cheap and it is also
what fixes its limits: this screen can only ever show what record 0002 chose to
keep, and two of the things `design/registry.md` draws on a history row are not
in that list. See the two paragraphs below.

**The source app is not shown, and this record does not add it.** **Decided on
the user's behalf.** `design/registry.md`'s `Dictation row` draws "source app"
and its `Filter chip` draws a filter on "App and date". Nothing stores it.
Adding it would mean asking Windows for the name of the foreground application
at the moment the microphone opens, storing that against every dictation, and
amending record 0002's data model to hold it. Three reasons not to, and the
third is the one that settles it. It is new personal data of a kind this app has
never kept: a list of which programs somebody uses and when, sitting beside what
they said in each. It reaches back into a shipped feature's capture path for
something no acceptance criterion asks for. And it is the least reversible thing
this record could do, because a column added is a column that starts filling
with data on the next dictation. Not showing it costs a person nothing they have
today. `/canvas` is owed a correction to `Dictation row`, on the same footing as
`Language list`'s correction on 2026-09-04: the record wins on content and the
comp wins on look.

**The word count is not shown either, and the reason is record 0005's
reason. Reversed 2026-09-04 by the first amendment: a count is shown, and it
counts characters. This paragraph is kept exactly as it was written, because
its reasoning is the whole reason the count is not words.** **Decided on the
user's behalf.** `Dictation row` draws a word count.
The text is stored, so the value has a source, but the counting rule does not,
and there is no rule this project can apply that is true in every language it
now offers. Splitting on spaces is the obvious answer and it reports "1 word"
for a paragraph of Chinese, which is a wrong fact printed on screen in the
language of exactly the person record 0006 exists for. Counting characters
instead is script independent and is not what the registry draws, and a row
already carries a duration, which answers "how much was this" better than a
length does. Counting words correctly needs Unicode word segmentation, which in
Rust means a new dependency and is therefore the user's decision and not this
record's. So the row carries the time, the duration, the language and the text,
and no count. This is the same shape as the pill's elapsed and word count chip,
which record 0002 left deliberately unbuilt on 2026-08-31 for the same missing
source, and as `Device row`, which record 0002's milestone 5 left unbuilt on
2026-09-04. `/canvas` is owed the correction here too, and Still open holds what
would bring the count back.

**The count is characters, and one string is counted once.** Settled by the
user on 2026-09-04, on the precedent record 0005 set: that record counts its
vocabulary budget in characters because a count of words is not script
independent, and the paragraph above refused a word count for the same reason
in the same words. So the row says how many characters were typed. It is not a
compromise for a missing dependency: a character count is the honest figure
this app can produce for every language `Language list` offers, and it needs
nothing installed.

Three things follow, and each is a rule rather than a preference.

- **Rust counts, and it counts with `chars().count()`.** The same call record
  0005's `rules.rs` already uses, so this project has one meaning of "a
  character" and not two. The count is a sixth field on `DictationView`,
  computed from `dictation.text` on the way out. The interface never counts:
  JavaScript's own `String.length` counts UTF-16 code units, so an emoji or any
  character outside the basic range would come out as two there and as one
  here, and the same transcript would carry two different numbers depending on
  which side of the boundary asked. AGENTS.md puts transforming a value apart
  from displaying it for exactly this.
- **What is counted is the string that was typed, whole and untouched.** No
  trimming, no collapsing of spaces, and the joining spaces record 0002 puts
  between phrases are counted like any other character. The number answers "how
  much of my document is this", so it counts what went into the document.
- **The cost, named rather than hidden.** A character here is a Unicode scalar
  value, which is not always a thing a person can point at. An accented letter
  typed as a letter plus a combining mark counts two, and a family emoji counts
  several. Counting what a person would point at means grapheme clusters, which
  in Rust is a new dependency and therefore the user's decision, exactly as word
  segmentation was. This goes into Still open rather than being left to be
  discovered as a bug.

**The same rule settles the pill's chip, and that is record 0002's to carry.**
`design/registry.md`'s `Elapsed and word count` is drawn, its band is already
reserved in `pill_window.rs` so that its arrival resizes nothing, and it was
left unbuilt on 2026-08-31 for this record's missing counting rule and for no
other reason. It is now unblocked. It counts the same way off the same string:
Rust already holds `typed`, the exact text that becomes the history row, so the
running count on the chip and the final count on the row are two readings of
one string and cannot disagree. That is what AC-14 is a promise about. The chip
counts **finalised wording only**, which is the user's choice on 2026-09-04:
Deepgram revises interim wording, so a count including the grey tail would fall
as a person speaks, and it would not match the row afterwards. Record 0002's
twentieth amendment holds the chip's own criterion, because the pill belongs to
that record and this one may not add to it.

**The Copy action is built, and this is not an exception to the data rule.**
The refusal below is lifted by the user, who was the only one who could lift
it, and the reading they gave is a distinction inside AGENTS.md's data rule
rather than a hole in it: **the app never routes transcribed text through a
third place of its own accord, and a person taking their own text at their own
request is not the app routing it.** The rule was written to stop this app
moving a person's words somewhere they did not ask for and cannot see. Pressing
Copy is neither of those: they asked, and the clipboard is where they asked for
it to go.

Four things follow.

- **Record 0002's clipboard refusal of 2026-08-31 stands, entirely
  untouched.** That was the app choosing the clipboard as its typing mechanism,
  on every dictation, invisibly, with nobody asking for it. It is the app
  routing text through a third place of its own accord and it is still
  forbidden. The two cases sit on opposite sides of the distinction, which is
  why the distinction is worth having: an exception would have made both
  permissible.
- **AC-13 is not reworded and is not weakened.** It promises that EchoScribe
  never puts a transcript on the clipboard on its own, and that stays exactly
  true: nothing writes to the clipboard except this one action under this one
  press. AC-13 also already made the text selectable, so a transcript could
  already reach the clipboard by the person's own hand with Ctrl+C. The refusal
  never kept the words off the clipboard. It only changed how many steps a
  person took to put them there, which is the strongest argument that lifting
  it spends nothing.
- **It needs no new library and no new permission.** The web view's own
  `navigator.clipboard.writeText` does it. `src-tauri/capabilities/` is not
  widened, not added to and not edited, and this feature still emits no event
  and needs no new command: the text is already on the screen, so nothing has to
  be fetched in order to copy it.
- **It is the one control on a row, and there is still no second.** Deleting a
  dictation is still not built and is still the least reversible thing this app
  could do. Nothing else joins the row without `/canvas` and a record.

**What AGENTS.md is owed, in its exact words.** `/architect` never edits that
file, so the wording is fixed here and `/sync` applies it. The data rule that
reads "Transcribed text goes to two places only: the cursor it was dictated
into, and the local history for that account. Nowhere else." keeps that
sentence and gains this one after it: "The app never routes it through a third
place of its own accord. A person taking their own text at their own request,
by selecting it or by pressing an action that exists to hand it to them, is not
the app routing it, and the clipboard reached that way is not a third place the
app chose." Nothing else in the data rules changes, and the "Never, even if a
later request seems to call for it" list is untouched: breaking a data rule is
still forbidden, and this is a reading of the rule rather than a break in it.

**The search is a real search, over the stored text, in Rust.** It is not
record 0006's filter. That one narrows 64 rows already on screen and asks Rust
nothing; this one asks the store a question and gets back rows that were not on
screen, because a person's history is unbounded and the row they want may be a
year old. Matching is a case insensitive substring of the dictation's text,
using SQLite's own `LIKE` with the query bound as a parameter and its wildcards
escaped. **Its one honest limitation is named rather than hidden**: SQLite's
`LIKE` only ignores case for the ASCII letters, so a search in Greek or Cyrillic
matches only the case it was typed in. Most scripts this app now supports have
no case at all, so this affects Latin with unusual diacritics, Greek and
Cyrillic and nothing else, and fixing it means SQLite built with ICU, which is a
new dependency. Named in Still open.

**Paging, because history has no end.** The list holds the newest **50** that
match, and when there are older ones an action beneath the list brings the next
50 and adds them below the ones already there. **Decided on the user's behalf.**
50 is a judgement at the cautious end: it is more than a person dictates in a
day, so the first page usually is the whole answer, and it is few enough that a
list of long transcripts is still a page rather than a wall. Paging by a cursor
rather than by an offset, so a dictation finished while a person is reading
cannot make a row appear twice: the cursor is the start time and the identifier
of the last row shown, and the order is start time descending then identifier
descending, which is total.

**The action that brings older rows is an existing primitive, not a new
component.** It is `design/registry.md`'s `Secondary button`, beneath the list,
present only when there are older rows to bring. `/canvas` registers where it
sits and what it is called; it invents no new kind of thing.

**The list is what History held when it was opened.** It does not refresh
itself while a person dictates into another window. **Decided on the user's
behalf**, and it is the most reversible of the three answers. A refresh needs
the dashboard to listen to a third event, which record 0004's Still open says is
an amendment; there is no event that fires at the right moment anyway, because a
row is written at the close of a dictation and `dictation:text` fires per phrase
before the row exists; and the route back is one a person already takes, which
is pressing History in the rail, and `src/shell/dashboard.js` already re-mounts
a screen when its own destination is pressed. AC-4 is written to say exactly
that and nothing more.

**Nothing is truncated and nothing is collapsed.** A dictation's whole text is
on its row, wrapped. **Decided on the user's behalf.** Truncating hides the one
thing this screen exists to show, and an expand control is a component nothing
draws. The cost is real and is named in What this makes harder: a five minute
dictation is a tall row.

**Every string from a dictation reaches the screen as text and never as
markup.** This is the load bearing security decision of the record and it is
AGENTS.md rule 7 applied where it actually bites. A transcription is text from
outside the program, and this is the first screen in EchoScribe that renders a
whole one. A dictation reading `<img src=x onerror=...>` set as `innerHTML`
would run script inside the dashboard's web view, which can invoke every command
this app has. So every value on this screen is set with `textContent`, and there
is a Rust source guard of the shape `src-tauri/src/dictate/mod.rs` and
`src-tauri/src/vocabulary/mod.rs` already use: a test that reads
`src/history/history.js` and fails the build if `innerHTML`, `outerHTML`,
`insertAdjacentHTML` or `document.write` appears in it. AC-12 and AC-11 are the
criteria a person can check; this guard is what keeps it true after the next
edit.

**Text from a dictation carries `dir="auto"`, and so does the search field.**
Record 0006 settled this for the pill and the reasoning is the same here: the
browser lays each string out from the text it actually holds, so a dictation in
Arabic reads right to left and one in English does not, and no list of right to
left languages is held anywhere. `design/registry.md`'s `Term row` already does
it for the same reason. AC-11 cannot hold without it.

**The one sentence a person can read lives in Rust, in this feature, and
nowhere else**, the same division record 0002's fourteenth amendment fixed and
records 0005 and 0006 follow. It is **decided on the user's behalf**.

| State | Code | Sentence |
|---|---|---|
| The history would not read | `HISTORY_NOT_READ` | "Your history could not be read, so none of it is shown." |

**Two refusals carry no sentence at all**: nobody signed in, and the core not
being up. Neither can happen on a screen that only exists while somebody is
signed in, and both only happen while Rust is already closing the window the
screen lives in. A screen with nothing to draw draws nothing rather than
inventing a sentence for a state nobody designed.

**The rail gains History, as a top level item and not a sub-section, and it
goes first.** This is the amendment record 0004 has been expecting since it was
written: "each of plan rows 3, 4 and 5 amends this record to add its own item as
it lands". History has no sub-sections, so it is a sibling of Settings rather
than a child of it, and it is first because it is the landing and because the
comp draws History as what the window shows at rest. **Decided on the user's
behalf**, and it is the only place in the rail's order that this record touches:
Settings keeps its four children in the comp's order, unchanged.

**The landing destination moves to History, and record 0004's AC-1 is reworded
for it.** That record's Value sourcing says the landing "moves to History when
plan row 5 is designed, by an amendment carried in that record and here", and
its AC-1 ends "and Settings open on Dictation", which stops being true the
moment this lands. **This is the first acceptance criterion in this project to
be reworded rather than added to**, and it is legitimate because record 0004
wrote down in advance that this exact sentence was temporary. Nothing is
renumbered, nothing is added, the count stays at 7, and only the tail of AC-1
changes. Said here plainly because every amendment before it has been able to
end with "no criterion is renumbered, reworded or added", and this one cannot.

**The empty state is the first thing every new person sees, and it is the one
`design/design-system.md` wrote its empty state rule for.** That rule asks for
four things and this screen owes all four: what has not happened yet, the hotkey
shown as keys, one short paragraph saying where the words will go and that they
stay on this machine, and one thing to do next plus one way to change the setup.
The thing to do next is pressing the hotkey, which is what the keys are, so the
one action is the way to change the setup: a link to Settings, Dictation. Its
wording is settled below.

**The empty state asks Rust which hotkey to draw, through the command that
already exists.** `get_hotkey` is the dictate feature's, it takes nothing, it
refuses when nobody is signed in, and it returns the chosen hotkey. The history
screen invokes it, and only when the list is empty with no search term, which is
the only moment the keys are drawn. **Decided on the user's behalf**, over
adding a second door in Rust of the `terms_for_dictation` shape. A door is what
Rust to Rust needs; this is a screen asking Rust a question, which is the
ordinary shape of every screen in this app, and it adds nothing at all to a
shipped feature. If that call fails the empty state draws its sentence and its
paragraph without the keys, because keys are not a control and their absence is
an omission rather than a guessed value.

**One thing the mount contract gains, and four screens do not notice.** The
empty state's action moves the rail to Settings, Dictation, so the history
screen needs a way to say so. `src/shell/dashboard.js` passes its own `show` as
a second argument to whichever screen it mounts, and the four screens that exist
take one argument and ignore it. No other file changes.

**Nothing on the out of scope list is built, and four near misses are refused
by name.** No export to a file, which is on AGENTS.md's list of uninvited
arrivals by name. No editing a transcript: it is a record of what was typed into
somebody's document, and a history that can be edited is not a record. No
re-dictating or replaying: the audio was discarded at the moment the words came
back and there is nothing to replay. No sharing or sending anywhere, ever.

## What is refused, and why

**Lifted 2026-09-04 by the first amendment.** The user gave the reading this
section says only they could give, and Copy is built. Everything below is kept
exactly as it was written, because a refusal that was lifted is more useful
than a refusal that was deleted: it is the argument the decision had to answer,
and the next person who wants to route a person's words somewhere will find it
here rather than finding a rule that has quietly gone soft. What answers it is
in The decision, under "The Copy action is built, and this is not an exception
to the data rule".

**The Copy action on a row is not built, and this record will not settle it
alone.** `design/registry.md` draws `Copy action` and calls it "the only action
on a row". AGENTS.md's data rules say: "Transcribed text goes to two places
only: the cursor it was dictated into, and the local history for that account.
Nowhere else." The clipboard is a third place, and this project has already read
that rule this way once, deliberately and at cost: record 0002 refused clipboard
paste as a typing mechanism on 2026-08-31, in writing, because the words would
"transit the clipboard, a third place any clipboard tool can read, feeding
Windows clipboard history and cloud sync unless flagged, against a data rule
AGENTS.md holds as a hard limit". AGENTS.md lists breaking a data rule under
"Never, even if a later request seems to call for it".

There is a real argument the other way, and it is not weak: a person pressing
Copy is asking for their own words, which is not the app sending them anywhere.
That argument may well win. It is not mine to make on the user's behalf, because
the standing instruction is to take the most reversible option and a hard limit
is the one place where the reversible option is to stop. Not building it costs
nothing that cannot be added in an afternoon; building it spends a rule.

**What a person can do instead, today, and it is AC-13.** The text on a row is
selectable, so a person selects it and presses Ctrl+C, and Windows does the
copying. EchoScribe never touches the clipboard. That is not as good as a
button, and it is not pretended to be: it is named in Still open as the first
thing to settle when the user is back.

**And that is what happened, on the same day.** AC-13 stays exactly as it is,
because selecting the text is still a way to take it and the new action does
not replace it. What changed is the number of steps, which is the point the
paragraph above makes and which turned out to be the argument that won:
lifting the refusal did not put the words anywhere they could not already go.

**Nothing else was refused.** No new library is needed, nothing costs money, and
`src-tauri/capabilities/` is untouched: the dashboard listens to the same two
events it already listened to, and this screen listens to none of them.

## What else was considered

| Option | Why not |
|---|---|
| History reading `dictation` through a door in the dictate feature, the `terms_for_dictation` shape | Genuinely close, and it is the shape records 0005 and 0006 use. It is for Rust to Rust across a boundary where one feature needs a value another owns. Here a whole feature is the reader: it would mean the dictate feature growing a paged, searched, counted read of its own table for somebody else's screen, which puts plan row 5's code inside plan row 2's folder. `store.rs`'s own comment says the opposite in writing. |
| History creating the `dictation` table itself, so it can be opened in any order | Two features running the same `CREATE TABLE IF NOT EXISTS` is two places a column can be added and one place it can be forgotten. Record 0006's guarded `ALTER` already showed how narrow that path is. One owner, one creator. |
| Storing the source application with each dictation, so the row and the filter chip can be drawn as the comp has them | New personal data of a kind this app has never kept: which programs somebody uses and when, beside what they said in each. It reaches into a shipped feature's capture path, no criterion asks for it, and a column is the least reversible thing this record could add, because it starts filling on the next dictation. |
| A word count by splitting the text on spaces | Reports "1 word" for a paragraph of Chinese. It is the same mistake record 0005 refused when it counted its budget in characters rather than words, and it would be wrong for exactly the person record 0006 exists for. |
| A character count in the word count's place | Script independent and correct, and not what the registry draws. The row already carries a duration, which answers "how much was this" better, and two length-ish figures on one row is clutter for no gain. |
| A full text search index, SQLite's FTS5 | Faster on a large history and it tokenises, so it would search Chinese properly. It is a second copy of every transcript in a second table, which doubles what has to be deleted with an account and is a second place the person's words live. `LIKE` over a few thousand rows on a local file is not slow. |
| Offset paging rather than a cursor | One integer instead of two values, and it is what most screens do. A dictation finished while a person is reading shifts every row down one, so the next page repeats a row and hides another. The cursor costs four lines of SQL and cannot do that. |
| Loading the whole history at once, with no paging | Simplest of all, and it is fine for a hundred rows. A person dictating twenty times a day for a year has seven thousand, each a paragraph, and the screen would build all of them before showing any. |
| Showing only the newest page and reaching older ones by searching | No paging control to design. It fails the row's own `Done when:` for the person who wants a dictation from March and cannot remember a word in it. |
| The list refreshing itself while a person dictates | Needs a third event on the dashboard, which record 0004 makes an amendment, and there is no event at the right moment: a row is written at the close of a dictation and `dictation:text` fires per phrase, before it exists. Pressing History in the rail already re-mounts the screen. |
| Truncating a long transcript to a few lines with an expand control | The expand control is a component nothing draws, and a collapsed transcript hides the one thing this screen exists to show. |
| A delete on a row, or a clear-all | Record 0002's data model says history is kept "until the person deletes it", so a route is owed eventually. `design/registry.md` says Copy is the only action on a row, so a second one is a `/canvas` decision, and deleting somebody's transcript is the least reversible action in this app. Named in Still open rather than invented here. |
| Reusing record 0002's `SETTING_NOT_SAVED` type and its sentences | Four features now want a settings-style error line and this is the fourth. See the note below: naming it is this record's job, extracting it is not. |

## The settings error line is now wanted by a fourth feature

Record 0006's Still open asks whether the three field error line should become
a shared module, because AGENTS.md says "something becomes shared only when
three features need it. Two is a coincidence", and with that record three did:
the dictate feature's settings, record 0005's vocabulary and record 0006's
language. Record 0005's build declined to extract it and record 0006 carried the
question forward for the user.

**This record makes it a fourth, and does not extract it.** The history screen's
read failure is the same shape as the other three: a machine `reason` for a log,
an optional mono `code` and an optional `message`, with `None` for the refusals
that carry no line. What differs, and it is the part that matters, is the
sentences, and those stay in each feature because each feature's wording is its
own.

It is named here rather than done for the reason record 0006 gave and one more
of this session's own. Extracting it means changing three shipped features'
public types and rewriting their tests for no acceptance criterion, mid-build,
in a session the user is away for. That is the opposite of the most reversible
option. **Four features is a stronger signal than three, and the answer is very
likely yes**: when the user settles it, the thing to move is the three field
shape and nothing else, and every sentence stays where it is.

## Data model

**Nothing is added.** This is the first feature in EchoScribe that stores
nothing at all: no table, no column, no migration, and no write of any kind.

What it reads is record 0002's, unchanged:

| Table | Read | Written here |
|---|---|---|
| `dictation` | `id`, `account_id`, `text`, `started_at`, `duration_ms`, `language` | Never |

Rules that must always hold:

- **Every read is scoped to the signed in account**, in the `WHERE` clause and
  not by filtering afterwards. That is record 0002's AC-18 and this record's
  AC-9, and it is one clause that must appear in all three of this feature's
  queries: the page, the count, and nothing else.
- **This feature never writes to `dictation`, and never creates it.** The
  dictate feature owns that schema. A `CREATE`, an `INSERT`, an `UPDATE` or a
  `DELETE` in `src-tauri/src/history/` is a bug, not a feature.
- The order is `started_at DESC, id DESC`, which is total, so paging by a
  cursor cannot skip or repeat a row. `dictation_by_account_recent` is the index
  record 0002 created for exactly this read.
- **No transcript is ever logged, on any path.** Not on a read failure, not on
  a refusal, not in a diagnostic. This is stricter than record 0005's rule about
  a vocabulary term and for an obvious reason: these are the words a person
  actually said. A failure logs its machine reason and never a row.
- Deleting an account deletes its `dictation` rows, which record 0002 already
  says. Nothing is added to that list here, because this feature stores nothing.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The signed in account id | AC-3, AC-9 | The Clerk session held in the Rust core, per record 0003. Never passed in from the interface, and no command here takes one. |
| The dictations shown | AC-2, AC-3 | `dictation` rows for that account, newest first, read by this feature. Record 0002's milestone 5 has been writing them since 2026-09-02. |
| Which dictations get a row at all | AC-2, AC-4 | Record 0002, settled by its thirteenth amendment: a row when, and only when, at least one finalised phrase reached the cursor. A silent dictation and a password refusal leave no row, so this screen never shows a blank one. Nothing here can change that and nothing here should. |
| The text of a dictation | AC-2, AC-11, AC-13 | `dictation.text`, which record 0002 defines as exactly the phrases that were typed, in order, with the same joining spaces. So what is shown here and what is in the person's document cannot differ. |
| When it was | AC-2 | `dictation.started_at`, stored as UTC by record 0002. |
| What a person reads for that time | AC-2 | The screen, through the browser's own date formatting in the machine's locale and time zone. **Decided on the user's behalf.** Only the screen knows the machine's conventions, and a format invented in this record would be one more piece of wording with no source; this is the same division record 0006 put the language list's alphabetical order on, for the same reason. |
| How long it lasted | AC-2 | `dictation.duration_ms`, stored by record 0002. |
| What a person reads for that duration | AC-2 | The screen. **Decided on the user's behalf**: whole seconds below a minute, minutes and seconds above it. Rounded, never a millisecond figure, because nobody reads a dictation's length to three decimal places. |
| The language it ran in | AC-2 | `dictation.language`, added by record 0002's nineteenth amendment for record 0006. The code the dictation was asked with, never what Deepgram detected. |
| What a person reads for the language | AC-2 | `design/registry.md`'s `Language tag`, which already says: mono, the literal code. So `en`, `zh-TW`, `multi`. No wording is owed and none is invented: this is the one value on the row that is a code on purpose. |
| The word count | AC-2 | **Nowhere, and it is therefore not shown.** Counting words correctly in every language this app offers needs Unicode word segmentation, which is a new dependency and the user's decision. See The decision, and Still open. **Superseded 2026-09-04 by the first amendment**: the row carries a count of characters instead, on the two rows below. A count of words is still not shown and the reasoning in this row is still why. |
| How many characters were typed | AC-2, AC-14 | Rust, `dictation.text.chars().count()`, as a sixth field on `DictationView`. Added 2026-09-04 by the first amendment. The same call record 0005's `rules.rs` counts its budget with, so this project has one meaning of "a character". Never counted in the interface: `String.length` there counts UTF-16 code units and would disagree with this for an emoji or anything outside the basic range. |
| What a person reads for that count | AC-14 | This record, "The wording of the screen": the number then the word `characters`, and `character` at one. The user's choice on 2026-09-04, over the comp's abbreviated `84 w` shape, because this project writes plain words and both surfaces have the room. |
| The same count while a dictation is running | AC-14 | Record 0002, its twentieth amendment, off the same `typed` string this row is a reading of, so the chip and the row cannot disagree. Finalised wording only. |
| What Copy puts on the clipboard | AC-15 | The transcript already on the screen, exactly as `dictation.text` holds it and nothing else with it: no time, no duration, no language. The interface does it with `navigator.clipboard.writeText`; no command is added and no event is emitted, because the value is already there. |
| What a person reads after pressing Copy | AC-15 | This record, "The wording of the screen": the button itself reads `Copied` for 2 seconds and then reads `Copy` again. 2 seconds is not invented here, it is the hold record 0002's eleventh amendment already fixed for the pill's last words. A clipboard write is otherwise silent, so a press with no answer cannot be told from a button that did nothing. |
| What Copy looks like | AC-15 | `design/registry.md`'s `Secondary button`, the primitive this screen already uses for the older action, chosen by the user on 2026-09-04 over waiting for `/canvas` to draw a new component. It is the same precedent this record already set for the older action: an approved primitive rather than an invented one. `Copy action` is owed a fill-in by `/canvas`, and it is a row correction rather than a drawing. |
| The source application | AC-2 | **Nowhere. It is not stored and this record does not add it.** See The decision. |
| Which dictations match a search | AC-5 | The store, matching `dictation.text` case insensitively against the typed query with SQLite's `LIKE`, the query bound as a parameter with its `%`, `_` and escape characters escaped. Never a string joined into SQL. |
| How many matched | AC-5 | Rust, from a count over the same query and the same account, so the number beside the search and the rows below it come from one moment. |
| How many rows one page holds | AC-7 | This record. 50. Fixed, not a setting. |
| Where the next page starts | AC-7 | The start time and the identifier of the last row already shown, handed back to Rust as a cursor. Never an offset, so a dictation finished while a person is reading cannot make a row appear twice. |
| Whether there are older dictations to show | AC-7 | Rust, from asking for one row more than the page holds and reporting whether it was there. Never the screen comparing a count. |
| The hotkey shown in the empty state | AC-8 | `get_hotkey`, the dictate feature's existing command, invoked by this screen only when the list is empty and no search is typed. |
| What a person reads for that hotkey | AC-8 | `design/registry.md`'s `Hotkey choice`, which already fixes both: the `Keycap` primitive for the one modifier, then the words "Double tap Ctrl" or "Double tap Alt". No new wording. |
| Where the empty state's one action goes | AC-8 | This record: Settings, on Dictation, which is where the hotkey is chosen. The screen asks the dashboard to move the rail; it does not navigate itself. |
| Which way round a dictation reads | AC-11 | The browser, from `dir="auto"` on the element holding the text. Record 0006's decision, unchanged, and no list of right to left languages is held anywhere. |
| The sentence for the one failure | AC-10 | This record, the one row table in The decision. Fixed wording, held in one place in Rust, never carrying anything read off a row. |
| Which refusals carry no sentence | AC-10 | This record: nobody signed in, and the core not being up. Both unreachable from this screen; the precedent is record 0002's fourteenth amendment. |
| Every label and sentence on the screen | AC-5, AC-6, AC-7, AC-8 | This record, "The wording of the screen" below. Settled before any code was written, which is the lesson records 0005 and 0006 learned from their own gates. |
| What a person reads for the rail item | AC-1 | `design/registry.md`, which owns wording. Rust hands out the identifier `history` and nothing else. |
| Which sections the rail holds, and where it lands | AC-1 | Record 0004's fixed list in `src-tauri/src/shell/rail.rs`, as amended by this record to hold `history` first and to land there. |

## Interface surface

One command, in the new feature, on the same terms as every other command in
this project: the interface asks and Rust decides, it takes no account id
because Rust knows it from the session, and it refuses when nobody is signed in.

- `get_history(query, before_started_at, before_id)` returns the page of
  dictations, how many match in total, and whether there are older ones. All
  three arguments may be absent: no query means everything for the account, and
  no cursor means the newest page. It returns one answer for the whole screen,
  for the same reason `get_vocabulary` does: the rows and the count must come
  from one moment, or a person reads "12 dictations match" above eleven of them.

Nothing else. No write command, no delete command, and no command that takes a
row and does something with it, because there is nothing on this screen that
changes anything.

**Amended 2026-09-04 by the first amendment, and the surface is unchanged
except for one field.** `get_history` returns the same shape with a sixth value
on each row, the character count, so no signature changes and nothing new is
invoked. The Copy action adds no command at all: the transcript is already on
the screen, and the web view's own `navigator.clipboard.writeText` puts it on
the clipboard. Still no event in either direction, and
`src-tauri/capabilities/` is still not widened, not added to and not edited.

**No event, in either direction.** This feature emits none and listens to none.
The dashboard's two events are the offline row's and are untouched, and
`src-tauri/capabilities/dashboard.json` is not widened, not added to and not
edited. Record 0004's closed Still open established that Tauri grants no
per-command permission for this application's own commands: what protects
`get_history` is its `not_signed_in` refusal in Rust.

One existing command this screen invokes, and it is another feature's:

- `get_hotkey()`, the dictate feature's, for the empty state's keys only. It is
  named here so that the whole of what this screen invokes is in one list. No
  Rust changes for it, and no door is added.

One change to a file the shell owns, and it changes nothing for the four screens
that exist:

- `src/shell/dashboard.js` passes its own `show` to whichever screen it mounts,
  as a second argument. The history screen's empty state uses it to move the
  rail to Settings, Dictation. The other four take one argument and ignore it.

Errors that matter: the history not reading, which has the sentence in The
decision; and not signed in, and the core not being up, which carry none, for
the reason given there.

## Risk

The plan row is medium weight. The three questions are answered rather than
skipped, because this screen displays the most personal thing EchoScribe holds:
every word a person has ever dictated, in one place, on one screen, searchable.
AGENTS.md's decision rules put personal data in the ask-first column, and this
section is the answer.

**The worst thing a malicious person could do here.** Four things, and the first
is the one that would actually work.

1. **Get script to run inside the dashboard by dictating it.** A transcription
   is text from outside the program (AGENTS.md rule 7), and this is the first
   screen that renders a whole one. Text reading `<img src=x onerror=...>`,
   dictated or spoken by somebody standing behind a person, would run inside the
   dashboard's web view if it were ever set as HTML, and that web view can
   invoke every command this app has. Refused two ways: every value is set with
   `textContent`, and a Rust source guard reads `src/history/history.js` and
   fails the build if `innerHTML`, `outerHTML`, `insertAdjacentHTML` or
   `document.write` appears in it. That guard is not decoration: this screen
   will be edited again, by somebody who wants one bold word.
2. **Get a query of their own into the SQL.** The search text goes into a `LIKE`
   in a real query. It is bound as a parameter, never joined into a string, and
   its wildcards are escaped so a search for `100%` searches for `100%`. Nothing
   in this project builds SQL by joining strings.
3. **Read somebody else's dictations.** Every query is scoped to the signed in
   account in its `WHERE` clause. That is record 0002's AC-18 and this record's
   AC-9, and it is the same protection, and no stronger: anybody with the
   machine and the file has the words. Said plainly rather than dressed up.
4. **Get a transcript into a log or a file.** Nothing here logs a row, on any
   path. A read failure logs its machine reason and nothing else.

**What we are storing that we must protect.** Nothing new. This feature stores
nothing at all. What it *shows* is the most sensitive thing in the app, and the
protection is where it already was: one SQLite file on the person's own machine,
one account per row, never anywhere else. What this record adds to that picture
is a screen, which is a new way for the words to be on display: a history open
on a shared screen shows everything a person has dictated to anyone walking
past. That is inherent in the feature and is not hidden, and it is why the
landing is worth naming as a decision rather than a default.

**What the system should refuse to do, even if asked nicely.**

- Put a transcript anywhere but this screen. No clipboard, no file, no export,
  no request that leaves the machine.
- Render any part of a dictation as markup.
- Build a query by joining a person's search text into SQL.
- Return a row that does not belong to the signed in account.
- Write, change or delete anything in `dictation`.
- Log a transcript, on any path, including a failure.

## Build plan

Two milestones, each leaving the project working.

1. **The store, the read and the one command, with no screen.**
   `src-tauri/src/history/` with its own connection, the paged read with its
   cursor, the account scope, the search with its escaped `LIKE`, the count, the
   one command and its one sentence, and `history::init` after `dictate::init`
   in `lib.rs`. Nothing calls it yet, so nothing a person can see changes.
   Testable in full on its own, which is where this feature's correctness lives:
   the account scope, the escaping, the cursor and the ordering.
2. **The screen, the rail item and the landing.** The History surface, its list,
   its search field, its result count, its older action, its empty state with
   the hotkey keys and the one action, its read failure state, `dir="auto"`
   where a transcript is shown, and the `textContent` guard. Record 0004's rail
   gains `history` first and lands on it, and the rail's "no section that does
   not exist" test loses its one sentinel in the same change, per standing rule
   11, because after this there is no unbuilt section left for it to name.

A third, added 2026-09-04 by the first amendment, after the two above were
built and tested:

3. **The count and the Copy action.** The count field on `DictationView` and
   its `chars().count()`, the count on the row's metadata line beside the
   duration where the comp puts it, the Copy button as the `Secondary button`
   primitive, its `Copied` hold, and one guard that keeps the clipboard write
   to this one place. The pill's chip is the same day's work and lives in
   record 0002's own build plan, because the pill is that record's.

## What this makes harder

- **Two features now read one table, and only one of them owns it.** A column
  added to `dictation` by the dictate feature is a column this feature will not
  know about, and a column removed is a read that breaks. The direction is safe,
  because history never writes, but the coupling is real and it is the first of
  its kind in this project.
- **The landing is now load bearing.** Record 0004 could say its landing was
  temporary and nothing depended on it. From here on, the first thing every
  person sees on every launch is their own history, so anything that makes this
  screen slow or wrong is felt on every launch rather than on a visit.
- **The first thing every new person sees is an empty state**, and it is the
  screen that has to explain the whole product. It carries more weight than any
  other empty state in the app, and it is the one to re-read when the hotkey or
  the sounds change.
- **A long dictation is a tall row.** Nothing truncates, deliberately, so a five
  minute transcript is a screenful. If a real person finds the list unusable
  because of it, the answer is a `/canvas` decision about how a long transcript
  is shown, not a quick clamp in a stylesheet.
- **Search is honest about case only for the ASCII letters.** A search in Greek
  or Cyrillic matches the case it was typed in. Most scripts have no case, so
  this is narrow, and it is exactly the kind of narrow gap that gets rediscovered
  as a bug.
- **`design/registry.md` and this record disagree in three places until
  `/canvas` runs**: the source app, the word count and the Copy action.
  **Two of the three closed on 2026-09-04 by the first amendment**, and they
  closed by the record moving to the comp rather than the other way round: a
  count is on the row, though it counts characters and not words, and Copy is
  built. Both rows are owed a `/canvas` fill-in, and neither is a drawing: the
  count is the comp's own place on the row and Copy is a primitive already on
  the list. The source app is still refused and that row still stands. The rule
  this project applies is that the record wins on content and the comp wins on
  look, and all three are content, but a reader of the registry alone would build
  three things this record does not.
- **A fourth feature now wants the settings error line**, and the question record
  0006 raised is one feature harder to keep deferring.
- **AGENTS.md's data rule now has a visible cost.** A person can see their
  transcript and cannot press a button to copy it. That is the rule working
  rather than failing, and it will look like a missing feature to anybody who
  has not read this record. **No longer true from 2026-09-04**, and what
  replaces it is harder in a quieter way: the data rule now carries a
  distinction, and a distinction has to be applied rather than just obeyed.
  Anybody adding a route for transcribed text has to decide which side of it
  they are on, and the honest answer is not always the convenient one. The two
  worked examples are in the rule's own paragraph in The decision, and record
  0002's clipboard refusal is the one that says no.
- **There is now exactly one clipboard write in this app, and it must stay
  exactly one.** A second one added anywhere, for any reason, is a data rule
  question and not a convenience. The guard the build adds makes a second one
  fail the build rather than trusting the next reader to know that.

## Still open

- ~~**Whether the Copy action may be built.**~~ **Closed 2026-09-04 by the
  first amendment.** The user gave the reading: a person taking their own text
  at their own request is not the app routing it. Copy is built, with no new
  library and no change to `src-tauri/capabilities/`, exactly as this bullet
  said it would need. What the answer left behind is a rule with a distinction
  in it, which is in "What this makes harder".
- **How a person deletes a dictation, or all of them.** Record 0002's data model
  says history is kept "until the person deletes it", and there is no route.
  `design/registry.md` says Copy is the only action on a row, so a second one is
  a `/canvas` decision, and deleting a transcript is the least reversible thing
  this app could do. It needs its own conversation and probably its own record.
- ~~**Whether the word count comes back.**~~ **Closed 2026-09-04 by the first
  amendment**, and closed the first of the two ways this bullet named: the user
  gave a counting rule rather than a dependency. It counts characters, on record
  0005's precedent. One answer did unblock two things, as this bullet said it
  would: the row's count and the pill's chip, the second carried by record
  0002's twentieth amendment.
- **Whether the count should count graphemes rather than characters.** What is
  counted is a Unicode scalar value, so an accented letter typed as a letter
  plus a combining mark counts two and a family emoji counts several. Counting
  what a person would point at needs grapheme cluster segmentation, a new
  dependency and therefore the user's decision, which is the same wall word
  segmentation hit. Named here on 2026-09-04 so that it is a known narrowness
  rather than a bug somebody finds. It is the same shape as this record's
  ASCII-only case folding in search.
- **Whether anything else on a row should be copyable.** Copy takes the
  transcript and nothing else, so a person wanting the time and the language
  with it still selects the row by hand. If that turns out to be what people
  actually want, it is a wording decision about what EchoScribe writes into
  somebody else's document, which is why it was not guessed at.
- **Whether the source application should be captured at all.** Not shown here
  and not stored. If it is ever wanted, it is a column on `dictation`, an
  amendment to record 0002, a change to the capture path in a shipped feature,
  and a conversation about storing which programs a person uses.
- **Whether search should ignore case in every script.** It ignores case for the
  ASCII letters only, which is what SQLite's `LIKE` does without ICU. Fixing it
  means SQLite built with ICU, which is a dependency question.
- **Whether 50 rows a page is right.** A judgement, not a measurement. What
  would settle it is a person with a year of history saying whether they are
  pressing the older action more than they would like.
- **Whether the list should refresh while the dashboard is open.** It does not.
  If a person dictating with History on screen finds it stale often enough to
  complain, the answer is a third event on the dashboard and an amendment to
  record 0004.
- **Whether the settings error line becomes a shared module.** Record 0006's
  question, now with a fourth feature behind it. See the section above.

## The wording of the screen

Settled here rather than left to the build, which is the lesson records 0005 and
0006 learned when `/develop`'s gate stopped them both on the same day: a word a
person reads is not a build detail, and this surface is not in the comp beyond
its shape. All of it is **decided on the user's behalf**. It is the screen's own
static wording, in `src/history/history.js`, and not Rust's, the same division
those two records' screens are on and the opposite of the one refusal sentence,
which is Rust's.

**Two rows were added on 2026-09-04 by the first amendment and they are the
user's own words rather than decided on their behalf.** One sentence already
here was re-read against the Copy action and stands unchanged: the empty
state's "What you say is saved here, on this machine, and nowhere else" is a
claim about what EchoScribe stores, and Copy stores nothing. The clipboard is
on the same machine, it holds nothing until a person asks, and the app puts
nothing in it otherwise, so the sentence is still true in the careful sense it
was written in.

| What | The words | Why these |
|---|---|---|
| The search field's label | "Search your history" | "Search" and not record 0006's "Find", and the difference is the point: that filter narrows rows already on screen, and this asks the store a question and brings back rows that were not. It says whose history, which is AC-9 said quietly, the same job "Your words" does on the vocabulary surface. |
| Whether it has a placeholder | **None.** | It has a real label directly above it. A placeholder repeating the label is noise and vanishes the moment somebody types, which is exactly when a person still wants it. The same answer, for the same reason, as `Term add field`. |
| The result count | "{n} dictations match." At one: "1 dictation matches." At none: "No dictations match." | Read out rather than only repainted, because a search rearranges a list under somebody who may not be able to see it happen. It is shown only while something is typed: with an empty search the number would be a count of everything, which is not a result. Record 0006's three-form count, unchanged in shape, so one pattern means one thing across the app. |
| When nothing matches | "No dictations match." | The same words as the count at zero, deliberately, so a person hears one thing and not two. That is record 0006's own choice for its filter, and the reason to repeat it is that it worked. The search keeps what was typed, so a person corrects a letter rather than starting again. |
| The action that brings older rows | "Show older dictations" | It says both what arrives and that they are older, so a person can tell it from a refresh. It is drawn only when there are older ones, so it never says nothing happened. |
| The empty state's first line | "Nothing dictated yet." | What has not happened, in three words, which is the first thing `design/design-system.md`'s empty state rule asks for. |
| The empty state's paragraph | "Press the hotkey anywhere on this machine, speak, and the words appear where your cursor already is. What you say is saved here, on this machine, and nowhere else." | Two sentences doing the rule's two remaining jobs: where the words will go, and that they stay on this machine. The second sentence is careful in the same way record 0005's caption is: it claims what this app controls, which is that nothing about a dictation is stored anywhere but this machine, and it claims nothing about the service that transcribed it, because a sentence here could not keep that promise. |
| The count on a row | "{n} characters." At one: "1 character." | Added 2026-09-04 by the first amendment, and **this one is the user's own choice**, not decided on their behalf like the rest of this table. The plain word in full, over the comp's abbreviated "84 w", because this project writes plain words everywhere else and both surfaces have the room for it. The same wording on the pill's chip, so one figure reads one way wherever a person meets it. |
| The Copy action | "Copy", then "Copied" for 2 seconds | Added 2026-09-04 by the first amendment, the user's own choice. "Copy" and not "Copy text" or "Copy transcript": there is one thing on the row to copy. "Copied" in the past tense on the button itself, rather than a message somewhere else on the screen, because the answer belongs where the press was. 2 seconds is record 0002's eleventh amendment's hold, reused rather than reinvented. |
| The empty state's one action | "Change the hotkey" | The rule asks for one thing to do next plus one way to change the setup. The thing to do next is pressing the hotkey, which the keys above it already are, and cannot be a button. So the one action is the other half, and it goes to Settings, Dictation, where the hotkey is chosen. |

## Answered at the gate

**Added 2026-09-04, after `/develop`'s gate**, on the same footing as records
0005 and 0006's sections of the same shape. The gate enumerated every value this
build must produce and found three **behaviours** with no named source rather
than three values, each one constrained by an acceptance criterion, and all
three were settled under the gate's option 3 before any code was written. All
three are **decided on the user's behalf**.

**A history with nothing in it draws no search field.** The empty state replaces
the list, the result count and the search field together, and only the surface's
own frame stays. This is the one of the three that changes the shape of the
screen, and it was found by asking what happens when AC-6 and AC-8 are both
true: a person with no dictations at all, typing into a search field. Two states
would then have claimed the same moment, one saying "nothing dictated yet" and
one saying "no dictations match", and a rule picking between them would be a
rule a reader has to find. Not drawing the field removes the collision by
construction, which is better than deciding it. It also happens to be right on
its own terms: a search field over a history that is empty is a control whose
every answer is already known, and the way out of an empty history is the
hotkey, not a query. It is the deliberate opposite of `Term add field`, which
the vocabulary empty state keeps, and the difference is exactly why: there, the
field is the way out.

**A new search starts again at the newest page.** Typing into the search field,
or clearing it, throws away every page already loaded and asks for the first
page of the new question. Only the older action adds to what is on screen, which
is what AC-7 says and all it says. A search is a new question, and answering it
by appending to the answer to a different one would put rows on screen that no
single query produced.

**A query of nothing but spaces is treated as no query at all.** It shows the
whole history, not a search for a space. Nothing is trimmed off a query that has
any other character in it, because a person searching for `the ` may well mean
the trailing space, and the store is asked for exactly what was typed.

## Decided on the user's behalf

Every one of these was `/architect`'s to ask and was decided under the user's
standing instruction for this session. Each is written where it is made, above,
with its reasoning. Each is reversible by an amendment, and the one thing that
would not have been reversible was refused instead and is listed under "What is
refused, and why".

1. That the **source application is not stored and not shown**, which is a
   correction `design/registry.md` is owed for `Dictation row` and the reason
   `Filter chip` is not built.
2. That the **word count is not shown**, because no counting rule this project
   can apply is true in every language it offers. **Reversed by the user on
   2026-09-04**, in the first amendment, and reversed in the way this list
   exists to allow: a count is shown, and it counts characters. Item 2 is the
   one thing on this list a person changed their mind about, which is worth
   leaving visible.
3. That the search is a **real search over the stored text in Rust**, matching a
   case insensitive substring with SQLite's `LIKE`, wildcards escaped, and that
   its ASCII-only case folding is named rather than fixed.
4. **50 rows a page**, and paging **by a cursor** rather than by an offset.
5. That the older rows arrive through the existing `Secondary button` primitive
   rather than a new component.
6. That the list **does not refresh itself** while the dashboard is open, and
   that re-opening History is the route, which is what AC-4 says.
7. That **nothing is truncated**: a dictation's whole text is on its row.
8. That every string from a dictation reaches the screen as **`textContent`**,
   with a Rust source guard that fails the build if that changes.
9. That a transcript carries **`dir="auto"`**, following record 0006.
10. The one sentence a person can read, and its code.
11. That **History goes first in the rail**, as a top level item, and that the
    **landing moves to it**, which requires rewording record 0004's AC-1.
12. That the empty state asks the dictate feature's **existing `get_hotkey`
    command** rather than a new door in Rust, and draws its paragraph without
    keys if that call fails.
13. That the dashboard passes its **`show` to whichever screen it mounts**, so
    the empty state's one action can move the rail.
14. What a person reads for a **time** and for a **duration**: the browser's own
    formatting in the machine's locale, and whole seconds below a minute with
    minutes and seconds above it.
15. Every label and sentence in "The wording of the screen" above: the search
    field's label, that it has no placeholder, the result count in all three of
    its forms, the no-matches sentence, the older action, and the empty state's
    line, paragraph and one action.
16. The three behaviours in "Answered at the gate" above, found by `/develop`'s
    gate on the same day and settled before any code was written: that an empty
    history draws no search field, that a new search starts again at the newest
    page, and that a query of only spaces is no query.

**Nothing in the first amendment is on this list.** Everything it settles was
decided by the user, in the conversation of 2026-09-04, and the two things it
settles are precisely the two this record would not decide alone: a reading of
a hard limit, and a counting rule. That is the list working, not being
bypassed.

## References

- `src-tauri/src/dictate/store.rs`, read 2026-09-04, whose own opening comment
  is the named source for this feature reading `dictation` rather than the
  dictate feature handing it over: "Reading history back is plan row 5's own
  feature and is deliberately not here."
- Record 0002, "dictate with a hotkey", for AC-17, AC-18, the `dictation` table
  and its index, and the thirteenth amendment's meaning of "completed".
- Record 0004, "the app shell", for the rail, the landing destination it says
  this record moves, and the capability question it closed.
- `design/design-system.md`, Empty states, whose four-part rule was written for
  this screen's empty state.

Added 2026-09-04 by the first amendment:

- Record 0005, "teach it your words", The decision, for the character budget
  and the argument that a count of words is not script independent. It is the
  precedent the counting rule rests on, and `src-tauri/src/vocabulary/rules.rs`
  is where `chars().count()` already means what it means here.
- Record 0002, "dictate with a hotkey", its ninth amendment for the clipboard
  refusal this amendment leaves standing, its eleventh for the 2 second hold
  the Copied state reuses, and its twentieth for the pill's chip, which is the
  other half of the same counting rule.
- `src-tauri/src/dictate/transcribe.rs`, read 2026-09-04, for `typed`, the
  string the chip counts and the history row is written from. One string, two
  readings, which is what makes AC-14's "the same figure" checkable rather
  than hopeful.
