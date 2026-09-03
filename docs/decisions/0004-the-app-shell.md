# 0004. The app shell

**Status:** Proposed
**Date:** 2026-09-02
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
  white reading surface on the right, and Settings open on Dictation.
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
that lands the rail, per standing rule 11, and its Sign out moves to the
account block.

**A third capability file**, `src-tauri/capabilities/dashboard.json`,
scoped to the dashboard window's label alone. It is granted the narrowest
set that lets the dashboard call the commands named under Interface surface
and nothing else. It is **not** granted `core:default`, and it is granted
nothing that lets a web view move, resize, close or otherwise touch any
window: the person resizes the dashboard through its own title bar, which
is Windows doing it, and Rust reads the result. `pill.json` is the shape to
copy, not `default.json`.

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
| Which sections the rail holds | AC-2 | This record. A fixed list, one entry per feature that is built: today Settings alone, with Dictation and Transcription beneath it. Handed to the interface by Rust, never a list the interface holds, for the same reason `get_hotkey` hands out the two hotkeys: a screen must not be able to invent a destination. A section joins the list by an amendment to this record when its own feature record is written. |
| What a person reads for each rail item | AC-2 | `design/registry.md`, which owns wording. Rust hands out identifiers, not labels, exactly as `get_hotkey` hands out stored values and the screen decides what a person reads. |
| The landing destination | AC-1 | This record. Settings, on its Dictation sub-section. It moves to History when plan row 5 is designed, by an amendment carried in that record and here. |
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

## Interface surface

Two commands, both on the same terms as every other command in this
project: the interface asks and Rust decides, neither takes an account id
because Rust knows it from the session, and both refuse when nobody is
signed in.

- `get_rail()` returns the sections that exist, as identifiers, and which of
  them is the landing destination. The interface renders what it is given
  and cannot add a destination, which is AC-2 as a rule in Rust and not
  only a rule in the design. It returns no wording.
- `sign_out()` already exists, from record 0003. The account block's Sign
  out calls it and nothing else. It is named here only because the
  dashboard's capability has to allow it.

There is deliberately **no command for the window's size or position**. Rust
positions and sizes the dashboard itself before showing it, and reads the
result back from Windows afterwards. Nothing about the geometry crosses
into the interface as a number, in either direction. This is the pill's
precedent after the AC-27 fix, written down here so nobody adds a
convenience getter later.

Which section is showing is the dashboard's own business and needs no
command and no event. It is drawing, not a decision.

Errors that matter: not signed in, which both commands refuse; and the rail
being unreadable, which cannot happen, because the list is fixed in this
record rather than stored.

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
   file, written from `pill.json` rather than `default.json`. The dashboard
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
   `get_rail()`, the account block at the foot with the Sign out that moves
   out of `mountSignedIn`, the white reading surface, and the landing on
   Settings, Dictation. `mountSignedIn` is deleted in the same change.
   Proved live: AC-2, by clicking every item in the rail, and AC-7, by
   signing out and then signing in as a second account and finding its own
   window place rather than the first's.

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
  still land on Settings, and nothing will fail.
- **Two error surfaces are now two windows apart.** A screen on the small
  window cannot show anything about the dashboard's state, or the other way
  round. That is a good boundary, and it is also one more thing to hold in
  mind when a future error needs a home.

## Still open

- **Whether Tauri 2 needs an explicit permission entry per application
  command in a non default capability.** `pill.json` grants only two event
  permissions and the pill invokes nothing, so it is not the precedent that
  answers this. Read it against the generated schema in
  `src-tauri/gen/schemas/` at build time and grant the narrowest set that
  actually works. Do not resolve it by granting `core:default`.
- **The 960x640 floor is a judgement, not a measurement.** It was chosen so
  a 1366x768 laptop can show the window. If a screen a real person uses
  cannot, it moves by amendment rather than by a quick edit.
- **What the Transcription section holds beyond the saved key row.** Record
  0002's AC-12 masked row is the only thing that exists there today. Not
  this record's to fill.
- **The landing destination once History exists.** Plan row 5's record
  settles it, moves it, and amends this record. Named here so that it is not
  discovered.
- **Whether the dashboard ever needs to listen to a Rust event.** It does
  not at first, and its capability grants nothing for it. Adding one is an
  amendment, not a build detail, because it widens a capability file.
