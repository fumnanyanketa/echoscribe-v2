# 0004. The app shell

**Status:** In progress
**Date:** 2026-09-02
**Amended:** 2026-09-03. `/develop`'s gate stopped before writing any code,
because deleting `mountSignedIn` would have taken record 0003's AC-14 from
met to unmet with nowhere left for its "working offline" sign to be, and
`design/registry.md` drew no component for one. `/canvas` has since drawn
it. This amendment carries in four things so the rebuild's gate passes:
where the offline sign lives and that record 0003's AC-14 stays record
0003's, proved here; that there is no per-command permission to grant for
this app's own commands, which closes the first Still open question; that
the account block's four values come back from `get_auth_state`, which
already exists; and that the dashboard does listen to two events after all,
which closes the last Still open question and settles what
`dashboard.json` grants. That last one corrects what the gate wrote and was
the user's choice on 2026-09-03. No criterion is renumbered, reworded or
added. The count stays at 7. Evidence:
`docs/evidence/the-app-shell/gate-2026-09-03-blocked.md`.
**Amended:** 2026-09-03, second of the day, by `/architect`. The three
milestones were built the same day and took six readings no record held. All
six are live in the code and visible to a person, and **all six stand; none is
reversed.** Five are recorded here; the sixth, the clearing table, is decided
here and carried into record 0002 by its fifteenth amendment. In order: the
shell learns that a key has changed its invariant from two new dictate events
rather than by asking on a timer; the small window shows the Deepgram key setup
screen when a person is signed in with no key saved, which is what the
invariant already required and had no screen for once `mountSignedIn` was
deleted; closing the dashboard closes the app, because AC-4 is written as
"close the app, reopen it" and there is no tray icon to reopen from; closing
the small window while the dashboard is up hides it rather than destroying it,
because every error screen in the app lives on that window; record 0002's
clearing table gets an owner, which is Rust, and the interface stops holding a
second copy of it; and the account block's initials circle is the raised rail
grey rather than the comp's violet, because `design/design-system.md` says
violet anywhere that is not about a live microphone is a bug. No criterion is
renumbered, reworded or added. The count stays at 7. One cost is carried
rather than fixed and is named in What this makes harder: closing the dashboard
also stops the global hotkey, so a person can turn dictation off across the
whole machine by closing a window.
**Amended:** 2026-09-04, third of the day, by `/architect`, for plan rows 3 and
4 together. **The rail gains two destinations, and this is the amendment The
decision said each of plan rows 3, 4 and 5 would bring.** It is one amendment
for two records deliberately, because both were designed in the same session and
both change the same fixed list: reading two separate amendments to one list
invites each to be taken for the whole change.

- `settings.languages`, for record 0006, "speak in your language".
- `settings.vocabulary`, for record 0005, "teach it your words".

**The rail's order is the comp's**, which `design/registry.md`'s
`Section sub-nav` row already names: Dictation, Languages, Vocabulary,
Transcription. So the two new items go between the two that exist rather than
after them, and the landing destination is untouched: it is still Settings, on
Dictation, and it moves to History when plan row 5 is designed, exactly as The
decision says.

**Nothing else about this record changes.** No new command: both features hand
their screens out through `get_rail`'s existing identifiers, and each has its
own commands in its own feature. No new event, in either direction. **Nothing in
`src-tauri/capabilities/` is added or widened**, and `dashboard.json` still
grants `core:event:allow-listen` and `core:event:allow-unlisten` and nothing
else, because neither new screen listens to anything. `src/shell/rail.js` gains
one wording row each and `src/shell/dashboard.js` gains one entry each in the
table that says which destination opens which screen, both of which are the
shape those two files were written for.

**AC-2 now covers four destinations rather than two**, and it is not reworded:
every item in the rail opens a screen, and after these two records' third
milestones each of the four does. It was recorded as met for `settings` and
`settings.dictation` on 2026-09-04 and as not met for `settings.transcription`
until that half landed; the same standard applies to these two, and neither
identifier is added to the rail before its screen exists. **The count stays at
7** and no criterion is renumbered, reworded or added. Still open's question
"what the Transcription section holds beyond the saved key row" is untouched:
these are two new sections, not additions to that one.
**Amended:** 2026-09-04, fourth of the day, by `/architect`, for plan row 5.
**The rail gains History and the dashboard lands on it.** This is the last of
the three amendments The decision said plan rows 3, 4 and 5 would each bring,
and it is the one this record has been holding a place for since it was written.

- `history`, for record 0007, "see what you've said before". It is a **top
  level item, not a sub-section**, because it has no children, and it goes
  **first**, above Settings. Settings keeps its four children in the comp's
  order, untouched.
- **The landing destination moves from `settings.dictation` to `history`.**
  Value sourcing below said in as many words that it "moves to History when plan
  row 5 is designed, by an amendment carried in that record and here". This is
  that.

**AC-1 is reworded, and it is the first acceptance criterion in this project to
be.** Its tail read "and Settings open on Dictation", which stops being true the
moment this lands. Nothing is renumbered and nothing is added, **the count stays
at 7**, and only that tail changes. It is legitimate because this record wrote
down in advance that the sentence was temporary, and it is called out here
because every amendment before this one could honestly end with "no criterion is
renumbered, reworded or added", and this one cannot.

**Nothing else about this record changes.** No new command: History hands its
screen out through `get_rail`'s existing identifiers and its own feature holds
its own command. No new event, in either direction. **Nothing in
`src-tauri/capabilities/` is added or widened**, and `dashboard.json` still
grants `core:event:allow-listen` and `core:event:allow-unlisten` and nothing
else. `src/shell/rail.js` gains one wording row and `src/shell/dashboard.js`
gains one entry in the table that says which destination opens which screen,
both of which are the shape those two files were written for. One thing is added
to `dashboard.js` that is not: it passes its own `show` as a second argument to
whichever screen it mounts, so record 0007's empty state can move the rail to
Settings, Dictation. The four screens that exist take one argument and ignore
it.

**AC-2 now covers six destinations rather than four**, and it is not reworded.
`rail.rs`'s `the_rail_holds_no_section_that_does_not_exist` guard loses its one
sentinel in the same change, per standing rule 11, because after this there is
no unbuilt section left for it to name; the exhaustive list check above it is
what keeps the rail honest from then on.
**Weight:** medium
**Plan row:** none of its own. It serves rows 1 to 5, and row 2's settings
screen is the first thing that cannot be built without it.
**Supersedes:** nothing

## In one line

EchoScribe gets a second window, the 1200x800 dashboard with the dark rail
and the white reading surface, alongside the 760x540 dark window that
already holds sign in, first run and every error; the dashboard opens on
Settings, its rail holds only sections that work, and it remembers the size
and place you left it. It buys a route into Settings, which four acceptance
criteria in record 0002 have been waiting on, and it costs a third window,
a third capability file, and a remembered position that has to survive a
monitor being unplugged.

## What this is for

Record 0002 promises that Settings shows the saved key, offers the two
hotkeys and holds the sound switch. None of that is reachable: every screen
built so far is a 760x540 dark whole window interruption, and the white
reading surface the design system locks has never been built. This record
decides the window it lives in, what the rail holds while most of its
destinations do not exist yet, and what a signed in person sees when
nothing is wrong.

It exists as its own record rather than as a fourteenth amendment to 0002
because none of it belongs to dictation. The window carries sign in and
first run too, and the rail is where plan rows 3, 4 and 5 will each add an
item.

## Acceptance criteria

- **AC-1**: Signed in with a Deepgram key saved, EchoScribe shows the
  dashboard without me clicking anything: a dark rail down the left, a
  white reading surface on the right, and History open. **Reworded
  2026-09-04** by the fourth amendment of that day, for record 0007. Until then
  it ended "and Settings open on Dictation", which this record always said was
  temporary.
- **AC-2**: Every item in the rail opens a screen. There is no item that
  does nothing when I click it, and no item that is greyed out.
- **AC-3**: The dashboard opens at 1200x800 the first time. I can resize
  it, down to 960x640 and no smaller.
- **AC-4**: I resize the dashboard and drag it somewhere, close the app,
  reopen it, and it comes back the same size in the same place on the same
  screen.
- **AC-5**: If the screen I left the dashboard on is gone, or the size I
  left it at no longer fits, it still opens wholly on screen at a size that
  fits. It never opens part way off an edge and never opens under the
  taskbar.
- **AC-6**: When something goes wrong while the dashboard is open, the
  small dark window comes forward carrying it, and the dashboard behind is
  not resized, not moved and not closed. When that error clears itself, the
  small window goes and the dashboard is there exactly as it was, and it is
  not brought to the front.
- **AC-7**: Signing out closes the dashboard and leaves me on the sign in
  screen. A second account signing in on the same machine gets its own
  dashboard size and place, never the first account's.

## The decision

**Two windows, and each one keeps the job it is shaped for.** The 760x540
non resizable dark window that exists today is kept exactly as it is, and
it stays the window for everything that has one way forward: sign in, first
run, the Deepgram key setup screen, the microphone error screen and the mid
dictation Deepgram error screen. A second window is created for the
dashboard, at the comp's Surface 2 geometry, holding the dark rail and the
white reading surface. This was the user's choice on 2026-09-02, over one
window resizing between the two modes.

**Only one of them is ever the window at rest.** The small window is shown
when, and only when, there is a pre shell state, meaning nobody is signed
in or no Deepgram key is saved yet, or there is an interruption that record
0002 or record 0003 names. Otherwise it is hidden and the dashboard is what
is on screen. So the invariant that everything below leans on is: **while a
person is signed in with a key saved, the dashboard window exists.**

**What happens when an error clears, which is AC-32 of record 0002 read
again.** AC-32 says that when an error clears itself the window "does not
come to the front, does not hide itself, and does not move". That sentence
was written when one window held both jobs, and with two windows the
faithful reading is that the small window hides and the dashboard, which by
the invariant above already exists, is revealed where and as it was.
Nothing is raised, nothing is resized, nothing moves, and the person is
never left looking at an empty dark window they now have to close. That is
what an ordinary Windows alert window does. AC-32 is not reworded, and
record 0002's fourteenth amendment records this reading, so that a person
reading AC-32 finds it.

**Signing out destroys the dashboard.** It is not hidden and kept. The
dashboard shows one account's settings and remembers one account's window
place, and a window kept alive across a sign out is a way for the second
account on a machine to see the first one's, which record 0002's AC-18
forbids in spirit. Rust closes it and shows the small window on the sign in
screen.

**The rail holds only sections that work.** Today that is Settings, with
Dictation and Transcription beneath it, the brand lockup at the top and the
account block pinned at the foot. History, Languages and Vocabulary are not
in the rail at all, not even greyed out. Two reasons, and the first is the
harder one: `design/registry.md` draws `Nav item` with exactly two states,
resting and active, so a disabled third state does not exist in this design
system and inventing one during a build is forbidden. The second is
plainer: a nav item that opens nothing is a promise the app cannot keep.
This means the rail ships holding one nav item and looks sparse, which is
honest, and it means each of plan rows 3, 4 and 5 amends this record to add
its own item as it lands.

**Corrected 2026-09-04 by the third amendment of that day.** Two of the three
have landed. Languages and Vocabulary are in the rail now, for records 0006 and
0005, in the comp's order: Dictation, Languages, Vocabulary, Transcription.
History is still not in the rail at all, not even greyed out, and every word of
the reasoning above still applies to it and to anything else unbuilt. The rule
that made this correction possible is the one this paragraph states: an item
joins when its screen exists, and neither of the two was added before its own
third milestone.

**Corrected again 2026-09-04 by the fourth amendment of that day.** All three
have landed. History is in the rail, for record 0007, as a top level item above
Settings, and it joined the same way the other two did: with its own screen, in
its own milestone, and not before. **The rail now holds every section this app
has**, so the sentence above about what a rail built today may show has nothing
left to exclude. The rule itself is not retired and must not be: the next
feature with a screen adds its item when the screen exists, and nothing is ever
greyed out, because `design/registry.md` still draws `Nav item` with exactly two
states.

**Nothing here decides that History, Languages or Vocabulary exist, or
when.** Plan rows 3, 4 and 5 already say they are being built and in what
order, and that is `/scope`'s to hold. What this record settles is only
what a rail built today may show, which can be answered without touching
any of them.

**The landing destination is Settings, on Dictation**, and it is temporary
by design. It is the only section that exists, so it is the only honest
landing. The comp's answer, the History empty state, is right and is plan
row 5's to deliver: when that row is designed, its record moves the landing
and amends this one. Nothing here assumes History is built, and nothing
here quietly becomes wrong when it is.

**Moved 2026-09-04 by the fourth amendment of that day. The landing is History,
and it is no longer temporary.** Record 0007 designed plan row 5 and did exactly
what the paragraph above said it would: it moved the landing and amended this
record, and AC-1 was reworded with it. The comp's answer turned out to be right,
which is the whole reason this was written as a placeholder rather than as a
choice. Two things that were true of the old landing are now true of the new one
and are worth carrying: the dashboard still does not remember which section you
were last on, and the landing is still a fixed value in `rail.rs` rather than a
stored one.

**The dashboard does not remember which section you were last on.** It
opens on the landing destination every time. Record 0002's AC-19 and AC-21
promise that the two dictation settings survive a restart, not that the
window's place in the app does, and a remembered section would make the
landing decision impossible to observe.

**The dashboard remembers its size and its place**, per account, chosen by
the user on 2026-09-02. The shape of that memory is taken from the pill
rather than invented: the size in logical pixels, and the place as two
fractions locating the window's centre within the working area of a screen,
exactly as `pill_x` and `pill_y` do in record 0002 and for the same reason.
A fraction survives a resolution change where a pixel does not. Alongside
them the name Windows gives the screen it was left on, as a best effort. On
open, Rust looks for a screen of that name among the ones Windows reports;
if there is none it uses the primary screen. Then it places the window at
its fractions within that screen's working area and clamps the whole
rectangle inside, shrinking the remembered size to fit if it no longer
does, but never below the floor unless the working area itself is smaller,
in which case the window is the working area. That is AC-5, and it is the
same clamp that makes AC-26 hold for the pill.

**Rust owns the geometry and the interface never sees a coordinate.** Rust
creates, sizes and positions the dashboard before it is shown, and reads
its size and position back from Windows when the person has finished moving
or resizing it. This is the pill's own precedent after the AC-27 fix: there
is no command through which the interface reports a position, because it
never has one.

**Where the code lives.** A new feature folder on both sides, because the
shell belongs to no existing feature and AGENTS.md forbids adding it to one
that seems related: `src-tauri/src/shell/` for the two windows, the
geometry and the one table, and `src/shell/` for the dashboard's own page,
the rail and the surface. `src/main.js` stays what it is, the router for
the small window. `src/sign-in/sign-in.js`'s `mountSignedIn`, the stub that
says "Signed in as" with a Sign out button, is deleted in the same change
that lands the rail, per standing rule 11. **It holds two things, not one,
and both move in that change.** Its Sign out moves to the account block,
and so does its "Working offline" line, along with the
`.signed-in__offline` rule in `src/sign-in/sign-in.css` that styles it.

**The offline sign moves into the account block, and record 0003's AC-14
stays record 0003's criterion.** Added 2026-09-03. `mountSignedIn` is the
only screen that sign has ever been on, and by the invariant above the
small window is hidden whenever a person is signed in with a key saved, so
deleting the stub would take AC-14 from met to unmet with nowhere left to
put it. `design/registry.md`'s `Offline row`, drawn 2026-09-03, is its new
home and owns everything about how it looks and what a person reads: an
amber `OFFLINE` badge and the sentence "Your sign-in could not be checked,
so this is your account from last time.", first in the account block's
reading order, above the initials and the name. The block is pinned to the
rail foot, so the row's arrival grows the block upward and nothing already
on screen moves. No new criterion is added here for it. AC-14 belongs to
the record that promised it, and milestone 3 below is where it is proved,
exactly as record 0002's AC-32 stayed record 0002's and is proved by
milestone 1 of this record.

**A third capability file**, `src-tauri/capabilities/dashboard.json`,
scoped to the dashboard window's label alone. **Corrected 2026-09-03**, and
what it grants is now settled rather than left to the build. It grants
`core:event:allow-listen` and `core:event:allow-unlisten`, which is
`pill.json`'s list exactly, and nothing else. Those two are for the offline
row, which has to arrive and go while the dashboard is open. There is
nothing to grant for the three commands the dashboard invokes: Tauri's
permission system covers its own plugin commands, and this application's
commands appear nowhere in `src-tauri/gen/schemas/`, so no per-command
grant exists to give or to withhold. What stops the dashboard reaching a
command it should not is the `not_signed_in` refusal already in Rust on
every command on this surface. That is the real control and it is already
built; this file is the second line, not the first. It is **not** granted
`core:default`, and it is granted nothing that lets a web view move,
resize, close or otherwise touch any window: the person resizes the
dashboard through its own title bar, which is Windows doing it, and Rust
reads the result. `pill.json` is the shape to copy, not `default.json`.

**The shell is told when a key changes the invariant, and does not ask.** Added
2026-09-03 by the second amendment. The invariant above turns on two things, and
one of them can change while the app is running: a person with no key saved
pastes one, or a saved key is cleared. Without a signal the shell only learns at
the next launch, so saving a first key leaves a person looking at a blank dark
window until they restart. The dictate feature emits two events for it,
`dictation:key_saved` when Deepgram has accepted a key and the row is written,
and `dictation:key_cleared` when the key and its row are gone. Rust listens to
both, alongside `auth:signed_in` and `auth:signed_out`, and settles the windows
from all four. **Neither event crosses into any web view and neither widens any
file in `src-tauri/capabilities/`.** They are Rust talking to Rust. The dashboard
still listens to two events and only two, `auth:offline` and `auth:signed_in`,
exactly as the first amendment settled. Both carry an empty payload, not even
the key's last four characters, because nothing listening needs them. A timer
asking `get_deepgram_key_info` was rejected for the same reason the offline
row's timer was on 2026-09-03: a repeating question to Rust is a worse thing to
own than a listener, and it cannot be prompt.

**Signed in with no key saved, the small window shows the key setup screen.**
Added 2026-09-03 by the second amendment. This is not a free choice, it is what
the invariant already required. The small window is shown whenever there is a
pre shell state, and "no key saved yet" is named above as one of them, so that
state owes a screen. Until this build the screen there was `mountSignedIn`, the
"Signed in as" stub, which this record deletes. The key setup screen is the only
other thing that state can honestly show: it is the one thing standing between
the person and a working app, it already exists, and record 0002's AC-9 already
puts them on it when they press the hotkey with no key. So a person who restarts
before they have a key lands where they left off rather than on a stub that told
them nothing. Nothing new is built for it and no criterion is added: it is
`src/main.js` mounting a screen it already had, in a state that used to mount the
stub.

**Closing the dashboard closes the app.** Added 2026-09-03 by the second
amendment. There were only ever two candidates and one of them is not available.
Hiding the dashboard and leaving the app running needs somewhere to bring it back
from, and there is no tray icon in this project and none drawn in
`design/registry.md`, so hiding would leave a person with a running app they
cannot see and cannot reach. Destroying the window without exiting breaks the
invariant, and the shell would immediately build it again, so the close button
would visibly do nothing. Exiting is the only reading that leaves AC-4's own
words, "close the app, reopen it", meaning what they say. The window's size and
place are written first and the exit waits for that write, so the geometry a
person just chose is not lost by the very thing that ends the app. **This has a
cost and this record does not pay it**: exiting stops the global hotkey, so
closing a window turns dictation off across the whole machine. See What this
makes harder.

**Closing the small window while the dashboard is up hides it, and clears
whatever it was carrying.** Added 2026-09-03 by the second amendment. The small
window is not one screen, it is the home of every error screen in the app, so
destroying it would leave the next microphone failure or Deepgram ending with
nowhere to be read. Rust refuses the close and hides the window instead. It also
forgets the interruption, because that is what a person dismissing an error
window means: they have read it and want it gone. The problem is not thereby
solved and nothing pretends it is. If the microphone is still blocked, the next
hotkey press brings the window straight back with the same screen, which is
record 0002's behaviour already and not a new promise. When the small window is
all there is, meaning nobody is signed in or no key is saved, closing it still
ends the app exactly as it always did: there is no dashboard behind it, so there
is nothing to reveal and nothing to keep running for.

**Record 0002's clearing table has an owner, and it is Rust.** Added 2026-09-03
by the second amendment, and this was the user's choice that day. The build gave
the table two readers, which was right, and two copies, which was not. Rust
needs it because this record puts the question "is the small window on screen"
in Rust, where AGENTS.md says a decision belongs, rather than leaving it to a
screen choosing not to ask. The interface needs to know which of the three
screens to draw. Neither reader can be deleted. So the table stops being copied
and starts being handed over: **Rust classifies an error kind into its screen
once, and `dictation:error` carries that answer as a field beside the kind, the
code, the message and the action.** The interface mounts the screen it is named
and no longer inspects a kind at all. `MIC_ERROR_KINDS` in
`src/dictate/mic-error.js`, `isDeepgramErrorKind` in
`src/dictate/deepgram-error.js` and the kind matching in `src/main.js` are all
deleted in the same change, per standing rule 11. The classifier lives in the
dictate feature, where the kinds are minted, and the shell reads it through one
named function; the shell already depends on the sign in feature this way, so
the direction is one the codebase already has. What is left over is each
family's pairing with its own proof, which stays on both sides because each side
clears the thing it is itself holding, and that residue is held together by a
source guard of the shape already in `src-tauri/src/dictate/mod.rs`: a Rust test
that reads `src/main.js` and fails the build when the two stop agreeing. Record
0002's fifteenth amendment carries all of this, because the table is that
record's.

**The account block's initials circle is the raised rail grey, never violet.**
Added 2026-09-03 by the second amendment. `design/registry.md`'s `Account block`
row names initials, a name, a mail address and a date, and no colour, so the
build had to choose one, and that choice is a decision rather than typing. It is
`--color-rail-active`, the same raised fill an active nav item uses, with
`--color-surface` initials on it. Violet was refused by
`design/design-system.md`'s own rule: violet is one channel that only ever means
a live microphone, and "if violet appears anywhere that is not about a live
microphone, that is a bug". The registry already reaches the same conclusion
twice for the same reason, at `Sound switch` and at `Waiting indicator`. The
brand lockup's violet dot at the head of the same rail is the registry's one
standing exception and is not reopened here. **The contrast pair is already
audited**: `design/check-contrast.py` checks `--color-surface` on
`--color-rail-active` as text, under the name "Active nav item", so what
`/canvas` owes is a registry row naming the fill, not a new contrast pair.

## What else was considered

| Option | Why not |
|---|---|
| One window, resized only when it changes mode | My own recommendation, and the user chose otherwise on 2026-09-02. It kept "the EchoScribe window" naming one thing, which is how AC-28, AC-30 and AC-32 are all written, and it needed no new capability file. Its cost was that the small dark interruption screens would have been drawn on a 1200x800 canvas once the dashboard had been reached, which is not what the comp draws. |
| One window, resized for every screen | A microphone error arrives while the person is dictating into another app, so this resizes the window behind their back, and clearing the error resizes it a second time. AC-32 exists to stop exactly that kind of movement. |
| The small window stays on screen after an error clears | The literal reading of AC-32's "does not hide itself". It leaves a person with an empty dark window in the taskbar to deal with, having done nothing wrong. |
| The small window hides and the dashboard is raised | AC-32 refuses it outright. The person is dictating into another app and a window taking focus costs them their place, which is the whole reason AC-27 and AC-32 exist. |
| Exactly 1200x800, never resizable, like the small window | Cheapest to build and to get right, because the layout only ever has to be correct at one size. A screen 768 pixels tall cannot show it at all, and this app is for people who work in other windows all day. |
| `tauri-plugin-window-state` for the remembered geometry | A new library, and it fails this project's own data rules before the question of whether to install it even arises: it writes window state to a JSON file in the app config directory, per window label, with no notion of an account. AGENTS.md requires every stored record to belong to exactly one signed in account. |
| Remembering the size only, not the place | My own recommendation, and the user chose size and place on 2026-09-02. Remembering a place is the part that went wrong twice for the pill: AC-24 and AC-26 exist because a remembered position becomes a lie when a monitor is unplugged or its resolution changes. Choosing to remember it means owing that same live proof again, which the build plan below now carries. |
| The rail showing all six destinations, unbuilt ones greyed out | Needs a disabled `Nav item` state that `design/registry.md` does not draw, so it would block record 0002's milestone 5 on another `/canvas` pass, and it shows a person five destinations of which two work. |
| Landing on a dashboard home screen | Nothing like it is drawn, so it hands milestone 5 back to `/canvas`, and it repeats what the first run Ready confirmation already said to the same person minutes earlier. |
| Waiting for History and landing there | What the comp draws, and it blocks the settings screen behind plan row 5, which has no decision record at all. AC-12, AC-19, AC-21 and AC-22 would stay unmet indefinitely. |
| Remembering which section the person was last on | Nothing has asked for it, it is a second stored value per account, and it would make AC-1's landing impossible to observe. |
| Asking `get_deepgram_key_info` on a timer instead of the two key events | Added 2026-09-03. Refused for the reason the offline row's timer was refused the same day: a repeating question to Rust is a worse thing to own than a listener, and a timer cannot be prompt, so a person who has just pasted their first key waits out an interval in front of a window that is still wrong. |
| Closing the dashboard hides it and leaves the app running | Added 2026-09-03. Needs somewhere to bring it back from, and there is no tray icon in this project and none drawn. A running app a person can neither see nor reach is worse than one that closed. |
| Closing the dashboard destroys the window without exiting | Added 2026-09-03. Breaks the invariant, so the shell rebuilds the window at once and the close button visibly does nothing. |
| Closing the small window destroys it while the dashboard is up | Added 2026-09-03. Every error screen in the app lives on that window, so the next microphone failure would have nowhere to be read. |
| Leaving both copies of the clearing table and adding only a source guard | Added 2026-09-03, offered to the user and not chosen. It changes no behaviour and lands with no rework, and the guard shape already exists in `dictate/mod.rs`. It catches the two lists of strings drifting apart. It does not catch the two sides coming to genuinely different conclusions, which is the failure that actually costs a person a wrong window. |
| Rust owning the clearing half too, through a new clearing event | Added 2026-09-03, offered to the user and not chosen. It would leave the table with nothing at all on the interface side. It costs a new event, which record 0002's clearing paragraph went out of its way to say it did not need, and one more hop between Rust and the screen for a residue that a source guard already holds. |
| The interface owning the clearing table, with Rust asking it | Added 2026-09-03, and not offered, because this project's own rules forbid it. Rust would need the interface to tell it whether to keep a window on screen, and both AGENTS.md and this record's Risk section put that decision in Rust rather than in a screen choosing not to ask. |
| The initials circle in the comp's violet | Added 2026-09-03. `design/design-system.md` reserves violet for one meaning, a live microphone, and calls violet used for anything else a bug. An account's initials are not a microphone. |

## Data model

One new table, in the same SQLite file as everything else, because
AGENTS.md holds all persistent data in one file on the person's own
machine. The `account` table is the sign in feature's, plan row 1, and is
referenced here.

| Table | Key | Fields | Relationship |
|---|---|---|---|
| `shell_window` | `account_id` | `width` integer, required, logical pixels. `height` integer, required, logical pixels. `center_x` real, required, defaulting to 0.5. `center_y` real, required, defaulting to 0.5. `screen_name` text, may be empty. `updated_at` text, required. | One account has zero or one |

Rules that must always hold:

- `account_id` is never empty. Nothing is written until we know whose it is.
- `center_x` and `center_y` are fractions between 0 and 1 giving where the
  centre of the window sits within the working area of a screen. They are
  not pixels, and they are read as nonsense rather than as instructions:
  anything outside that range, or unreadable, is treated as the default,
  exactly as `pill_x` and `pill_y` are in record 0002.
- `width` and `height` are clamped on read to no smaller than the 960x640
  floor and no larger than the chosen screen's working area. A row holding
  a size that cannot be shown produces a usable window, never an unusable
  one.
- `screen_name` may be empty, and a name that matches no screen Windows
  reports is not an error. The primary screen is used.
- Nothing about what a person said, no transcript, and no secret of any
  kind is ever in this table. It holds four numbers and a screen name.
- Deleting an account deletes this row too. Record 0002 says deleting an
  account deletes all three of its rows and its Credential Manager entry;
  this is a fourth row.
- The row is written when the person has finished moving or resizing the
  window, not on every pixel of a drag.

One migration, creating one table. The shell opens its own connection to
the same file, the way the dictate feature's store does beside sign in's.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The signed in account id | AC-1, AC-4, AC-7 | The Clerk session held in the Rust core, per record 0003. Never passed in from the interface. |
| Whether the dashboard may be shown at all | AC-1, AC-7 | Two things Rust already knows: the auth state from record 0003, and the presence of a `deepgram_credential` row for the account, which is record 0002's own source for whether a key exists. |
| Which sections the rail holds | AC-2 | This record. A fixed list, one entry per feature that is built: History first, then Settings with Dictation, Languages, Vocabulary and Transcription beneath it, in that order, which is the comp's. Handed to the interface by Rust, never a list the interface holds, for the same reason `get_hotkey` hands out the two hotkeys: a screen must not be able to invent a destination. A section joins the list by an amendment to this record when its own feature record is written. **Languages and Vocabulary joined on 2026-09-04** by the third amendment of that day, for records 0006 and 0005, and **History joined the same day** by the fourth, for record 0007. |
| What a person reads for each rail item | AC-2 | `design/registry.md`, which owns wording. Rust hands out identifiers, not labels, exactly as `get_hotkey` hands out stored values and the screen decides what a person reads. |
| The landing destination | AC-1 | This record. **History, from 2026-09-04**, by the fourth amendment of that day, for record 0007. Until then it was Settings, on its Dictation sub-section, and this row always said it moved to History when plan row 5 was designed. It did, by an amendment carried in that record and here, and AC-1 was reworded with it. |
| Whether the dashboard remembers the last section | AC-1 | This record. It does not. The landing destination is where it opens, every time. |
| The dashboard's size on a first ever open | AC-3 | This record. 1200x800 logical pixels, the comp's Surface 2 geometry, centred on the screen holding the focused window and clamped inside its working area. |
| The smallest the dashboard may be | AC-3, AC-5 | This record. 960x640 logical pixels. Fixed, not a setting. A judgement rather than a measurement; see Still open. |
| The dashboard's remembered size | AC-4 | `shell_window.width` and `height`, read back from Windows by Rust when the person finishes resizing. |
| The dashboard's remembered place | AC-4 | `shell_window.center_x` and `center_y`, fractions of the working area of the screen it was left on, converted by Rust from the position Windows reports when the person finishes moving it. The interface never sees a coordinate and has no command through which to send one. |
| Which screen the dashboard opens on | AC-4, AC-5 | `shell_window.screen_name`, matched against the screens Windows reports at the moment the window opens. The primary screen when there is no match, no stored name, or Windows cannot say. |
| The working area of that screen | AC-3, AC-5 | Asked of Windows for the chosen screen each time the window opens. Same source and same reason as record 0002's pill rows: the working area excludes the taskbar wherever the person keeps it, which is what makes AC-5's "never under the taskbar" hold. |
| When the small dark window is shown | AC-1, AC-6 | This record. When, and only when, there is a pre shell state, nobody signed in or no key saved, or an interruption record 0002 or 0003 names. Otherwise it is hidden. |
| What the small window does when an error clears itself | AC-6 | This record, and it is record 0002's AC-32 read for two windows. It hides, revealing the dashboard where and as it was. Nothing is raised, resized or moved. The dashboard is always there to reveal, by the invariant in The decision. |
| What happens to the dashboard on sign out | AC-7 | This record. Rust closes it and shows the small window on the sign in screen. Not hidden and kept. |
| Which account's window place is used | AC-7 | The `shell_window` row for the signed in account. A second account on the machine has its own row or none, and none means the defaults. |
| The account block's initials, name, mail address and signed-in date | AC-2, and record 0003's AC-5 and AC-7 | `get_auth_state()`, which already exists and already returns all four on its `AccountView`, and never a token. Added 2026-09-03. No new command, no new stored value, no widened capability. |
| Whether the offline sign is showing | Record 0003's AC-14 | `get_auth_state()` at open, which returns `signed_in_offline` when this launch has had no successful refresh yet, per record 0003. While the dashboard is open it changes on two events Rust already emits from `sign_in/renewal.rs`: `auth:offline` when a refresh cannot reach Clerk, and `auth:signed_in` when a later one gets through. Nothing is owed in Rust. Added 2026-09-03. |
| What a person reads in the offline sign | Record 0003's AC-14 | `design/registry.md`'s `Offline row`, which fixes the badge and the sentence at design time. The interface holds no wording of its own here, and Rust hands out a state, not a sentence, the same division as the rail's items above. Added 2026-09-03. |
| Where the offline sign sits | Record 0003's AC-14 | `design/registry.md`'s `Offline row`: the account block at the rail foot, first in the block's reading order. Added 2026-09-03. |
| When the shell learns a key has been saved or cleared | AC-1 | `dictation:key_saved` and `dictation:key_cleared`, emitted by `src-tauri/src/dictate/deepgram_key.rs` and listened to in Rust only. Added 2026-09-03 by the second amendment. Never a timer, and never asked of the interface. Both carry an empty payload and neither reaches a web view, so neither widens a capability file. |
| What the small window shows when a person is signed in with no key saved | AC-1 | This record, second amendment. The Deepgram key setup screen, which record 0002's AC-9 already owns and already draws. Not a new screen and not a new criterion: it is the screen the pre shell state above always owed, and it replaces the deleted `mountSignedIn` stub. |
| What closing the dashboard does | AC-4 | This record, second amendment. It writes the window's size and place, then exits the app. There is no tray icon to hide into, and the invariant would rebuild a window merely destroyed. |
| What closing the small window does while the dashboard is up | AC-6 | This record, second amendment. Rust refuses the close, hides the window, and forgets the interruption it was carrying. Every error screen lives on that window, so it is never destroyed while there is an app to show one. With no dashboard behind it, closing it ends the app as it always did. |
| Which screen a `dictation:error` kind lands on | AC-6, and record 0002's AC-28, AC-30, AC-32 | Rust, once, in the dictate feature where the kinds are minted, carried to the interface as a field on `dictation:error`. Added 2026-09-03 by the second amendment and by record 0002's fifteenth. The interface no longer inspects a kind, and the shell reads the same classifier through one named function, so record 0002's clearing table has exactly one copy. |
| The fill behind the account block's initials | AC-2 | This record, second amendment. `--color-rail-active`, with `--color-surface` initials. Not violet: `design/design-system.md` reserves violet for a live microphone. The pair is already audited in `design/check-contrast.py` as "Active nav item". `design/registry.md` owes a row naming the fill; it owes no new contrast pair. |

## Interface surface

Three commands, **corrected from two on 2026-09-03**, all on the same terms
as every other command in this project: the interface asks and Rust
decides, none takes an account id because Rust knows it from the session,
and all refuse when nobody is signed in. Only the first is new.

- `get_rail()` returns the sections that exist, as identifiers, and which of
  them is the landing destination. The interface renders what it is given
  and cannot add a destination, which is AC-2 as a rule in Rust and not
  only a rule in the design. It returns no wording. This is the one new
  command in this record.
- `get_auth_state()` already exists, from record 0003. The account block
  calls it for the four things it shows, the initials, the name, the mail
  address and the signed-in date, all of which come back on its
  `AccountView`, and for whether the offline row is showing, which is the
  difference between its `signed_in` and `signed_in_offline` states. It
  never returns a token. Added to this list on 2026-09-03: it was always
  the only route those five values could travel, and leaving it unnamed
  invited a reader to think the account block invents its contents.
- `sign_out()` already exists, from record 0003. The account block's Sign
  out calls it and nothing else. It is named here so that the whole of what
  the dashboard invokes is in one list. It is **not** named because a
  capability has to allow it: as the capability paragraph above now records,
  there is no per-command grant for this application's own commands.

**Two events, and they are the only two the dashboard listens to.** Added
2026-09-03, and this is what closes the last question in Still open.
`auth:offline` and `auth:signed_in`, both already emitted by
`src-tauri/src/sign_in/renewal.rs`, are what make the offline row arrive
and go while the dashboard is open. Nothing is owed in Rust for either. The
dashboard emits nothing, ever. Its capability grants
`core:event:allow-listen` and `core:event:allow-unlisten` for this and for
nothing else, which was the user's choice on 2026-09-03 over asking
`get_auth_state()` again on a timer: a repeating question to Rust is a
worse thing to own than a listener, and a timer cannot honour the registry
row's promise that the sign goes the moment a refresh succeeds. Adding a
third event stays an amendment, for the reason Still open gave: it widens a
capability file.

There is deliberately **no command for the window's size or position**. Rust
positions and sizes the dashboard itself before showing it, and reads the
result back from Windows afterwards. Nothing about the geometry crosses
into the interface as a number, in either direction. This is the pill's
precedent after the AC-27 fix, written down here so nobody adds a
convenience getter later.

Which section is showing is the dashboard's own business and needs no
command and no event. It is drawing, not a decision.

Errors that matter: not signed in, which all three commands refuse; and the
rail being unreadable, which cannot happen, because the list is fixed in
this record rather than stored.

**Four events reach Rust, and they are not the dashboard's two.** Added
2026-09-03 by the second amendment. The paragraph above is about what a web view
listens to and is unchanged: the dashboard listens to `auth:offline` and
`auth:signed_in`, and to nothing else. Separately, and inside Rust, this feature
listens to four signals that change whether the dashboard may exist at all:
`auth:signed_in` and `auth:signed_out` from the sign in feature, and
`dictation:key_saved` and `dictation:key_cleared` from the dictate feature. The
last two are new, added by this amendment, and they exist because the invariant
can change while the app is running. Rust listening to Rust needs no permission
and grants none: **none of these four widens `src-tauri/capabilities/`, and
`dashboard.json` is untouched by this amendment.** The rule that a third
*dashboard* event is an amendment still stands, for the reason Still open gave,
and it is about web views rather than about events in general.

Rust also listens to `dictation:error`, `dictation:needs_key`,
`dictation:opened` and `dictation:text`, which record 0002 already emits and
already broadcasts to every window. Nothing is owed in the dictate feature for
any of them except the classification field the fifteenth amendment adds to
`dictation:error`.

## Risk

This record touches sign in and it widens `src-tauri/capabilities/`, so the
three questions are answered rather than skipped. They are answered from
AGENTS.md's data rules and record 0002's Risk section rather than newly
asked of the user, because this record stores no personal data: it holds
four numbers and a screen name.

**The worst thing a malicious person could do here.** Reach a window that
can do more than draw. A capability file is the only thing standing between
a web view and the Windows APIs Tauri exposes, and the cheapest mistake to
make here is copying `default.json` for the new window because it works.
That is why the dashboard is denied `core:default` by name above, and why
`pill.json` is named as the shape to copy. Nothing else in this record is
reachable from outside: the geometry never crosses into the interface, and
the rail's contents are a literal in Rust.

**Added 2026-09-03, because the shape of that protection is not what this
section first assumed.** It said the dashboard is granted the narrowest set
that lets it call the commands under Interface surface. There is no such
set. Tauri grants permissions for its own plugin commands, and this
application's commands are not among them, so a capability file can neither
allow nor forbid `get_rail`, `get_auth_state` or `sign_out`. The control
that actually holds is the `not_signed_in` refusal in Rust on every one of
them, which is already built and is checked against the session rather than
against which window asked. The capability file still matters, for exactly
the thing this paragraph opened with: it is what keeps the dashboard from
reaching Tauri's own window, shell and filesystem APIs. So the two facts to
carry are that a shorter list than `pill.json`'s is not available to buy,
and that a longer one buys nothing the commands need.

**What we are storing that we must protect.** Nothing that needs
protecting, and that is a property to keep rather than to be pleased about.
If a later change is tempted to put something else in `shell_window`
because the table is conveniently per account, it does not go there: this
table is a window's size and place, and history, settings and secrets all
have homes already.

**What the system should refuse to do, even if asked nicely.** Show the
dashboard to anyone who is not signed in, keep it alive across a sign out,
put a rail item on screen that opens nothing, or accept a size or a
position from the interface. Each of those is refused in Rust, not by the
screen choosing not to ask.

## Build plan

Three milestones, each leaving the project working. Record 0002's milestone
5 settings screen is built after the third and is not part of this record.

1. **The two windows.** Rust creates the dashboard window when the person
   is signed in with a key saved, hides the small window while the
   dashboard is shown, shows the small window for a pre shell state or an
   interruption, and closes the dashboard on sign out. The new capability
   file, `pill.json`'s two event permissions and nothing else. The dashboard
   holds nothing but a white surface at this point. Proved live: sign in and
   land on the dashboard; switch Windows microphone access off and press the
   hotkey, and watch the small window come forward with the error while the
   dashboard stays exactly where it was; switch access back on and dictate,
   and watch the small window go and the dashboard come back untouched and
   unraised. That is AC-1, AC-6 and record 0002's AC-32 in one sitting.
2. **The remembered geometry.** The table and its migration, the clamp on
   read, the write when a move or a resize finishes, the screen match with
   its primary screen fallback, and the 960x640 floor. Proved live the way
   the pill's was, because it is the same problem: two monitors of different
   sizes, leave the window on the second one, restart, and it comes back
   there; then unplug that monitor, restart, and it opens wholly on screen
   on the remaining one. That is AC-3, AC-4 and AC-5.
3. **The rail and the surface.** The rail with what exists, rendered from
   `get_rail()`, the account block at the foot with its four values from
   `get_auth_state()`, the Sign out that moves out of `mountSignedIn`, the
   offline row that moves out of it too, the white reading surface, and the
   landing on Settings, Dictation. **`mountSignedIn` and its
   `.signed-in__offline` rule are deleted in this same change**, per
   standing rule 11, and this is the change that owes both of the things it
   held a new home. Proved live: AC-2, by clicking every item in the rail;
   AC-7, by signing out and then signing in as a second account and finding
   its own window place rather than the first's; and **record 0003's AC-14,
   which is proved here and stays record 0003's criterion**, by starting the
   app with the network off and finding the `OFFLINE` badge and its sentence
   at the top of the account block with the name and Sign out unmoved, then
   putting the network back and watching the row go on the next refresh
   without anything else on screen shifting. Evidence for that half is
   named after AC-14 and belongs to record 0003's set, the same way this
   record's milestone 1 carries evidence for record 0002's AC-32.

## What this makes harder

- **Three windows now, and windows are the thing to keep down.** Every
  event Rust broadcasts reaches one more listener, every window is another
  capability file to get right, and "the EchoScribe window comes forward"
  stops naming one thing, so record 0002's amendment has to say which. The
  next feature that wants a window of its own should be made to justify it
  against this paragraph.
- **A remembered position is a promise that breaks quietly when hardware
  changes.** It cost the pill two acceptance criteria and a live two monitor
  sitting, and this window now owes the same proof. It will be tempting to
  skip the unplug half. That is the half that fails.
- **The rail re-lays out as features land.** Because nothing is greyed out,
  each of plan rows 3, 4 and 5 has to remember to amend this record and add
  its item. A row that forgets ships a feature nobody can reach.
- **The landing destination is temporary and nothing enforces that.** If
  plan row 5's record forgets to move it, a person who has History will
  still land on Settings, and nothing will fail. **Settled 2026-09-04**: record
  0007 did move it, and the risk this bullet named never landed. What it leaves
  behind is worth keeping, because the shape recurs: a value written down as
  temporary is only temporary if something later reads the note.
- **Two error surfaces are now two windows apart.** A screen on the small
  window cannot show anything about the dashboard's state, or the other way
  round. That is a good boundary, and it is also one more thing to hold in
  mind when a future error needs a home.
- **One of record 0003's criteria is now only reachable through this
  record's shell.** Added 2026-09-03. AC-14's offline sign lives in the
  account block, so record 0003 can no longer be verified on its own: a
  person checking AC-14 has to have the dashboard. This is the second such
  tie, after record 0002's AC-32, and both were found by a gate rather than
  by the record that owned the criterion. The lesson to carry is that
  deleting a screen deletes every promise it was keeping, so the next
  deletion should be made to list them before it happens rather than after.

- **Closing a window now turns dictation off across the whole machine.** Added
  2026-09-03 by the second amendment, and this is the one cost of the six
  readings that is carried rather than fixed. EchoScribe's headline promise is
  that the hotkey works anywhere, and the hotkey lives in the app, so closing
  the dashboard ends it. A person who closes the window to tidy their desktop
  has quietly switched the product off, and nothing tells them. There was no
  alternative available today: the two other readings of the close button are
  worse, and the thing that would actually solve it, a tray icon, is not drawn
  in `design/registry.md`, is not in any record, and would be a new decision
  about an app that keeps running when it has no window. It is handed to
  `/scope` as a row to weigh rather than built here, and it is named in Still
  open so that it is not rediscovered.
- **The clearing table is now one copy with a wire between its readers.** Added
  2026-09-03 by the second amendment. That is better than two copies, and it is
  not free: the interface is now blind to error kinds and can only draw what
  Rust names, so a kind Rust forgets to classify reaches a screen that will not
  be mounted, silently. The classifier must return a screen for every kind it is
  given or refuse loudly, and a new kind now touches Rust before it touches any
  screen. The source guard covers the residue, not this.

## Still open

- ~~**Whether Tauri 2 needs an explicit permission entry per application
  command in a non default capability.**~~ **Closed 2026-09-03. It does
  not, because no such entry exists.** `src-tauri/gen/schemas/` was read on
  2026-09-03: `acl-manifests.json` and `capabilities.json` hold no entry for
  any of this app's own commands, and searching both for `get_auth_state`
  and `save_deepgram_key` returns nothing. Tauri's permission list covers
  its own plugin commands only. So there is nothing to grant or withhold
  per command, and `dashboard.json` grants only the two event permissions
  the offline row needs. The instruction not to resolve it by granting
  `core:default` was right and is honoured. What protects the commands is
  the `not_signed_in` refusal already in Rust; see the capability paragraph
  in The decision and the addition in Risk.
- **The 960x640 floor is a judgement, not a measurement.** It was chosen so
  a 1366x768 laptop can show the window. If a screen a real person uses
  cannot, it moves by amendment rather than by a quick edit.
- **What the Transcription section holds beyond the saved key row.** Record
  0002's AC-12 masked row is the only thing that exists there today. Not
  this record's to fill.
- ~~**The landing destination once History exists.**~~ **Closed 2026-09-04.**
  Record 0007 settled it, moved it to History, and amended this record by the
  fourth amendment of that day, which also reworded AC-1. Naming it here is what
  stopped it being discovered, which is what this bullet was for.
- ~~**Whether the dashboard ever needs to listen to a Rust event.**~~
  **Closed 2026-09-03. It does, to two, from the first milestone that has a
  rail.** The offline row has to arrive and go while the dashboard is open,
  so it listens to `auth:offline` and `auth:signed_in`, both already emitted
  by `sign_in/renewal.rs`. This bullet said adding one would be an
  amendment because it widens a capability file, and that is exactly how it
  was added: the user chose the two event permissions on 2026-09-03 over
  asking `get_auth_state()` on a timer. A third event is still an
  amendment, for the same reason.
- **Whether EchoScribe should keep running with no window, behind a tray icon.**
  Added 2026-09-03 by the second amendment. Closing the dashboard closes the app
  and so stops the global hotkey, which is the whole product. A tray icon is the
  ordinary Windows answer and this record deliberately did not reach for it: it
  is drawn nowhere, it is in no record, and an app that runs with no window on
  screen touches the no silent listening rule closely enough to deserve its own
  conversation rather than a line here. Handed to `/scope` as a row.
- **Whether the key setup screen shows its step indicator when it is reached at
  load.** Added 2026-09-03 by the second amendment, and noticed rather than
  decided. `design/registry.md` draws a "STEP 2 OF 2" indicator for the first
  run path and says there is none when the screen is reached by pressing the
  hotkey with no key. Signing in, quitting before pasting a key and reopening is
  a third route in, and the registry does not say which of the two it is. The
  screen as built draws no indicator on any route, so nothing is currently
  wrong on this route; the gap is that nothing says so on purpose. Not this
  record's to settle: it belongs to `/canvas` and record 0002.
