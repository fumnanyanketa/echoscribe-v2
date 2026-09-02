# 0002. Dictate with a hotkey

**Status:** In progress
**Date:** 2026-08-27
**Amended:** 2026-08-29. AC-2 and AC-19 were named but never settled, and
`/develop`'s gate stopped on both. This amendment settles them and touches
nothing else.
**Amended:** 2026-08-29. The pill's opening position, which screen it opens
on, and whether a dragged position is remembered had no named source, and
`/develop`'s gate stopped on AC-1. This amendment settles all three, adds
AC-23 to AC-26, adds two fields to `dictation_setting`, and writes in how
the 300 millisecond double tap window is measured. Nothing else changes.
**Amended:** 2026-08-29. The live `/check verify` run on milestone 1 came
back FAIL. It closed both spikes this record left open, and one of the
answers broke an assumption the record was built on: the pill can take
focus, and does, on a plain single click. This amendment closes both
spikes, keeps the no focus refusal exactly as written, and replaces the
mechanism the record named for it, which has been disproved live. No
criterion is renumbered, reworded or added. The count stays at 27.
**Amended:** 2026-08-30. Milestone 2 left three things owed. This amendment
settles where a person reads a microphone error, given that the pill cannot be
clicked; blesses a fourth microphone error kind this record never named; and
writes down which microphone the feature uses. It adds AC-28 to AC-31 and takes
the count to 31. No existing criterion is renumbered or reworded. Evidence:
`docs/evidence/dictate-with-a-hotkey/milestone-2-decisions-owed.md`.
**Amended:** 2026-08-30, second of the day. Step 2a's build went up and the live
check found blocked-by-Windows reading as the catch-all. The mechanism this
record named for telling the four errors apart was disproved: cpal on Windows
never returns the permission-denied kind the mapping relied on. A privacy block
arrives as the catch-all kind with the Windows access-denied code in its
message, proven by probe on 2026-08-30 with the machine-wide microphone toggle
off. This amendment replaces the detection mechanism for that one kind and
touches nothing else. No criterion is renumbered, reworded or added. The count
stays at 31.
**Amended:** 2026-08-30, third of the day. Milestone 3 was built against five
things this record named and never settled: where a person goes to get a
Deepgram key, what request checks a pasted one, which Deepgram response means
which cause, the sentence for each key error, and AC-13's two next steps. All
five were answered in the build conversation and written down in
`docs/evidence/dictate-with-a-hotkey/milestone-3-decisions-owed.md`. This
amendment carries all five into the record itself, most of them into Value
sourcing, so nothing downstream has to read an evidence file to know what was
decided. It approves a second outside address, `https://console.deepgram.com`.
None of the five renumbers, rewords or adds a criterion. It also settles one
thing nobody had asked before: what an error screen shows once the person has
fixed the cause and dictated successfully. That adds AC-32 and takes the count
to 32.
**Amended:** 2026-08-30, fourth of the day, by `/sync` rather than
`/architect`, because neither half is a decision. The live `/check verify`
sitting checked this record's Interface surface against what the build actually
exposes and found two places they had stopped matching. `pill_drag_finished()`
is named here and does not exist, because the AC-27 fix moved the whole drag
into Rust. `dictation:needs_key` exists, is what AC-9 rides on, and was never
added to the event list when milestone 3 built it. Both are corrected in place
and marked there. No criterion is renumbered, reworded or added, the count
stays at 32, and nothing in Still open is touched. Evidence:
`docs/evidence/dictate-with-a-hotkey/report.md`.
**Amended:** 2026-08-30, fifth of the day, by `/architect`. Two questions in
Still open are closed with what the live sitting found, and milestone 4 is
handed a state its AC-30 list did not name. The sound bullet was not merely
stale, it was wrong: the fallback it said was keeping the two sounds audible
had never once run and could not have. The pill focus bullet is closed by the
fix `/debug` landed and the live re-proof that followed. The third change names
the microphone dying after it opened as a source of a mid dictation error,
alongside the Deepgram ones, and settles its ending and its words. No criterion
is renumbered, reworded or added and the count stays at 32. AC-30 already
covers that state in as many words, which is why it gains no criterion of its
own. Evidence:
`docs/evidence/dictate-with-a-hotkey/AC-2-sound-fallback-never-fires.md`,
`AC-27-pill-no-longer-takes-focus.md` and
`finding-microphone-dies-mid-dictation.md`.
**Amended:** 2026-08-31, the ninth. Milestone 4, streaming and typing, was
built and run live, committed at `64bf2c7`, against six decisions this record
named and never settled. All six were answered by the user on 2026-08-30 and
written up in
`docs/evidence/dictate-with-a-hotkey/milestone-4-decisions-owed.md`; this
amendment carries them in, most into Value sourcing, and records a seventh
found fact, that the Windows property naming a password field is `IsPassword`.
One of the six is a visible promise no criterion covered, the pill's grey
unfinished wording, which becomes AC-33 and takes the count to 33. The other
five land under AC-14, AC-20 and AC-30, by the fifth amendment's own
reasoning: what was missing was this record's list of causes, not a promise.
This amendment also settles the decision the live run forced, evidence in
`docs/evidence/dictate-with-a-hotkey/finding-new-notepad-collapses-injected-unicode.md`:
the new Windows Notepad collapses backlogged injected keystrokes, so typing
gains a per application direct channel for receivers proven to collapse,
simulated character keystrokes stay the default everywhere else, and the
batch and pause pacing in `typing.rs` is ratified as a mitigation, never a
cure. Both of this record's original spikes stay open and are re-pointed,
because milestone 4 was to settle them and did not.
**Amended:** 2026-08-31, the tenth. `/develop`'s gate stopped on step 4a:
the collapse list names the new Windows Notepad, and no row named the
property the running code reads to recognise it in the focused window at
typing time. This amendment settles it as the Store package identity, the
package family name `Microsoft.WindowsNotepad_8wekyb3d8bbwe`, read from
this machine on 2026-08-31 rather than assumed, with cannot tell meaning
the default keystrokes run. It also carries in the one line the user
decided the same day and `/debug` had deliberately held: consecutive
finalised phrases within one dictation are joined by a single space before
typing. No criterion is renumbered, reworded or added. The count stays
at 33.
**Amended:** 2026-08-31, the eleventh, by `/develop` under its gate's
option 3, authorised by the user the same day. One value, needed by AC-20
and AC-30 and named nowhere: how long the pill holds its last words before
it closes. Today the pill hides in the same instant those words are
emitted, which is why the live sitting saw the password refusal "just
disappear". Settled at 2 seconds, measured from the words being shown, for
every ending that puts words on the pill; then the pill closes and the
closing sound plays, exactly as already decided. Nothing about which words,
which endings or which sounds changes. The build itself did not proceed:
the same gate found the pill's geometry once a transcript line exists has
no named source, the drawn design holding a 232x44 MIC OPEN form and a
472x52 listening form with no rule for when the pill is which, and the
user routed that to `/architect`. The 2 seconds is not built until that
returns. No criterion is renumbered, reworded or added. The count stays
at 33.
**Amended:** 2026-08-31, the twelfth. `/develop`'s gate stopped where the
eleventh amendment says: the pill's geometry once anything is shown on it
had no named source. The drawn design holds a 232x44 MIC OPEN form and a
472x52 listening form with no rule for when the pill is which; the
registry's pill shell row says fixed geometry across every state; the
design system's prose says the pill never grows; and the decided sentences
do not fit the 232 form. The user settled it the same day: the pill has one
geometry, the drawn 472x52 listening form, from open to close, in every
state. The 232x44 form is retired as a window size. This unblocks the
eleventh amendment's 2 second hold and the pill's text rendering, and it
answers the error pill geometry question plan row 2 had flagged. No
criterion is renumbered, reworded or added. The count stays at 33.
**Weight:** heavy
**Plan row:** 2
**Supersedes:** nothing

## In one line

Double tap Ctrl anywhere on the machine and EchoScribe opens the
microphone, shows a floating pill with a live waveform, streams your voice
to Deepgram with your own key, and types the finalised words at your
cursor in whatever app you are in; double tap again to stop. It buys the
whole core promise of the app in one feature, and it costs a keyboard hook
that watches every keystroke all day, which this record constrains hard.

## What this is for

Someone mid flow in a document, an email or a message wants to switch to
speaking for a stretch without leaving the app they are in. This is the
feature that makes that possible, and it is the tracer bullet: it proves
the hotkey, the microphone, the outside transcription service and typing
into another app all connect, for real, before any one of them is made
great.

## Acceptance criteria

- **AC-1**: Signed in, with a working Deepgram key saved, double tapping
  Ctrl opens the microphone and the floating pill appears with a waveform
  that moves with my voice, within one second of the second tap.
- **AC-2**: When the microphone opens I hear the short sound Windows
  already plays when a device is plugged in, and when it closes I hear the
  one it plays when a device is unplugged. Both come from the machine's own
  sound scheme. No sound file ships with the app.
- **AC-3**: While I am still speaking, the words appear as typed text at
  my cursor in whatever app is focused, arriving in phrases, without me
  holding or releasing any key.
- **AC-4**: Text that has already been typed is never taken back or
  rewritten. Only wording Deepgram has marked as final is ever typed.
- **AC-5**: Double tapping Ctrl again closes the microphone, the pill
  disappears, and the closing sound plays.
- **AC-6**: Two Ctrl presses with any other key in between, such as
  Ctrl C followed by Ctrl V, do not start dictation.
- **AC-7**: If I click into a different window mid dictation, the words
  that follow appear at the cursor in that new window.
- **AC-8**: After 30 seconds with no speech, or after 5 minutes of
  dictation, the microphone closes on its own and the pill disappears.
- **AC-9**: If I press the hotkey with no Deepgram key saved, the
  microphone does not open. A guided setup screen appears instead,
  explaining what a Deepgram key is and linking out to get one.
- **AC-10**: Pasting a valid key into that setup screen checks it against
  Deepgram, saves it, and dictation works from then on, including after
  closing and reopening the app.
- **AC-11**: An invalid key is rejected at the setup screen with a message
  saying so, and nothing is saved.
- **AC-12**: Settings shows only the last four characters of the saved
  key. The full key is never shown again once saved.
- **AC-13**: If the saved key stops being accepted, dictation stops and
  the message says which of the two happened, the key was rejected or the
  allowance ran out, with the matching next step for each.
- **AC-14**: If the connection drops mid dictation, the app tries to
  reconnect once. If that fails, the microphone closes, the pill
  disappears, and a message says dictation stopped because the connection
  was lost. Words already typed stay where they are.
- **AC-15**: If the microphone is unavailable, the pill never appears and
  the message names the actual cause, with a link straight to the Windows
  microphone privacy setting when access is blocked.
- **AC-16**: The hotkey does nothing at all when nobody is signed in.
- **AC-17**: Each completed dictation is saved with its text, its start
  time and how long it lasted, against the signed in account, and is still
  there after closing and reopening the app.
- **AC-18**: A second account signing in on the same machine sees none of
  the first account's dictations and cannot use its Deepgram key.
- **AC-19**: Settings offers exactly two hotkeys to choose between, double
  tap Ctrl and double tap Alt. Either side of the keyboard counts for each.
  Picking the other one works immediately, with no restart, and is still in
  force after a restart.
- **AC-20**: If the focused field is a password field, nothing is typed
  and the pill shows that dictation was blocked.
- **AC-21**: Settings has one switch for the opening and closing sounds, on
  to begin with. Switched off, both sounds are silent and nothing else
  about dictation changes: the pill still appears. The choice is still
  there after a restart.
- **AC-22**: There is no way to set any hotkey other than those two. No box
  to type one into, no way to record one, and the one I did not pick does
  nothing at all.
- **AC-23**: The first time the pill appears it is at the bottom centre of
  the screen, just above the taskbar, on the same screen as the window I am
  typing into. It never covers the taskbar and never sits part way off the
  edge.
- **AC-24**: I can drag the pill anywhere on the screen. Where I let go is
  where it opens next time, still after closing and reopening the app, and
  in the same relative spot when I next dictate on a different screen.
- **AC-25**: Once the pill has opened it stays where it is for the whole
  dictation, even when I click into a window on another screen and the
  words follow me there. It moves only when I drag it.
- **AC-26**: If the screen I left the pill on is unplugged, or its
  resolution changes, the pill still opens fully on screen, on the screen
  holding the window I am typing into.
- **AC-27**: Clicking or dragging the pill never moves my typing cursor.
  I can drag it mid dictation and the words that follow still land in the
  app I was already typing into, and my place in that app is not lost.
- **AC-28**: If the microphone will not open, the EchoScribe window comes to
  the front saying what went wrong: a short code, one sentence naming the
  cause, and exactly one action. No pill appears and neither sound plays.
- **AC-29**: When Windows is blocking microphone access, that one action opens
  the Windows microphone privacy page itself. On every other microphone
  failure the one action is Try again, and pressing it starts dictation there
  and then if the microphone now opens.
- **AC-30**: Nothing in dictation ever asks me to click the pill. When
  something goes wrong while the microphone is open, the pill says so in words
  and closes, and if there is anything I can do about it the EchoScribe window
  comes forward carrying that one action.
- **AC-31**: Dictation uses whichever microphone Windows is set to. Changing
  the Windows default and dictating again uses the new one, and there is
  nowhere in EchoScribe to choose a microphone.
- **AC-32**: If I fix what an error told me was wrong and then dictate
  successfully, the EchoScribe window is no longer showing that error. It
  clears on its own, quietly: the window does not come to the front, does not
  hide itself, and does not move.
- **AC-33**: While I am still speaking, the pill shows the wording Deepgram
  has not yet settled, in grey, and replaces it as I go. Those unfinished
  words appear on the pill and nowhere else: they are never typed at my
  cursor, never saved, and when the final wording comes out differently
  nothing at my cursor has to be taken back.

## The decision

A low level Windows keyboard hook watches for two taps of the chosen
modifier within 300 milliseconds, where a tap means that key went down and
came back up with no other key pressed in between. The 300 milliseconds
are measured from the release of the first tap to the press of the second,
so how long either key is held down never eats into the window. The
chosen modifier is Ctrl, the default, or Alt, and either side of the
keyboard counts. Any
other key resets the count and is thrown away in the same instant, never
compared against a stored binding, which is why the allowed set is two
modifier double taps and not arbitrary key combinations. That opens an
always on top pill with a live waveform, plays the sound Windows uses for a
device being plugged in unless sounds are switched off, and starts
capturing the microphone. Audio
streams straight to Deepgram over their official Rust SDK using the
person's own API key, which lives in the Windows Credential Manager and
never enters the interface. Only wording Deepgram marks as final is typed,
by simulating real keystrokes into whatever window is focused at that
moment, so nothing typed ever has to be taken back. A second double tap,
30 seconds of silence, or 5 minutes elapsed closes it. The finished text
is saved to a local SQLite table owned by the signed in account.

The pill opens at the bottom centre of the working area of whichever screen
holds the focused window, just above the taskbar wherever that person keeps
it. It can be dragged anywhere, and where it is left is remembered against
the account as a position within a screen rather than a point on one
particular screen, so it opens in that same relative spot on whatever
screen the next dictation happens on. Rust places it once, as it opens, and
does not move it again for the rest of that dictation even when the words
follow focus to another screen.

The pill window never takes focus. This is not a nicety: AC-3 and AC-7
both send text to whatever window has focus, so a pill that activated on
click would take the words for itself the moment someone moved it.

**How that is achieved changed on 2026-08-29.** This record used to say the
pill was created as a window Windows will not activate, and treated that as
settling it. The live run disproved it. A single click on the pill moved the
typing cursor away from the app the person was in, and the keystrokes that
followed were lost. No drag was needed. See
`docs/evidence/dictate-with-a-hotkey/AC-27-clicking-the-pill-steals-focus.md`.

The refusal stands exactly as written. Nothing here is softened to fit what
the code currently does, and there is no grace period in which focus may
move and then be put back: a person cannot see a brief theft, and anything
they typed inside it is gone. AC-27 stands, AC-24 stands, and the pill stays
draggable. Windows' own on screen keyboard is draggable and never takes
focus, so this is a solved shape rather than a hope.

What the record got wrong was treating the pill as one window settled by one
style set once. It is a frame with a web view inside it, and the drag hands
the window to the operating system. Each of those can take focus on its own,
so the approach is now that the pill must be non activating at every layer
that could take it. Three things must each hold, and each must be proved
live, not read in the code:

1. **The frame refuses activation at the moment of the click**, not only at
   the moment it is created. Pressing the mouse on the pill must leave the
   foreground window untouched.
2. **The web view inside it never takes keyboard focus.** The pill holds
   nothing that can hold a caret, and its one interactive area does not
   accept one either.
3. **The drag never hands the window to the operating system's own move
   loop**, because that loop activates whatever it moves. The pill has to be
   moved without asking Windows to run the move for us.

The pill has exactly one area that answers the mouse at all, the grip. Every
other part of it is inert. The live run showed a click anywhere on the pill
taking focus, which means the surface that can steal focus is currently
larger than the surface meant to be interactive, and that gap has to close
whichever way the three points above are met.

**Stop condition.** If all three hold and a live pass still shows the focused
window moving, `/debug` stops there and reports what was tried. It does not
keep going, and it does not quietly drop the drag. Dropping the drag and
making the pill inert to the mouse is a real option, weighed below, and
taking it is the user's call in a fresh conversation, not a fallback taken
on the way past.

**Where a person reads an error, settled 2026-08-30.** Milestone 2 found two
decided things that looked as though they could not both hold. The design
system puts errors in the pill, and the amendment above takes the pill out of
the mouse's path everywhere except its grip, so the pill has no surface that
can carry AC-15's link to the Windows privacy setting.

They can both hold, because they were never about the same error. AC-15's own
words are that the pill never appears. The pill on screen is the one visible
sign that the microphone is open, which is the whole of the no silent
listening rule, so a pill may not appear when the microphone did not open. An
error that happens before there is a pill was never a pill error.

So the rule, for every error in this feature and not only AC-15:

- **The pill states what happened, in words, and never asks to be clicked.**
  It is a sign, not a control. This holds for errors that arrive mid
  dictation too, AC-13 and AC-14, where the pill does exist.
- **Every action a person can take lives in the EchoScribe window**, which
  takes clicks and always has. On a microphone failure that window is brought
  to the front carrying a short code, one sentence of cause, and exactly one
  action. That is the same shape AC-9 already uses when the hotkey is pressed
  with no Deepgram key saved: a hotkey press that cannot start dictation, and
  a window that says why.

The cost is named rather than hidden. Bringing that window forward moves the
focused window away from the app the person was typing in. That is a real
interruption, it is the same one AC-9 already accepts, and it is only ever the
answer to a hotkey the person just pressed and got nothing from. Nothing is
brought forward that the person did not ask for.

**There is no drawn comp for this screen.** The EchoScribe window shows the
sign-in screen and nothing else today, so a microphone error is the first
screen in it that is not sign in. `design/registry.md` has to gain it before
it is built, which is `/canvas` work and the next session either way. Two
things are owed there: the microphone error state on the EchoScribe window,
and the registry's own "Error pill" line, which says an error pill has
"exactly one action" and must now say it has none. This record does not draw
either and must not be read as having done so.

**The four microphone errors, their wording and their one action.** `/develop`
wrote all four sentences during milestone 2 and was right to say the wording
was not its to choose. Three are confirmed exactly as written, one is changed,
and each is paired with the single action AC-29 requires.

| Kind | Sentence | The one action |
|---|---|---|
| `microphone_blocked_by_windows` | Windows is not letting EchoScribe use the microphone. | Opens the Windows microphone privacy page |
| `microphone_in_use_by_another_app` | Another app is using the microphone right now. | Try again |
| `no_microphone_found` | Windows cannot find a microphone. | Try again |
| `microphone_unavailable` | The microphone could not be opened. | Try again |

"No microphone is plugged in", which `/develop` wrote for the third, is
replaced. A laptop's microphone is built in and was never plugged in, so that
sentence sends a laptop user looking for a cable that does not exist.

**The fourth kind is blessed.** This record's interface surface named three
microphone errors. cpal can fail in ways that are none of them, an unsupported
stream configuration among them, and `microphone.rs` maps those to a fourth
kind rather than dressing one of the three up to fit. That is right, and it is
right for this record's own reason: each error reads differently because each
has a different next step, and telling somebody another app has their
microphone when it does not sends them to the wrong place entirely. A
catch-all that admits it does not know is honest and still actionable, because
Try again is a real next step for a transient failure. The fourth kind is
named here so it is a decision rather than an implementation detail, and so
nothing later collapses it into one of the three.

**How blocked-by-Windows is detected, settled 2026-08-30.** This record used
to assume the audio layer would say "permission denied" when the privacy
setting was the cause, and the code mapped that kind to
`microphone_blocked_by_windows`. Disproved live: cpal on Windows never
produces that kind for anything. Its own error table has no arm for access
denied, so a privacy block falls into its catch-all, and on 2026-08-30 the
error screen showed `MICROPHONE_UNAVAILABLE` with a Try again button while
the real cause was the machine-wide microphone toggle being off.

The replacement: when the microphone fails to open and the failure is not one
of the two kinds the audio layer does name reliably (device busy, no device),
EchoScribe asks Windows directly whether microphone access is switched off.
It reads, and only ever reads, the same three switches the Windows privacy
page writes: the machine-wide microphone toggle, the per-user toggle, and the
per-user toggle for desktop apps. If any of the three says deny, the error is
`microphone_blocked_by_windows`. If all say allow, or the switches cannot be
read at all, the error stays the honest catch-all, because a switch that
cannot be read is not evidence of blocking. Those switches are a Windows
convention, not a contract; if a future Windows moves them, the failure
degrades to the catch-all with Try again, which is wrong-but-safe rather than
wrong-and-misleading. The check is read only, forever: EchoScribe never
changes a privacy setting, it only reads the state and opens the page for the
person to decide.

**The microphone dying after it opened, settled 2026-08-30.** Everything above
is about a microphone that would not open. One can also stop working while it
is open, and it was seen doing exactly that: with the pill up and the meter
moving, switching Windows' desktop-apps microphone access off killed the
stream. The pill went on saying `MIC OPEN`, the bars went flat, and nothing
told the person, because a dead microphone and a quiet room are identical on a
meter designed to read flat at silence. The stream error went to stderr and
nowhere else. Evidence:
`docs/evidence/dictate-with-a-hotkey/finding-microphone-dies-mid-dictation.md`.

This record names three things that can go wrong mid dictation and all three
are Deepgram's: the connection lost, the key rejected, the allowance spent,
with a scope failure on the live stream handed to milestone 4 as a fourth. The
audio device itself failing after a successful open is a fifth, and it was
never named anywhere. It is named here.

**It gains no criterion of its own.** AC-30 already covers it in as many words:
"when something goes wrong while the microphone is open, the pill says so in
words and closes, and if there is anything I can do about it the EchoScribe
window comes forward carrying that one action." A dying device is that, exactly.
What was missing was never a criterion, it was this record's own list of what
can go wrong. A new criterion would restate AC-30 for one source and split one
criterion's proof across two numbers. What this state gets instead is a source
named, an ending fixed, its words fixed, and a proof step in milestone 4.

**The ending.** The same one every other mid dictation failure has. The pill
stops saying `MIC OPEN`, says `MIC STOPPED` long enough to be read, and closes.
The closing sound plays, because the microphone did close, and that sound is
how a person who is not looking at the pill learns it. Then the EchoScribe
window comes forward with the code, the sentence and the one action, exactly as
it does when the microphone will not open at all.

**Which of the four kinds it is** is decided by the same classification, run
again: a stream that dies for a reason the audio layer does not name reliably
asks Windows about the three consent switches, and is
`microphone_blocked_by_windows` with Open Windows settings if any of the three
says deny, or the honest catch-all with Try again if they all say allow or
cannot be read. That is not tidiness, it is what makes the observed case right.
The person who just switched microphone access off is sent to the page that
undoes it. A Try again button there would be a door that cannot open.

**One sentence changes, and only for this moment.** Three of the four sentences
are true whether the microphone never opened or stopped later. The catch-all is
not: "The microphone could not be opened." is false about a microphone that
did open. So `microphone_unavailable` gets a second sentence, **"The microphone
stopped working."**, used when the failure arrives mid dictation. Same kind,
same code, same Try again. No fifth kind and no fifth code is created, so a
person quoting `MICROPHONE_UNAVAILABLE` is quoting the same thing either way.

**It clears itself with the others.** The clearing table's first row already
covers it: any of the four microphone errors clears when the microphone opens.
Nothing new is needed and nothing about AC-32 changes.

**The silence cap must not get there first.** AC-8 closes the pill after 30
seconds without speech, and a dead microphone is 30 seconds without speech. If
the cap wins that race the person is told nothing at all, the pill simply goes
away, and this whole paragraph has bought nothing. The device failure ends the
dictation the moment the stream reports it, and the cap is left for the thing
it is for, a person who stopped talking.

**Two things are not drawn and must not be invented during the build.** The
pill's `MIC STOPPED` state, and the catch-all's second sentence in the
microphone error screen. `design/registry.md` has to gain both before milestone
4 builds them, which is `/canvas` work. This record fixes the words and does not
draw the states.

**Which microphone is used.** Whichever one Windows is already set to. This
record names no microphone anywhere: no device column, no device field in
`dictation_setting`, no device command, and until now no criterion. That was
never an omission to be filled in during a build, and AC-31 writes it down so
it stops looking like one. `design/registry.md` does draw a device row for
dictation settings, which means a picker exists somewhere in the product's
future. It is not in this record and it is not milestone 5's to slip in.
A picker needs a device list, a stored device against the account, what
happens when that device is gone, and a live input level in a settings screen,
which would open the microphone from a second place in the app and touch the
no silent listening rule. That is a feature with its own risk, so it gets its
own record when it is wanted, not a settings row here.

**The Deepgram key, settled 2026-08-30.** Milestone 3 built AC-9 to AC-13 and
found five things this record named without settling. All five are settled
here, for the reason the four microphone sentences were: an address that
leaves the machine and the words a person reads are decisions, not details a
build gets to choose.

**Where a person goes to get a key.**
`https://console.deepgram.com/signup?jump=keys`. Deepgram's signup page,
jumping straight to the keys screen, so AC-9's link lands the person on the
exact thing it sent them for. It is a fixed literal held in Rust and opened
through the system browser by a command that takes nothing, the same shape as
the Windows privacy page above. The interface never supplies or sees it, and
it is never built from anything.

**What checks a pasted key.** `GET https://api.deepgram.com/v1/auth/token`,
with the pasted key in an `Authorization: Token <key>` header. It is
Deepgram's own documented way to test a key. It proves the key is real and
that Deepgram is reachable, it sends no audio, and it costs no allowance.
AC-10 promises that a checked key means dictation works from then on, so what
the check actually is decides whether that promise is true.

**The gap that leaves, named rather than hidden.** A 200 from that endpoint
does not prove the key can open a streaming socket. A key without the right
scope passes this check and then fails on the live stream. Milestone 4 must
therefore treat a scope failure on the stream as a real state of its own, and
must never assume a saved key works because this check accepted it. Nothing
in milestone 3 may be read as having closed that.

**Which Deepgram response means which cause.** From Deepgram's published
error table, checked against their live documentation on 2026-08-30:

| What comes back | The cause it means |
|---|---|
| 2xx | The key is good. Save it. |
| 401 `INVALID_AUTH` | Rejected by Deepgram |
| 402 `ASR_PAYMENT_REQUIRED` | The allowance ran out |
| No answer at all: connect failure, DNS, timeout | Could not reach Deepgram |
| Anything else, 403, 429 and 5xx among them | The honest catch-all |

The catch-all is a fourth kind and not a fold-in, for the same reason and in
the same shape as the fourth microphone error above: telling somebody their
key is bad when Deepgram was merely rate limiting them sends them off to
replace a key that was fine.

**A consequence worth writing down.** 402 is only ever returned for a
transcription request, so `/v1/auth/token` can never produce one. An
exhausted allowance therefore cannot appear at the setup screen at all. It is
a live stream state, which is AC-13's own wording, and it arrives in
milestone 4. Milestone 3 builds the kind, the code, the sentence and the
action; milestone 4 wires the trigger and proves it.

**The four key errors, their codes and their wording.** Four kinds, four
codes, four sentences, fixed here and held in one place in Rust so no screen
can invent its own. Two of these codes were already drawn in
`design/registry.md` and are unchanged; the other two are fixed here.

| Kind | Code | Sentence |
|---|---|---|
| `deepgram_key_rejected` | `DEEPGRAM_KEY_REJECTED` | Deepgram did not accept this key. Nothing was saved. |
| `deepgram_no_allowance` | `DEEPGRAM_NO_ALLOWANCE` | This key's Deepgram allowance has run out. |
| `deepgram_unreachable` | `DEEPGRAM_UNREACHABLE` | EchoScribe could not reach Deepgram, so the key was not checked. Nothing was saved. |
| `deepgram_check_failed` | `DEEPGRAM_CHECK_FAILED` | The check did not succeed and Deepgram did not say why. Nothing was saved. |

No sentence carries anything read off the pasted key, and no sentence blames
the key when the network was the cause.

**A fifth kind exists since 2026-08-31**, `deepgram_key_not_allowed`, for a
key that is real but not allowed to stream. It is settled below with
milestone 4's decisions, and the four above are unchanged.

**AC-13's two next steps.** AC-13 asks for the matching next step for each of
its two causes, and the two genuinely differ.

| Cause | Its one action | What that action does |
|---|---|---|
| Rejected | Replace key | Opens the AC-9 setup screen, so a new key can be pasted |
| Allowance ran out | Open Deepgram console | Opens `https://console.deepgram.com` in the system browser, where the person tops up or makes a new key |

Making them both Replace key was weighed and rejected: a person whose
allowance has run out gets nowhere by pasting the same key again.

**This approves a second outside address.** `https://console.deepgram.com` is
a fixed literal held in Rust beside the signup page, opened by its own command
that takes nothing. Those two, with the Windows privacy page, are the only
three places this feature sends anybody outside the machine. A fourth is
another amendment, not an addition.

**What an error screen shows once the cause is fixed, settled 2026-08-30.**
Every error this feature puts in the EchoScribe window is a claim that
dictation cannot happen. The person can make that claim untrue without
touching EchoScribe at all: switch the microphone back on in Windows, close
the app that had it, top the allowance up on Deepgram's own site. Then they
press the hotkey, dictation works, and the window is sitting behind everything
still saying it cannot. Nothing had told it otherwise.

So, for every error screen in this feature and not only the microphone one:
**an error screen clears itself the moment the thing it complained about is
shown to work**, and the window then shows whatever is ordinarily true.

It clears quietly. The window does not come to the front, does not hide
itself and does not move. The person is dictating into another app, and this
record has one rule about bringing a window forward: only ever in answer to a
hotkey that produced nothing. A successful dictation is the opposite of that.
This is the same ending a successful Try again already has, drawn as
"Microphone error, retried" in `design/registry.md`. What changes is that the
button is no longer the only door into it.

What counts as shown to work is each error's own proof, never one blanket
signal:

| The error | What clears it |
|---|---|
| Any of the four microphone errors | The microphone opening |
| The three key errors a setup screen can show: rejected, unreachable, check failed | A key being accepted by Deepgram and saved |
| `deepgram_no_allowance`, which only ever arrives mid dictation | The first finalised words coming back from Deepgram |

An open microphone is not proof that a Deepgram allowance is back, which is
why there is a second trigger rather than one. Clearing the allowance error
when the microphone opened would take the message away while the problem was
still there. Milestone 3 has no live stream, so that row is wired in milestone
4 with the rest of AC-13.

No new event carries this. `dictation:opened` and `dictation:text` already
exist and already reach every window, the same way `dictation:error` does.
Nothing in `src-tauri/capabilities/` widens for it.

**What is asked of Deepgram on the stream, settled 2026-08-31.** The model is
`nova-3`, Deepgram's current general model, chosen over `nova-2` and over
`flux-general-en`. Accuracy is the one thing this app must do well and the
model is the largest single lever on it; Deepgram's own published figure for
nova-3 is a 54.2% reduction in word error rate on streaming. It also supports
70+ languages against nova-2's 40+, and plan row 4, speak in your language,
inherits whatever is chosen here. It costs more per minute than the legacy
models, and that is spent from the person's own Deepgram allowance, never the
project's. The stream asks for English, fixed here because Still open already
hands the language to plan row 4; punctuation on, because the words go
straight into a document and a person adding every full stop by hand has not
saved any time; and interim results on, which the next block is for.

**Unfinished wording on the pill, settled 2026-08-31, and it is AC-33.** This
resolved a disagreement between two locked documents. `design/registry.md`
draws a transcript line on the pill, final text ink, interim grey with a
dotted rule. This record's interface surface carried exactly one event with
text on it, `dictation:text`, with one finalised phrase, so nothing could
ever have fed the grey half. Neither half had been built, so neither had been
caught. The drawn design wins: a second event, `dictation:interim`, now
carries the wording Deepgram has not yet settled, and the pill draws it grey.

What does not change: only wording Deepgram marks as final is ever typed,
AC-4 is untouched, and nothing typed is ever taken back. The grey line is a
sign on the pill and never a keystroke.

What this costs, named rather than hidden: a partial transcript now crosses
into the interface, which nothing in this feature did before. The data rules
hold, because they forbid a partial transcript being written to any table,
any log or any file, and this is none of those. It is still more transcript
surface than existed yesterday, so the grey line must never be stored, never
be logged, and never be read by anything but the pill that draws it. That is
now a refusal in Risk, not only a sentence here.

**A key that is not allowed to stream, settled 2026-08-31.** A fifth key
kind of its own, not folded into `deepgram_key_rejected`:

| Kind | Code | Sentence | The one action |
|---|---|---|---|
| `deepgram_key_not_allowed` | `DEEPGRAM_KEY_NOT_ALLOWED` | This key is not allowed to transcribe live audio. | Open Deepgram console |

The pill shows the code and the sentence, then closes, and the closing sound
plays. The EchoScribe window comes forward carrying the one action, because
there is something the person can do. The action is Open Deepgram console
and not Replace key, because a key's permissions are changed in Deepgram's
console and nowhere else; pasting a new key is the right door only for
somebody making a whole new key. It reuses `open_deepgram_console()`, so no
new outside address is approved. It is not folded into rejected on the same
grounds this record has already refused two folds: Deepgram accepted this
key at the setup screen, and saying it did not sends the person off to
replace a key that was never the problem. This names the scope failure
state the sixth amendment handed milestone 4 in as many words. Its wording
is decided; its live trigger is not, because how Deepgram actually reports
it has never been seen. That is the first spike in Still open, and until it
runs the mapping in `deepgram_key.rs` is a documented guess.

**The connection dropping, settled 2026-08-31.**

| Kind | Code | Sentence | The one action |
|---|---|---|---|
| `deepgram_connection_lost` | `DEEPGRAM_CONNECTION_LOST` | Dictation stopped because the connection to Deepgram was lost. | Try again |

The same shape as every other mid dictation ending: the pill says so and
closes, the closing sound plays, and the EchoScribe window comes forward
with Try again, which goes through `try_start` like every other way into
dictation. Words already typed stay exactly where they are, which AC-14
requires outright. Reusing `DEEPGRAM_UNREACHABLE` was weighed and rejected:
its sentence ends "the key was not checked. Nothing was saved.", which is
false about a connection that dropped mid sentence, and changing that
sentence would break it on the setup screen where it is already right.

**Speech during the one reconnect attempt, settled 2026-08-31.** Held in
memory and sent once the connection is back. The microphone stays open for
the attempt, so the pill does not flicker and the person is not interrupted
for a blip that fixed itself. The audio is held in memory only, never on
disk and never in a log; the data rules are unchanged, captured, sent,
discarded. The attempt is capped at 5 seconds and the held audio is capped
at the same 5 seconds, one cap used twice, so a long outage can never grow
a buffer. When the cap is reached the reconnect has failed, and the
connection lost ending above is what happens next.

**Typing refused into a password field, settled 2026-08-31.**

| Code | Sentence |
|---|---|
| `BLOCKED_PASSWORD_FIELD` | EchoScribe will not type into a password field. |

The pill shows both, in the registry's already drawn Error pill shape, a
mono code and one sentence and no action at all. Then dictation stops and
the pill closes. The EchoScribe window deliberately does not come forward:
AC-30 brings it forward only when there is something the person can do,
there is nothing to do here except not dictate into a password box, and
moving somebody's focus while they are on a password field would be the
single worst moment in the app to do it. Dictation stopping rather than
merely skipping keystrokes was never this decision's to make; the Risk
section already fixes it.

**The property that names a password field, recorded 2026-08-31.** Windows
UI Automation's `IsPassword`, asked of the focused element. There is no
second candidate, so this was found rather than decided. How widely it
actually reaches is still the second spike in Still open.

**How characters reach a focused window, settled 2026-08-31.** The live run
after milestone 4 proved three things at once, evidence in
`docs/evidence/dictate-with-a-hotkey/finding-new-notepad-collapses-injected-unicode.md`:
EchoScribe's typing is character perfect, the same burst arrives 205 of 205
in a classic edit control, and the new Windows Notepad collapses backlogged
injected keystrokes, delivering every keystroke that queued during a stall
as a copy of the newest one. The line in What this makes harder that "a
small number of apps handle fast simulated input badly" came true through
its own remedy, finding by use, and the set it called small now contains
the default editor of Windows 11, the first app most people will try
dictation in. Pacing was tried live and honestly judged insufficient: any
receiver stall longer than the pause rebuilds the backlog, so no pace
reaches zero.

The decision is a cure, applied per application, not a documented limit:

- **Simulated character keystrokes stay the default for every app.** Every
  property AC-3 rests on is unchanged: characters and never keys, the
  password check first, nothing interpreted.
- **The batch of 8 UTF-16 units with a 10 millisecond pause between calls
  is ratified**, as exactly what `typing.rs` labels it: a mitigation for
  marginal receivers, never a cure.
- **Receivers proven to collapse backlogged injected keystrokes get a
  direct channel instead.** Each character is posted straight to the
  focused text box as a character message, skipping the shared input queue
  where the backlog builds. It is still a character and never a key, the
  password check still runs first, and nothing touches the clipboard, so
  AC-20 and the data rules are untouched.
- **The list of receivers on that channel is this record's, and it starts
  with exactly one entry: the new Windows Notepad.** A receiver joins the
  list by amendment, carrying evidence of the same shape as the finding
  above, never by a quick addition on the way past a bug.
- **A spike proves the channel before it is built.** Post the finding's own
  205 character burst into the new Notepad through the direct channel and
  compare exact strings, the same proof the finding used, with EchoScribe
  not running. If it does not arrive intact, the channel is not built, and
  the choice comes back to the user as clipboard against documented limit,
  because those are the two options left and each costs something only the
  user can spend.

Clipboard paste was weighed and refused, both for the bad apps alone and
for everywhere. It is fast, atomic and known to work, and it puts the
dictated words in a third place: the clipboard is readable by any clipboard
tool and feeds Windows clipboard history and cloud sync unless flagged, and
the data rules say the words go to the cursor and the local history and
nowhere else, as a hard limit. Accepting the collapse as a documented limit
was refused because the failure lands in the default editor of Windows 11
and garbles the very words this app exists to get right.

**The property that recognises a receiver on the collapse list, settled
2026-08-31, the tenth amendment.** The Store package identity of the
application about to receive the phrase, called the package family name:
Windows keeps one permanent name for every packaged app, and the new
Windows Notepad's is `Microsoft.WindowsNotepad_8wekyb3d8bbwe`, read from
this machine on 2026-08-31 rather than assumed. The code asks it of the
process behind the focused window at the moment of typing, the same moment
the password check already asks its question. It was chosen over the
program file name, which the old and the new Notepad share and any program
may take, and over the text box's class name, which other applications may
share; either one lets the list grow silently, which this record forbids.
When Windows will not answer, the receiver is treated as not on the list
and the default keystrokes run: cannot tell means the normal, proven path,
the same rule the password check follows.

**What separates two finalised phrases at the cursor, decided by the user
2026-08-31 and carried in by the tenth amendment.** A single space, typed
before every finalised phrase after a dictation's first, through the same
one door as the phrase itself, so the password check covers it too.
Deepgram's phrases arrive trimmed, so without this consecutive phrases
collide into one word. Nothing is typed before the first phrase, and the
pill's events are unchanged. This was held during the `/debug` sitting of
2026-08-31 on purpose, so every test there ran against unchanged typing;
that hold ended with the sitting.

**The pill's one geometry, settled 2026-08-31 by the twelfth amendment.**
The pill is one size for its whole life: the drawn listening form, 472x52,
from the moment it opens, before a word has been said, to the moment it
closes, whatever it is showing. MIC OPEN before words, the transcript line
with its grey tail, MIC STOPPED, and every word ending this record fixes,
the password refusal among them, all render inside that one shell.

This resolves a disagreement between two locked documents, the same shape
AC-33 resolved. The comp draws two working sizes and writes no rule for
when the pill is which, while the design system's prose says the pill never
grows and the registry's pill shell row says fixed geometry across every
state. This time the written rule wins over the drawings, chosen by the
user on 2026-08-31: a small form that exists only for the breath before the
first words buys almost nothing, and it would make the pill visibly change
size a moment after speech starts, on every dictation. The comp's 232x44
MIC OPEN form is retired as a window size; what it shows may survive as the
wide shell's content before words arrive, which is `/canvas`'s to draw, not
this record's. The comp's 48 high error pills are reconciled the same way:
one geometry across every state means they take the same 472x52 shell,
which is what the registry's own error pill row already requires in as many
words.

Three rules follow, and each is a rule rather than a preference:

- **The pill window is sized once, as it opens, and is never resized while
  it is open.** AC-25's promise that the pill never moves mid dictation
  becomes structurally true rather than defended: a window that cannot
  change size never has to shift to stay on screen.
- **Everything the pill can ever show fits inside one fixed footprint, and
  the whole footprint is clamped inside the working area at open.** The
  elapsed and word count that appears after 20 seconds is part of that
  footprint from the first moment, so its arrival resizes nothing and moves
  nothing. The comp places it outside the pill's own 52 pixels, which is
  why the footprint and the pill are named apart here; where it sits within
  the footprint is `/canvas`'s call.
- **No stored value changes.** `pill_x` and `pill_y` are fractions naming
  the pill's centre within a working area and say nothing about its size. A
  position saved against the old 232x44 pill is re-clamped at the next open
  against the wide footprint by the clamp that already runs, which is the
  same rule AC-23 and AC-26 already impose on a screen of any size. No
  migration.

What this costs, named rather than hidden: the pill is 472 pixels wide from
its first instant, roughly twice the drawn MIC OPEN form, sitting over the
person's document before they have said anything. Accepted, because the
wide form is needed within about a second of speech starting anyway. And
the change is not free to build: the geometry constants in
`pill_window.rs`, the placement tests beside them, the grip rectangle in
`pill_mouse.rs` and the fixed sizes in `pill.css` all change together, and
any change to the pill's geometry re-opens the AC-27 live click proof, per
the standing line in What this makes harder. That proof rides in the
`/check verify` pass this feature already owes.

## What else was considered

| Option | Why not |
|---|---|
| Hold to talk | Safest against leaving the microphone open, but the user wanted a free hand for long dictation. The silence timeout and hard cap cover the safety gap it leaves. |
| Ctrl + Shift + Space as the hotkey | Would use ordinary Windows hotkey registration and need no keyboard hook at all, which is simpler and far less invasive. The user wanted double tap Ctrl, which Windows cannot register as a normal hotkey. |
| Holding both Ctrl keys at once | The user's first phrasing. Needs the same hook, and is awkward two handed. Double tap of either Ctrl is the same key with a better feel. |
| Typing every word immediately and fixing with backspaces | Truly word by word and the fastest possible feel, but a miscounted backspace, a focus change or an app's own autocorrect means it deletes the person's own text. |
| Not typing anything until you stop | Completely safe and much simpler, but loses the live feel that decision 0001 chose streaming for. |
| Locking dictation to the window it started in | Words could never land somewhere unintended. The user chose follow the cursor deliberately, accepting that a popup or a stolen focus sends text elsewhere. The visible pill and the password field refusal are the mitigations. |
| Letting the hotkey be any key combination, Ctrl+Shift+Space style | What most apps offer, and what people expect from a hotkey setting. It would force the hook to compare every ordinary keystroke, all day, against a stored binding. That breaks the one property the whole risk section rests on: that the hook looks at modifiers and discards everything else instantly. Two modifier double taps buys the setting without paying that. |
| Shift and Win as double tap options | More choice for anyone whose Ctrl and Alt are already claimed. Repeated Shift taps are what triggers Windows Sticky Keys and what some language input methods use, and a double tapped Win opens and closes the Start menu underneath you. Both misfire in normal use. |
| Bundling two sound files with the app | Identical on every machine and fully under our control. It ignores the sound scheme the person has chosen, adds assets and a `/canvas` pass, and buys nothing a system sound does not. |
| Clipboard paste instead of keystrokes | Instant for long text, but overwrites what the person had copied and is blocked outright by some apps. |
| Deepgram key in the SQLite file | One storage place and it would carry to Mac unchanged, but the app would have to hold the encryption key next to the thing it protects. |
| The pill at the text cursor | Follows the eye, so you never look away from the words arriving. It covers the very text being typed, and it would jump every time focus changed, which AC-7 makes an ordinary thing to do. |
| A fixed pill that cannot be dragged | One less stored setting and always in the one place you learned. Whatever default is picked will sit on top of something for somebody, and without a drag they have no way out short of another settings screen. |
| A dragged position pinned to the one screen it was dragged on | Simpler to reason about, and the drag wins outright over everything else. It also means the only visible sign the microphone is open can sit on a screen the person is not looking at, which is the thing choosing the focused screen was meant to prevent. |
| The pill following focus between screens mid dictation | Keeps the microphone sign in front of you at all times, which is the strongest reading of the no silent listening rule. A window that jumps screens mid sentence is startling, and always on top means it stays visible where it is. The two sounds cover the gap instead. |
| Storing the pill position in pixels | Restores exactly what was dragged. An offset taken from a large screen lands off the edge of a smaller one, and Windows scaling differs per monitor, so it would break the moment a laptop was undocked. Fractions of the working area survive both. |
| A grace period: the pill may take focus, then puts it straight back | Far easier to build, and on most clicks nobody would notice. A person cannot see a theft that lasts a moment, and whatever they typed inside it is gone with nothing on screen saying so. A promise that holds most of the time is a different product from the one this record describes. |
| Telling the person when the pill has taken focus | Cheapest of all, and honest. It hands the person the problem instead of solving it, and the problem is the single thing the app exists to get right. |
| Dropping the drag and making the pill inert to the mouse | Weighed properly on 2026-08-29, not dismissed. It makes the refusal structurally true rather than carefully defended: nothing can click a window the mouse passes straight through, so there is no layer left to get wrong. It costs AC-24, and a pill that has landed on top of the thing you are reading then has no way out, which is the same objection that ruled out a fixed pill. Windows' own on screen keyboard is draggable and does not take focus, so the guarantee looks available without paying that. Kept as the named next move if the drag cannot be made to hold. |
| Moving the pill only from a settings screen | Keeps a way out while the pill is never touchable during dictation, which is the only time focus matters. The settings screen does not exist until milestone 5 and AC-24 is a milestone 1 promise, so this leaves the pill unmovable for four milestones. |
| Treating the hotkey stalling after a click as the same bug as the focus theft | They appear together and one fix might well cover both. The hook is a machine wide one that is called whatever window is focused, so the link is a guess. Fixing on the guess and proving once would let a second fault ship behind a passing check. They are investigated together and proved separately. |
| Leaving history entirely to plan row 5 | Keeps this feature to the bare tracer bullet, but row 5 would then have to reach back into dictation code to start recording, which is exactly the cross feature edit the architecture rules forbid. |
| Giving the pill one more clickable area, so it can carry the privacy link | Keeps every error where the design system first put it, and the pill is already the thing the person is looking at. It reopens the fix for the AC-27 focus defect, whose most important layer has no automated test, to buy a surface for a message that AC-15 says must appear when there is no pill at all. Expensive, and aimed at the wrong error. |
| An error pill with no action, and the instruction in words only | Cheapest of the four, and it keeps the pill inert. It costs AC-15's link outright, and it puts a pill on screen when the microphone did not open, which breaks the one thing the pill means. |
| A Windows notification for the microphone error | Needs no screen and survives the person not looking at the app. AGENTS.md puts notifications out of scope, so this is a different decision wearing a small hat. |
| Matching the Windows access-denied code inside the error message | The failure message carries the access-denied number, and that number is stable across languages. But it depends on the wording format of a library this project does not control, and the same number can occasionally mean an access problem that is not the privacy setting, which would send someone to a settings page that has nothing wrong on it. |
| Matching the code and reading the switches, either one counts | Slightly more robust against Windows or the library changing. Two mechanisms to keep correct instead of one, for a case the switch check already catches, including the machine-toggle case the live run actually hit. |
| A microphone picker in this record | The device row is already drawn, so it looks like a small addition. It needs a device list, a device stored against the account, a rule for when that device is gone, and a live input level in settings, which opens the microphone from a second place in the app. That is a feature with its own risk section, not a settings row. |
| Deepgram's plain signup page as the get-a-key link | One less query string to keep working, and it survives Deepgram rearranging their console. It lands the person on a signup form and leaves them to find the keys screen themselves, which is the one thing AC-9 sent them for. |
| Checking a pasted key by opening a real streaming socket | The only check that proves what AC-10 actually promises, scope included, so the gap named above would not exist. It opens the microphone path from a second place in the app to have something to send, or sends silence and pays allowance for it, and it turns a setup screen into a dictation session. Too much machinery, and too much risk, for a check. |
| Folding 403, 429 and 5xx into rejected | Three kinds instead of four, and one fewer sentence to write. It tells somebody their key is bad when Deepgram was rate limiting them or having an outage, and sends them off to replace a key that was fine. Same reasoning as the fourth microphone error. |
| Both of AC-13's next steps being Replace key | One action to build and one screen to draw. A person whose allowance has run out gets nowhere by pasting the same key again, and AC-13 asks in as many words for the matching next step for each. |
| Leaving a stale error on screen until the person acts on it | Nothing ever changes behind their back, and the code stays there to be quoted. The window then says dictation cannot start while dictation is running, which is a lie on screen, and the person has no reason to go back and look at it. |
| Clearing every error the moment the microphone opens | One trigger and one rule, the simplest thing to build and to read. An open microphone does not prove a Deepgram allowance is back, so it would take the allowance message away while the problem was still there. |
| Hiding the EchoScribe window once the error clears | Tidier, and the person is dictating in another app anyway. It moves a window they may have deliberately left open, which is an act they did not ask for, and this record only ever brings a window forward in answer to a hotkey that produced nothing. |
| A short note saying dictation is working now, in place of the cleared error | Confirms the fix worked, which is friendly after a person has gone off to Windows or Deepgram to sort something out. It is a component the design system does not hold, so it would owe `/canvas` a drawn row, to say something the words already arriving at their cursor say better. |
| `nova-2` as the model | Cheaper per minute and longer established. Fewer languages, 40+ against nova-3's 70+, which narrows plan row 4 before it has even been looked at, and a worse published streaming error rate, which spends the one thing this app must do well. |
| `flux-general-en` as the model | Built for voice agents, with turn detection built in. Turn detection decides when a person has finished speaking in a conversation, which is the wrong instinct entirely for somebody dictating a paragraph: it would cut them off mid thought. |
| No unfinished wording on the pill, ever | Less transcript surface and nothing new crossing into the interface. It makes the drawn transcript line in `design/registry.md` a lie, and a pill that shows nothing through a long sentence looks dead. The registry drew the grey line on purpose, and the drawn design won. |
| Folding the not-allowed key into `deepgram_key_rejected` | One fewer kind and one fewer sentence. Deepgram accepted this key at the setup screen, so saying it was rejected is false and sends the person off to replace a key that was never the problem. The same fold this record has already refused twice. |
| Reusing `DEEPGRAM_UNREACHABLE` for a dropped connection | One fewer code. Its sentence ends "the key was not checked. Nothing was saved.", which is false about a connection that dropped mid sentence, and changing it breaks the setup screen where it is already right. |
| Dropping the held audio during the reconnect attempt | The structurally safer reading of audio is transient, and simpler. It costs a silent hole in the middle of a sentence with nothing on screen saying where the words went, on a connection that recovered fine. The 5 second cap buys the safety without the hole. |
| Bringing the EchoScribe window forward on a password refusal | Every other mid dictation ending does it. There is nothing here for the person to do, and moving their focus while they are on a password field is the single worst moment in the app to do it. |
| Clipboard paste for the collapsing receivers only | Fast, atomic and known to land intact in the new Notepad. The dictated words would transit the clipboard, a third place any clipboard tool can read, feeding Windows clipboard history and cloud sync unless flagged, against a data rule AGENTS.md holds as a hard limit. Refused 2026-08-31. |
| Clipboard paste everywhere | One typing mechanism instead of two. The same clipboard cost on every dictation, and Ctrl V does not mean paste in every app, so it fixes the new Notepad by breaking apps that keystrokes already handle correctly. |
| Accepting the Notepad collapse as a documented limit | No new code and no second mechanism to own. The failure lands in the default editor of Windows 11, the first app most people will try, and it garbles the very words the app exists to get right. |
| Slower pacing and smaller batches | Already tried live: batches cut from 32 units to 8 with 10 millisecond pauses still collapsed a long phrase. Any receiver stall longer than the pause rebuilds the backlog, so no pace reaches zero. Kept only as the ratified mitigation for marginal receivers. |
| Opening small and growing once, when the first thing to read arrives | Matches both drawings as drawn, and the pill is at its smallest in its quietest moment. It visibly changes size a moment after speech starts, on every dictation; near a screen edge the growth must shift the pill to stay on screen, which bends AC-25's promise that it never moves; and the never-take-focus proof would have to hold at both sizes and across the change. Refused by the user 2026-08-31. |
| Keeping 232x44 for every state and making the words fit | The smallest possible object over the document, and one geometry. The decided sentences do not fit it: wrapping grows the pill and scrolling is forbidden outright by the locked design system, and a sentence a person cannot read is not a sign. |

## Data model

Three tables. The account itself is defined by the sign in feature, plan
row 1, and referenced here.

| Table | Key | Fields | Relationship |
|---|---|---|---|
| `dictation` | `id`, auto | `account_id` text, required. `text` text, required. `started_at` text, required, UTC. `duration_ms` integer, required. | One account has many dictations |
| `dictation_setting` | `account_id` | `hotkey` text, required, one of exactly two values, `double_tap_ctrl` or `double_tap_alt`, defaulting to `double_tap_ctrl`. `sounds_enabled` integer, required, defaulting to on. `pill_x` real, required, defaulting to 0.5. `pill_y` real, required, defaulting to 1.0. `updated_at` text, required. | One account has zero or one |
| `deepgram_credential` | `account_id` | `key_last_four` text, required. `credential_target` text, required. `saved_at` text, required. `last_validated_at` text, may be empty. | One account has zero or one |

Rules that must always hold:

- `account_id` is never empty on any row. Nothing is written until we know
  whose it is.
- The Deepgram key itself is never in this database. Only its last four
  characters, for the masked display, and the name of the Windows
  Credential Manager entry holding the real one.
- No audio, and no partial or in progress transcript, is ever written to
  any table, any log, or any file.
- Deleting an account deletes all three rows and its Credential Manager
  entry.
- `dictation` is indexed on account and start time, newest first, because
  that is the only way plan row 5 will read it.
- History is kept until the person deletes it. Nothing expires on its own.
- `hotkey` is refused on write if it is not one of the two named values. A
  row read back holding anything else is treated as the default, never as
  an instruction to the hook.
- `pill_x` and `pill_y` are fractions between 0 and 1 giving where the
  centre of the pill sits within the working area of a screen. They are not
  pixels and they do not name a screen. Anything outside that range, or
  unreadable, is treated as the default, the same way `hotkey` is.

One migration, creating all three tables.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The signed in account id | AC-16, AC-17, AC-18 | The Clerk session held in the Rust core, per decision 0001. Never passed in from the interface. |
| The Deepgram API key | AC-3, AC-10, AC-13 | Entered by the person on the setup screen, stored in Windows Credential Manager, read only by Rust. |
| Last four characters of the key | AC-12 | `deepgram_credential.key_last_four`, stored when the key is saved. |
| Whether a key exists at all | AC-9 | Presence of a `deepgram_credential` row for the account. |
| The hotkey binding | AC-1, AC-19 | `dictation_setting.hotkey`, defaulting to `double_tap_ctrl` on first run. |
| The set of hotkeys that may be chosen | AC-19, AC-22 | This record. Fixed at two, `double_tap_ctrl` and `double_tap_alt`. Not a setting, not extendable at runtime, and not open to the interface to widen. |
| Which physical keys count as the chosen modifier | AC-19 | This record. Left and right Ctrl both count as Ctrl, left and right Alt both count as Alt. |
| The opening and closing sounds | AC-2, AC-5 | The machine's own sound scheme, played by name through the `windows` crate: the device connected event on open, device disconnected on close. Nothing is bundled and nothing is generated. See Still open for the fallback if they will not play by name. |
| Whether sounds play at all | AC-2, AC-5, AC-21 | `dictation_setting.sounds_enabled`, on for a new account. Off silences both and changes nothing else. |
| 300 millisecond double tap window | AC-1, AC-6 | This record. Fixed, not a setting. Measured from the release of the first tap to the press of the second, so holding either key down does not eat into the window. |
| "No other key in between" | AC-6 | The keyboard hook's own view of the key sequence, held in memory only for the current tap, never stored. |
| Waveform movement | AC-1 | Loudness computed from the live microphone buffer in Rust and sent to the pill as a number. The audio itself never reaches the interface. |
| Where the pill opens within a screen | AC-1, AC-23, AC-24 | `dictation_setting.pill_x` and `pill_y`, fractions of that screen's working area, defaulting to 0.5 and 1.0, which is bottom centre. Rust clamps them so the pill is always wholly inside the working area with a small gap at the edge, which is what makes AC-26 hold on a screen of any size. |
| Which screen the pill opens on | AC-1, AC-23, AC-25 | The screen holding the focused window, asked of Windows at the moment the pill opens. The primary screen if Windows cannot say. Asked once per dictation and never again while the pill is open, so focus moving to another screen does not move it. |
| The working area of that screen | AC-23, AC-26 | Asked of Windows for the chosen screen each time the pill opens. The working area excludes the taskbar wherever the person keeps it, which is what makes bottom centre mean above the taskbar rather than under it. |
| Whether the pill can take focus | AC-3, AC-7, AC-24, AC-27 | This record. It cannot, ever. Fixed, not a setting, and not something a later change may relax. Until 2026-08-29 this row named a single window style as what delivered it. The live run disproved that. What must be achieved instead is the three points in The decision, each proved by clicking the pill for real. |
| Whether the hotkey responds while the pill is on screen | AC-5 | The machine wide keyboard hook, which is called for every keystroke whatever window is focused. It does not read the foreground window and must never start doing so. So if the hotkey stops responding once the pill has been touched, that is a fault on this path in its own right, whatever it looks like from outside. |
| A dragged position | AC-24 | The pill window's own position, read from Windows by Rust when the drag ends and converted to fractions of that screen's working area before it is stored. The interface reports that a drag finished. It never computes or sends coordinates. |
| Which words are final | AC-3, AC-4 | Deepgram's own final flag on each streamed result. Nothing else counts as final. |
| Where the text is typed | AC-3, AC-7 | The window that has focus at the moment each phrase is ready, asked of Windows each time. |
| Whether the focused field is a password field | AC-20 | Windows UI Automation, asked of the focused element at the moment of typing. The property is `IsPassword`, recorded 2026-08-31; there is no second candidate. Best effort, see Still open. |
| 30 second silence, 5 minute cap | AC-8 | This record. Fixed, not settings. Silence means no final wording from Deepgram in that period. |
| Rejected key versus allowance exhausted | AC-11, AC-13 | The error Deepgram returns, distinguished by its own response. The mapping is fixed in The decision: 401 `INVALID_AUTH` is rejected, 402 `ASR_PAYMENT_REQUIRED` is the allowance, no answer at all is unreachable, and anything else is the honest catch-all `deepgram_check_failed`, which names no cause. Until 2026-08-30 this row named the source and stopped there, which could not be acted on. 402 is only ever returned for a transcription request, so the allowance case can never arrive at the setup screen; it is a milestone 4 state. |
| Microphone unavailable, and why | AC-15 | The error the audio layer returns when opening the device, mapped to a named cause. |
| Start time of a dictation | AC-17 | The moment the microphone opened, taken in Rust as UTC. |
| Duration | AC-17 | The moment the microphone closed, minus the start time. |
| Which microphone is opened | AC-1, AC-15, AC-31 | Whichever input device Windows is set to as its default, asked of the system at the moment the microphone opens. This record stores no device and offers no way to choose one. |
| Which of the four microphone errors it is | AC-15, AC-28, AC-29, AC-30 | The error the audio layer returns when opening the device, or, added 2026-08-30, the one its error callback reports for a stream that dies after opening. Both run the same mapping to the same four named kinds. Anything that is none of the three known causes is the fourth, `microphone_unavailable`, and is never dressed up as one of the others. Until 2026-08-30 this row said the blocked kind came from the audio layer's permission-denied kind; disproved live, cpal on Windows never produces it. Blocked is now detected by reading the Windows microphone consent switches, see the detection paragraph in The decision. |
| Whether Windows has microphone access switched off | AC-15, AC-29 | The three consent switches the Windows privacy page writes, read directly from Windows by Rust, read only, and only after the microphone has already failed to open for no named reason. Deny on any of the three means blocked; anything else, including the switches being unreadable, does not. Never stored, never logged beyond the named error kind, never shown to the interface as anything but the kind. |
| The sentence shown for each microphone error | AC-15, AC-28, AC-30 | This record, the four sentence table in The decision. Fixed wording, not a setting, and it never carries a device name, a path, or anything from the audio. Added 2026-08-30: the catch-all kind has a second sentence, "The microphone stopped working.", used only when the failure arrives mid dictation, because the first one says the microphone could not be opened and it had been. Five sentences across the same four kinds. |
| What the pill says when the microphone dies mid dictation | AC-30 | This record. `MIC STOPPED`, held long enough to read, then the pill closes and the closing sound plays. Fixed wording. It never carries a device name, a code or anything from the audio; the code and the action are the window's, not the pill's. |
| Whether a mid dictation failure came from the microphone or from Deepgram | AC-13, AC-14, AC-30 | Which of the two reported it. The audio stream's own error callback for the device, the Deepgram connection for the rest. Added 2026-08-30, because until then the record named only Deepgram causes mid dictation and the device path went to stderr and nowhere else. |
| Where a microphone error is read | AC-15, AC-28, AC-30 | This record. The EchoScribe window, brought to the front. Never the pill, which has no action and never appears when the microphone did not open. |
| The Windows microphone privacy page | AC-15, AC-29 | A fixed literal address held in Rust, `ms-settings:privacy-microphone`, opened through the Windows shell with the already approved `windows` crate. The interface never supplies or sees it, and it is never built from anything. |
| The address a person gets a Deepgram key from | AC-9 | This record. A fixed literal held in Rust, `https://console.deepgram.com/signup?jump=keys`, opened through the system browser by `open_deepgram_signup`, which takes nothing. The interface never supplies or sees it, and it is never built from anything. |
| What checks a pasted key against Deepgram | AC-10, AC-11 | This record. `GET https://api.deepgram.com/v1/auth/token`, with the pasted key in an `Authorization: Token <key>` header. Deepgram's own documented way to test a key. It sends no audio and costs no allowance. A 200 does not prove the key can open a streaming socket, so milestone 4 must treat a scope failure on the live stream as a state of its own. |
| The sentence shown for each key error | AC-11, AC-13 | This record, the four sentence table in The decision. Fixed wording, not a setting, held in one place in Rust so no screen invents its own. Nothing read off the pasted key ever appears in a sentence, and no sentence blames the key when the network was the cause. |
| The one next step for each of AC-13's two causes | AC-13 | This record, the two step table in The decision. Replace key on a rejected key, which opens the AC-9 setup screen. Open Deepgram console on a spent allowance, which opens the address below. |
| The Deepgram console page | AC-13 | This record. A fixed literal held in Rust beside the signup page, `https://console.deepgram.com`, opened through the system browser by `open_deepgram_console`, which takes nothing. The second and last Deepgram address this feature opens, and the third and last outside address of any kind, the Windows privacy page being the other. The interface never supplies or sees it. |
| What clears an error screen once the cause is fixed | AC-28, AC-30, AC-32 | This record, the clearing table in The decision. Each error's own proof that the thing it complained about now works: the microphone opening for the four microphone errors, a key being accepted for the three setup screen key errors, the first finalised words for a spent allowance. Never a timer, never one blanket signal, and never the interface deciding on its own. |
| What is asked of Deepgram on the stream | AC-3, AC-4, AC-33 | This record: `nova-3`, English, punctuation on, interim results on. Settled 2026-08-31. The language stays plan row 4's to widen, per Still open. |
| Unfinished wording on the pill | AC-33 | Deepgram's interim results on the live stream, carried to the pill by `dictation:interim` and read by nothing else. Never typed, never stored, never logged. |
| The sentence and action for a key not allowed to stream | AC-30, AC-32 | This record: the fifth key kind table in The decision, `deepgram_key_not_allowed`. The action reuses `open_deepgram_console()`, so no new outside address. Its live trigger is undecided until the first spike in Still open runs; the 403 mapping in `deepgram_key.rs` is a documented guess until then. |
| The sentence and action for a lost connection | AC-14, AC-30 | This record: the `deepgram_connection_lost` table in The decision. Try again goes through `try_start`, the one way into dictation. |
| Speech during the one reconnect attempt | AC-14 | Held in memory only, capped at 5 seconds, the same cap as the attempt itself, then discarded. Never on disk, never in a log. Settled 2026-08-31. |
| What the pill shows when typing is refused | AC-20 | This record: `BLOCKED_PASSWORD_FIELD` and its one sentence, no action, and the EchoScribe window stays where it is. Settled 2026-08-31. |
| How characters reach the focused window | AC-3, AC-7 | This record, settled 2026-08-31: simulated character keystrokes by default, with the ratified batch of 8 and 10 millisecond pause as mitigation; the direct character channel for receivers on the collapse list. Both run the password check first, and neither ever touches the clipboard. |
| Which receivers are on the collapse list | AC-3 | This record. Exactly one today, the new Windows Notepad. A receiver joins by amendment carrying evidence in the finding's shape, never by a quick addition in code. |
| How a focused receiver is matched against the collapse list | AC-3 | This record, settled 2026-08-31: the package family name of the process behind the focused window, read at the moment of typing. The new Windows Notepad's is `Microsoft.WindowsNotepad_8wekyb3d8bbwe`, read from the machine rather than assumed. Cannot tell means not on the list, so the default keystrokes run. |
| What separates two finalised phrases at the cursor | AC-3, AC-4 | This record, decided by the user 2026-08-31: a single space, typed before every finalised phrase after a dictation's first, through the same one door and behind the same password check. Nothing before the first phrase, and the pill's events are unchanged. |
| How long the pill holds its last words before closing | AC-20, AC-30 | This record, settled 2026-08-31 by the eleventh amendment: 2 seconds, from the moment the words are shown, for every ending that puts words on the pill. Then the pill closes and the closing sound plays. Fixed, not a setting. Not yet built; it lands with the pill's text rendering once `/architect` settles the pill's geometry. |
| The pill's geometry | AC-1, AC-20, AC-23 to AC-27, AC-30, AC-33 | This record, settled 2026-08-31 by the twelfth amendment: one fixed form for every state from open to close, the drawn 472x52 listening form. Sized once at open, never resized while open, and the whole footprint, the 20 second counter included, is clamped inside the working area at open. The 232x44 form is retired as a window size. |

## Interface surface

Everything below is a Tauri command, so the interface asks and Rust
decides. No command takes an account id, because Rust already knows it
from the session. Every command refuses when nobody is signed in.

- `save_deepgram_key(key)` returns the last four characters on success, or
  a named error: rejected by Deepgram, could not reach Deepgram, or the
  check did not succeed and Deepgram did not say why. Used only by the setup
  screen. **Corrected 2026-08-30.** This bullet used to list "no allowance
  left" here and to name no catch-all. An exhausted allowance cannot reach
  this command, because Deepgram only ever returns 402 for a transcription
  request. That kind still exists, in the same four kind table in The
  decision, and it arrives on the live stream in milestone 4.
- `get_deepgram_key_info()` returns the last four characters, when it was
  saved and when it was last checked, or nothing if no key is saved. Never
  returns the key.
- `clear_deepgram_key()` removes both the credential entry and the row.
- `get_hotkey()` returns the chosen hotkey and the two it may be chosen
  from, so the interface renders a list rather than deciding what is
  allowed. `set_hotkey(binding)` takes one of those two and refuses
  anything else outright, which is the only refusal it has left to make.
- `get_dictation_sounds()` and `set_dictation_sounds(enabled)`, for the one
  sound switch.
- **Corrected 2026-08-30.** This bullet used to name a command,
  `pill_drag_finished()`, through which the interface told Rust the person had
  let go of the pill. There is no such command and there will not be one. The
  AC-27 fix moved the whole drag into Rust: the pill's web view is kept out of
  the mouse path entirely, so the interface never sees the mouse and has
  nothing to report. `pill_mouse.rs` runs the drag and stores the position
  itself. Everything the old bullet promised still holds and is now Rust's
  alone: where the window actually ended up is read after the drag, worked out
  as a place within that screen's working area, and stored. There is still no
  getter, because Rust positions the pill itself before showing it, and nothing
  about the position ever crosses into the interface as a number. Found by the
  surface check in `docs/evidence/dictate-with-a-hotkey/report.md`.
- `get_dictation_state()` returns idle, listening, or stopped with a
  reason. Lets the pill recover its state after a reload.
- `open_microphone_privacy_settings()` opens the Windows microphone privacy
  page. It takes nothing and returns nothing. The address is a fixed literal
  in Rust, so the interface can ask for that one page and no other.
- `open_deepgram_signup()` opens `https://console.deepgram.com/signup?jump=keys`
  in the system browser. It takes nothing and returns nothing, and its address
  is a fixed literal in Rust for the same reason. It is AC-9's link out to get
  a key.
- `open_deepgram_console()` opens `https://console.deepgram.com` in the system
  browser, same shape and same reason. It is AC-13's next step when the
  allowance has run out, and it does nothing else.
- `retry_dictation()` tries to open the microphone again and starts dictation
  if it opens, or comes back with the same named error if it does not. It is
  the Try again action on three of the four microphone errors, and it goes
  through the same path the hotkey does, so there is only ever one way in.

Rust sends these events out to the pill. None carries audio or the key.

- `dictation:opened`
- `dictation:level` with a single loudness number, many times a second
- `dictation:text` with one finalised phrase
- `dictation:interim` with the wording Deepgram has not yet settled, for the
  pill's grey line and for nothing else. **Added 2026-08-31, and it is what
  AC-33 rides on.** It exists because `design/registry.md` drew a grey
  interim line that no event here could ever have fed. It is never typed,
  never stored, never logged, and never read by anything but the pill.
- `dictation:closed` with why it closed: you stopped it, silence, the time
  cap, or an error
- `dictation:blocked` when typing was refused, currently only a password
  field
- `dictation:error` with a named kind and a message safe to show
- `dictation:needs_key` when the hotkey was pressed and no Deepgram key is
  saved. It carries nothing, no kind and no sentence, because nothing has gone
  wrong: there is a setup step outstanding and the guided screen explains it.
  It is what AC-9 rides on. It is read in the EchoScribe window and not the
  pill, because no pill appears and no sound plays when there is no key.
  **Added to this list 2026-08-30.** Milestone 3 built this event and this
  section was never amended to name it. Found by the surface check in
  `docs/evidence/dictate-with-a-hotkey/report.md`.

Errors that matter and must each read differently: no key saved, key
rejected, the key not allowed to transcribe live audio, allowance exhausted,
cannot reach Deepgram, the key check failing
for some other reason, connection lost mid dictation, microphone blocked by
Windows, microphone in use by another app, no microphone found, the microphone
failing for some other reason, not signed in. The two "for some other reason"
kinds are the two honest catch-alls, one per side, and neither is ever dressed
up as one of the named causes beside it.

**Added 2026-08-30:** each of those four microphone kinds can also arrive mid
dictation, when the device dies after it opened. Same four kinds and the same
four codes, with one extra sentence for the catch-all and `MIC STOPPED` on the
pill. The decision settles all of it. The list above is unchanged because the
causes are unchanged; what changed is that they were only ever read as reasons
the microphone would not start.

**Where each of those is read, settled 2026-08-30.** The pill says what
happened, in words, and carries no action, because nothing but its grip
answers the mouse. Every action lives in the EchoScribe window, which Rust
brings to the front when it has something the person must act on. A
microphone error is read there and nowhere else, because AC-15 forbids a pill
appearing at all when the microphone did not open. `dictation:error` already
goes to every window, so the EchoScribe window receives it on the capability
it has today and nothing needs widening.

## Risk

**The worst things a malicious person could do.** All four were named and
all four are treated as real.

1. Turn the keyboard hook into a keylogger. The hook sees every keystroke
   on the machine, so a future change, a bug or a compromised dependency
   that logged what it sees would make EchoScribe a keylogger running all
   day. This is the single most dangerous thing in the feature.
2. Steal the Deepgram key and spend the person's allowance.
3. Open the microphone silently, recording someone who does not know.
4. Make the app type something the person did not say, by treating
   transcribed text as anything other than plain characters.

**What is being stored that must be protected.** The Deepgram key, in
Windows Credential Manager, never in the database and never shown in full
after saving. The dictation history, which is a record of things the
person actually said, kept locally and partitioned by account. Nothing
else. No audio, no keystrokes, no record of which apps were dictated into.

**What the system must refuse to do, even if a later request seems to
call for it.**

- Type into a password field. Dictation stops and says it was blocked.
- Write audio to disk or into a log, ever. This survives any future
  request for a debug mode that saves recordings.
- Log, buffer or store any key the hook sees other than the chosen
  modifier. The hook counts taps of Ctrl or Alt and discards everything
  else immediately, without comparing it against anything. No diagnostics
  include other keys. No exceptions. This is why the hotkey may only be one
  of two modifier double taps: any wider set would mean the hook inspecting
  ordinary keystrokes, and that is refused here for good.
- Show the Deepgram key in full after it is saved, in the interface, a
  log, an error message or a support file.
- Let the interface hold the key or the raw audio. Both stay in Rust.
- Open the microphone without the pill being visible.
- Let the pill take focus. Clicking or dragging it must never change which
  window is focused, because the focused window is where the words go.
  Tested live on 2026-08-29 and found broken. The refusal is unchanged by
  that. The fix has to achieve it rather than soften it, and a grace period
  in which focus moves and is then put back does not count as meeting it.
- Write to a Windows privacy setting, ever. The consent switches are read to
  name an error and for nothing else. Changing them is the person's act, on
  the Windows settings page this app can open for them, never the app's.
- Put a device name, a file path, or anything drawn from the audio into an
  error message, an event or a log. A microphone error says which of four
  things went wrong and nothing else about the machine.
- Show a pill when the microphone is not open. The pill is the one visible
  sign that it is, so an error that happens before it opens is read in the
  EchoScribe window instead, never on a pill.
- Treat transcribed text as anything but characters to type. It is
  outside input and is never interpreted.
- Open any outside address other than the three fixed literals this record
  names: the Windows microphone privacy page, Deepgram's signup page and
  Deepgram's console. All three live in Rust, none is built from anything, and
  the interface can ask for those pages and no others. A fourth is an
  amendment, not an addition.
- Keep showing an error the person has already fixed. An error screen is a
  claim that dictation cannot happen, so it clears itself the moment the thing
  it named is shown to work. It never clears on a timer, and it never clears
  on a signal that does not actually prove the cause is gone.
- Store, log, or hand an unfinished transcript to anything but the pill.
  `dictation:interim` is drawn grey and then gone. It never enters a table, a
  file, a log, the history, or the typed output, and no future feature may
  read it. Added 2026-08-31 with AC-33.
- Hold reconnect audio beyond its 5 second cap, or hold it anywhere but
  memory. A dropped connection is never a reason a person's voice touches a
  disk or a log. Added 2026-08-31.
- Put the dictated words on the clipboard. Typing is simulated character
  keystrokes or the direct character channel, never paste, settled
  2026-08-31. A future request for paste is a new decision against a data
  rule AGENTS.md holds as a hard limit, not a tuning.

## Build plan

1. **The hotkey and the pill.** Double tap Ctrl detection with the no
   other key guard, the always on top pill opening and closing with its
   two sounds, and the signed in gate. No microphone yet. Start by proving
   the two system sounds play by name, because they are reached through the
   sound scheme rather than the short documented alias list. If they will
   not, fall back to `SystemAsterisk` on open and `SystemExclamation` on
   close, and record which way it landed. Both are alert sounds and neither
   is as apt, which is why it is a fallback. **Settled 2026-08-29:** the
   scheme's own device event names played on this machine and the fallback
   was never reached. The fallback is kept and is still unproven code.
   Detection is written against a chosen modifier from the start, with Ctrl
   as its value, so milestone 5 adds a setting rather than reworking the
   hook. The pill is placed by
   Rust on the screen holding the focused window, at its stored position
   within that screen, clamped inside the working area; it can be dragged,
   and where it is left is stored. Prove this against two monitors of
   different sizes, and against unplugging one, because AC-26 is the part
   most likely to be wrong. Prove the pill cannot take focus, AC-27, in the
   same pass: drag it while a text editor is focused and confirm the caret
   is still blinking in the editor afterwards. **That pass ran on 2026-08-29
   and failed**, on a plain click with no drag at all, so the proof is owed
   again and is wider than it was. See 1a.

**Step 1a, added 2026-08-29: making the pill genuinely non activating, and
re-proving milestone 1.** `/debug` owns the fault, working to the three
points in The decision and stopping where the stop condition says to. The
re-verify that follows must cover, in one sitting:

- A single click on the pill, no drag, with the caret in a text editor, then
  typing, and the characters landing in the editor.
- The same after a drag.
- The hotkey still responding straight after the pill has been touched, with
  no click anywhere else. That is AC-5, and it is proved on its own rather
  than assumed to follow from the focus fix.
- The two clauses the first run left unobserved, AC-1's focused screen and
  AC-24's carrying across screens. Both need one open of the pill while a
  window on the other monitor is focused.
- The sound fallback: set both device sounds to None in Windows, open and
  close the pill once, hear the two alert sounds, put the scheme back.
  **Ran 2026-08-30 and failed**, and this line was wrong about what a pass
  sounds like. Nothing was heard on open or on close, because a `(None)` entry
  plays silence and still reports success. Fixed in `661ce2f`; see the sound
  bullet in Still open for the whole of it. Two changes here. First, `/develop`
  owes one more piece before this is re-run: the check that a named sound file
  is actually there, settled as point 1 of that bullet, so the fix and the
  file check are proved in one sitting. Second, the pass is corrected. On this
  machine both alert names resolve to the same file, so hearing **one sound on
  open and the same one on close** is a pass. Silence is the only failure.

2. **The microphone and the waveform.** Live capture feeding a loudness
   number to the pill, silence and time cap closing it on their own, and
   every microphone failure reading correctly. Audio is discarded and
   nothing leaves the machine. **Built on 2026-08-29 except the reading
   correctly half**, which had nowhere to be read. See 2a. AC-8's silence cap
   is built, tested and deliberately unarmed until milestone 4 arms it, which
   is what `limits.rs` and its `without_deepgram` constructor are for.

**Step 2a, added 2026-08-30: somewhere to read a microphone error.** Milestone
2 is not finished until AC-15 holds, and AC-15 cannot hold while its message
reaches only an event and the dev console. `/canvas` goes first, because there
is no drawn comp for a microphone error on the EchoScribe window and none may
be invented during a build. Then AC-28, AC-29 and AC-31, in this order:

- `/canvas` adds the microphone error state on the EchoScribe window to
  `design/registry.md`, and corrects the registry's "Error pill" line, which
  says an error pill has "exactly one action" and must now say it has none.
- The four sentences in `microphone.rs::MicError::message` are made to match
  the table in The decision. Three are already right. `no_microphone_found`
  changes to "Windows cannot find a microphone."
- The EchoScribe window is brought to the front on a microphone failure,
  showing the code, the sentence and the one action.
- `open_microphone_privacy_settings()` and `retry_dictation()`.
- Proved live, all four kinds. Blocked is provable by switching microphone
  access off in Windows privacy settings; no microphone found by unplugging
  or disabling every input device. Prove the pill does not appear and neither
  sound plays in each case, and that Try again starts dictation once the cause
  is fixed. AC-30's mid dictation half cannot be proved until milestone 4, so
  it is proved there, not assumed here.

**Step 2b, added 2026-08-30: telling blocked-by-Windows apart for real.** Step
2a was built and its screen is right, but the blocked kind never fires: the
mapping it relied on was disproved the same day. Replace the classification so
that a failure which is not device-busy and not no-device asks the Windows
consent switches, per the detection paragraph in The decision. Read only. The
switches live behind the already approved `windows` crate, which needs its
registry feature switched on; that is a feature of an approved library, not a
new one. Step 2a's live proof then covers this too: with the machine-wide
microphone toggle off, the screen must say `MICROPHONE_BLOCKED_BY_WINDOWS`
with Open Windows settings as the one action, and the same again with only
the desktop-apps toggle off.

3. **The Deepgram key.** The guided setup screen, checking a key against
   Deepgram, storing it in Credential Manager, the masked display in
   settings, and every key related error reading correctly. **Built on
   2026-08-30**, against five decisions this record owed and now holds, above
   under "The Deepgram key". Two halves of it are deliberately not finished
   here and must not be read as met: AC-12's masked row has nowhere to live
   until the settings screen exists in milestone 5, so the last four
   characters are stored and returned and nothing displays them; and AC-13
   cannot be proved live until something streams, so its four kinds, codes,
   sentences and actions are built and unit tested and milestone 4 wires the
   trigger. Evidence:
   `docs/evidence/dictate-with-a-hotkey/milestone-3-decisions-owed.md`.
**Step 3a, added 2026-08-30: clearing an error the person has already fixed.**
AC-32, and the only part of this amendment that is not already built. Today
the EchoScribe window leaves an error screen on two triggers, a successful Try
again and a key being saved, both of which are buttons on the screen itself.
Neither fires when the person goes off to Windows or to Deepgram, fixes the
cause there, and comes back with the hotkey. The window then sits behind
everything saying dictation cannot start while it is running.

- The shell also leaves an error screen on `dictation:opened`, which already
  reaches every window and needs no new event and no capability change.
- Only the four microphone kinds clear on that signal. The allowance kind
  clears on the first finalised words and is milestone 4's, per the clearing
  table in The decision. Do not wire it to the microphone opening to get it
  working sooner.
- The window is not brought forward, not hidden and not moved. The person is
  dictating into another app, and this record brings a window forward for one
  reason only.
- Proved live, on the kind that most invites the round trip: switch microphone
  access off in Windows, press the hotkey, read the error, switch access back
  on without touching EchoScribe, press the hotkey again. The words land, and
  the EchoScribe window behind them is no longer showing the error.

4. **Transcription and typing.** Streaming to Deepgram, typing finalised
   phrases as simulated keystrokes at the focused cursor, the password
   field refusal, and the reconnect and connection lost behaviour. It also
   carries three things milestone 3 handed it, each named here so none is
   forgotten: a scope failure on the live stream is a state of its own, because
   a key the setup check accepted may still not be allowed to open a socket;
   AC-13's live 401 and 402 are wired to the four kinds milestone 3 built, and
   proved there; and the allowance error clears on the first finalised words,
   which is the one clearing trigger milestone 3 could not build.

   **Added 2026-08-30: a fourth source of a mid dictation failure, and it is
   not Deepgram's.** The microphone dying after it opened, seen live during the
   verify sitting. The ending, the classification, the changed sentence and the
   race with AC-8's silence cap are all settled in The decision, under "The
   microphone dying after it opened". `/canvas` goes first for the two states
   it names, the pill's `MIC STOPPED` and the catch-all's second sentence, the
   same way step 2a needed a drawn screen before it could be built. Prove it
   live the way it was found: with the pill up and the meter moving, switch
   Windows' desktop-apps microphone access off. The pill must say `MIC STOPPED`
   and close, the closing sound must play, and the EchoScribe window must come
   forward saying `MICROPHONE_BLOCKED_BY_WINDOWS` with Open Windows settings as
   its one action. Switch access back on, press the hotkey, and the error must
   clear itself, which is AC-32's path and costs nothing extra in the same
   sitting. All of it is part of AC-30's proof, not a criterion of its own.

**Step 4a, added 2026-08-31: typing that survives a stalling receiver.** The
live run proved the new Windows Notepad collapses backlogged injected
keystrokes, evidence in
`docs/evidence/dictate-with-a-hotkey/finding-new-notepad-collapses-injected-unicode.md`,
and The decision now holds the cure, under "How characters reach a focused
window". In order:

- The spike first: post the finding's own 205 character burst into the new
  Notepad through the direct channel and compare exact strings, the same
  proof the finding used, with EchoScribe not running. Intact means build it.
  Anything else means stop and take the choice back to the user, because the
  two options left, clipboard and documented limit, each cost something only
  the user can spend.
- Then the channel itself, behind the same one door typing already has, with
  the receiver list from The decision deciding which mechanism runs, and the
  password check first on both, exactly as today.
- Proved live the way the fault was found: one real dictation into the new
  Notepad and the same one into a classic edit control, both arriving
  character perfect, compared as strings and not by eye.

5. **History and settings.** The three tables and their migration, saving
   each finished dictation against the account, per account separation, the
   two hotkey choices as a list in settings, and the sound switch. Both
   settings take effect immediately and are still in force after a
   restart.

## What this makes harder

- **A keyboard hook runs whenever the app is running.** It is the most
  security sensitive code in the project, it will show up in security
  tooling and antivirus heuristics, and every future change near it needs
  the same scrutiny as the first one.
- **Text follows the cursor, so it can land somewhere unintended.** A
  popup or another app stealing focus mid sentence sends words there. This
  was chosen deliberately over locking to the starting window. The visible
  pill and the password field refusal are the only mitigations.
- **Double tap can misfire.** The no other key guard covers Ctrl C then
  Ctrl V, but a genuine double tap of Ctrl alone, for any reason, starts
  dictation.
- **The pill can end up on a screen you are not looking at.** It is placed
  once, on the screen you were typing on, and stays there for the whole
  dictation. Click into a window on another monitor and the only visible
  sign the microphone is open is behind you. The opening and closing
  sounds, AC-2, are what cover that, which is a second reason the sound
  switch in AC-21 silences sounds and nothing else.
- **Typing only finalised wording means a slight lag.** Words arrive in
  phrases a beat behind the voice, not one by one. This is the price of
  never taking typed text back.
- **Simulated keystrokes are slower than pasting** and a small number of
  apps handle fast simulated input badly. Those apps will need finding by
  use, not by reading code. **Come true, 2026-08-31.** Finding by use found
  the first one, and it is the new Windows Notepad, the default editor of
  Windows 11. The cure is the per application direct channel in The
  decision. This line stays, because the next such app is still found the
  same way, and it now has somewhere to go when found: the collapse list,
  joined by amendment.
- **Typing now has two mechanisms, and a list deciding between them.**
  Simulated keystrokes by default, the direct character channel for
  receivers proven to collapse. Every future misbehaving receiver is a
  diagnosis against evidence in the finding's shape and then an amendment,
  never a quick addition to a list in code. Two paths also means two
  proofs: any change to typing is proved in a classic edit control and in
  the new Notepad, not one or the other.
- **A partial transcript now crosses into the interface.** Only to the
  pill, only to be drawn grey, and never stored or logged, but it is more
  transcript surface than existed before AC-33, and every future change
  near the pill inherits the duty to keep the grey line display only.
- **The hook, the typing and the credential vault are Windows only.** The
  microphone, via cpal, and the database carry to Mac. The rest is a
  second implementation behind the platform boundary AGENTS.md requires.
- **The Deepgram setup step stands between a new person and their first
  word.** Decision 0001 already flagged this. AC-9 through AC-11 make that
  screen load bearing.
- **The pill is a sign and never a control, so every action costs a window.**
  Anything a person can do about an error has to be somewhere they can click,
  and that is the EchoScribe window coming to the front. Each new error worth
  acting on is therefore a screen `/canvas` has to draw, not a line of text
  added to the pill. That is the price of the pill never taking focus, and it
  is paid every time, not once.
- **Bringing the window forward interrupts the app the person was in.** It is
  only ever done in answer to a hotkey they just pressed that produced
  nothing, so it is never a surprise, but it does move focus out of their
  document. Nothing else in this feature may bring a window forward.
- **A key that passes the setup check can still fail on the stream.** The
  check that AC-10 rests on proves the key is real and Deepgram is reachable.
  It cannot prove scope without opening a socket and paying for it, so the
  setup screen can say yes to a key that dictation later says no to. That
  gap is permanent, not a milestone 3 shortcut, and every future change near
  the stream has to keep a scope failure as a state a person can read rather
  than a crash.
- **Every error screen now owes a way out that nobody presses.** An error
  clears when the thing it named is shown to work, and that proof arrives from
  wherever dictation happens to be running rather than from the screen itself.
  So each new error worth showing costs a wording, an action, and a signal
  that says it is over. Leave the third one out and the app sits there telling
  somebody about a problem they fixed twenty minutes ago.
- **The pill refusing focus is not free, and now never will be.** This
  record first wrote it down as one window style set once, and it turned out
  to need care at three separate layers. Any future change to the pill, its
  markup, its styling, or the library hosting it can break it again without
  anything failing to build. It is the one part of this feature where a
  clean build proves nothing, so a live click test belongs in every pass
  that touches the pill.
- **The pill is wide from its first instant.** One geometry means the 472
  pixel listening form is what appears before a word has been said, roughly
  twice the width the comp drew for that moment. That is the price of
  nothing over the document ever changing size, paid on every dictation,
  and it is why the retired 232x44 form must not quietly come back as an
  optimisation.

## Still open

- **How widely a password field can actually be detected.** Windows UI
  Automation reports this reliably for most modern apps and browsers, and
  not at all for some older or custom ones. When it cannot tell, the app
  types normally, because refusing everywhere it is unsure would break
  dictation in ordinary apps. Settle the real coverage by testing during
  milestone 4 and record what was found.

  **Re-pointed 2026-08-31, still open.** That last instruction is stale:
  milestone 4 is built and did not settle it. What milestone 4 did settle
  is the property, `IsPassword`, recorded in The decision, and the refusal
  itself, `BLOCKED_PASSWORD_FIELD`. The coverage half is genuinely
  unanswered. What is owed is a live sitting with the user at the machine
  and a list of real applications: Chrome, Edge, Firefox, a Windows
  credential prompt, a password manager, and at least one older desktop
  app. It changes no behaviour, because the rule for cannot tell is fixed;
  it measures how often AC-20's promise actually holds, which is worth
  knowing before anybody trusts it.
- **Whether the pill needs its own design pass.** The locked design system
  covers ordinary screens. An always on top floating pill with a live
  waveform may not be covered by it. Settle when milestone 1 starts, and
  run `/canvas` first if it is not.
- **Whether those two device sounds can be played by name. Closed
  2026-08-29. They can.** The machine's own scheme gave the device connected
  sound on open and the device disconnected sound on close. The
  `SystemAsterisk` and `SystemExclamation` fallback was never reached. Evidence:
  `docs/evidence/dictate-with-a-hotkey/spikes-owed-to-record-0002.md`.

  **Corrected 2026-08-30, and this is the part that was wrong.** This bullet
  used to say the fallback was kept "because on a machine whose scheme has no
  device sounds it is the only thing keeping the open and close audible". It
  was keeping nothing audible. It had never run and it could not run. Setting a
  device sound to `(None)` on the Windows sound page leaves the scheme entry
  present and empty; `PlaySoundW` then plays silence and still reports success;
  and `sound.rs` read that success as proof a person had heard something. So
  the preferred name won every time and both dictation sounds went quiet, on
  exactly the machine the fallback existed for. Observed live 2026-08-30, all
  three ways, evidence in
  `docs/evidence/dictate-with-a-hotkey/AC-2-sound-fallback-never-fires.md`.
  Fixed in commit `661ce2f`: the scheme entry must name a sound, read from the
  registry, and `PlaySoundW` must then succeed, because a name absent from the
  scheme altogether does report failure honestly. Two checks, not one. The
  general lesson is now standing rule 14 in AGENTS.md.

  **Three things that fix did not decide, settled here 2026-08-30.**

  1. **A named sound file that has since been deleted is checked for, and a
     path that cannot be checked is played anyway.** Same class of hole, one
     machine along: an entry that names a file nobody deleted and an entry that
     names a file somebody did look identical to the fix above, and the second
     plays silence. So the path is read from the scheme, environment variables
     in it are expanded, a bare file name is resolved against the Windows media
     folder, and a file has to be there. If the path cannot be resolved, or
     cannot be checked, the name is tried and Windows gets the last word. It is
     never read as silence. That is deliberate and it is the same rule
     `sound.rs` already holds for an entry it cannot read, and `consent.rs` for
     a switch it cannot read: not knowing is not evidence. The cost is named
     rather than hidden. Windows resolves these entries in ways this app does
     not fully control, so a path form we fail to resolve quietly degrades to
     today's behaviour, which is wrong-but-safe rather than silent. `/develop`
     owns this, in build plan step 1a, before the re-verify.

  2. **The fallback is kept, and the two alert sounds may well be the same
     sound.** On this machine `SystemAsterisk` and `SystemExclamation` both
     resolve to `Windows Background.wav`, so once the fallback fires the open
     and close are identical. Raised as finding 2 of
     `docs/evidence/dictate-with-a-hotkey/report.md`. Kept anyway: one sound
     heard twice still says something happened, the pill says which, and the
     alternative on a scheme with no device sounds is hearing nothing at all.
     Hunting for a pair of alert names that differ was weighed and rejected,
     because which names collapse onto one file is a Windows convention this
     project does not control and the next machine may collapse a different
     pair. What this does change is the build plan's step 1a wording, which
     expected to "hear the two alert sounds": one sound, heard on open and
     again on close, is a pass. Silence is the only failure.

  3. **The re-verify is still owed.** Nothing above has been proved against the
     fixed build. Step 1a's sound item has not been run since `661ce2f`, so on
     this machine the fallback remains code that has never made a sound. It
     belongs to `/check verify`, with the file check from point 1 built first
     so both are proved in one sitting.
- **Whether the pill can be clicked and dragged without taking focus.
  Closed 2026-08-29, and the answer was no.** Not on the mechanism this
  record used to name. See The decision for what replaced it. What is open
  now is narrower and different: whether the three points named there can
  all be made to hold for a pill that has a web view inside it. Windows' own
  on screen keyboard says the shape is possible; it does not say it is
  possible here. `/debug` finds out, the stop condition says when to stop
  trying, and the answer gets recorded here either way.

  **That narrower question is closed too, 2026-08-30, and the answer is yes.**
  All three points hold for a pill with a web view inside it. `/debug` found
  the cause and the fix landed: the web view is kept out of the mouse path
  entirely, and `pill_mouse.rs` runs the drag and stores the position in Rust,
  so the interface never sees the mouse. Re-proved live with Notepad focused
  and never clicked again for the whole test, which is what makes it a real
  test: a click on the grip, a click on the inert bars, and a drag a hand's
  width across the screen, with typing after each one landing in Notepad every
  time and Notepad's title bar staying active throughout. The hotkey also
  answered on the first double tap straight afterwards with no click anywhere,
  which was the knock-on AC-5 fault from the same failed run. Evidence:
  `docs/evidence/dictate-with-a-hotkey/AC-27-pill-no-longer-takes-focus.md`.
- **Whether a key the setup check accepts can always open a streaming
  socket.** `GET /v1/auth/token` proves the key is real and Deepgram is
  reachable. It does not prove scope. A key without the right permission
  passes the setup screen and fails on the live stream, and how Deepgram
  reports that, and how often it happens to a key made the ordinary way, is
  not known. Settle it in milestone 4 by pointing a deliberately narrow key at
  the stream, and record what came back. Until then the setup screen's green
  light means the key is real, not that dictation will work.

  **Re-pointed 2026-08-31, still open.** Milestone 4 is built and did not
  settle this either. It built the kind, `deepgram_key_not_allowed`, its
  code, its sentence and its action, and wired a trigger that is a
  documented guess: `deepgram_key.rs` maps a 403 on the stream to it and
  says in the code that it is a guess. Deepgram's own error page, checked
  2026-08-30, documents 401 for both an invalid key and an insufficient
  one, so whatever tells them apart has to be read off a live response.
  Still owed, and it needs the user: a key deliberately made without
  streaming permission in their own Deepgram console, pointed at the
  stream, and what comes back recorded verbatim.
- **The language the transcription runs in** is fixed to English here.
  Plan row 4 owns making it a choice, and will add a language field to
  `dictation`.

## New libraries

Approved during this decision, none installed yet. Each is load bearing,
not convenience.

| Library | For | Why this one |
|---|---|---|
| `windows` | The keyboard hook, simulating keystrokes, asking about the focused field | Microsoft's own Rust bindings. One dependency covering three Windows specific jobs. |
| `cpal` | Capturing the microphone | The established Rust audio crate, and it carries to Mac unchanged. |
| `deepgram` | Live transcription | Deepgram's official Rust SDK. Young at version zero, so expect breaking changes between versions. |
| `keyring` | The Windows Credential Manager entry | The standard crate for this, with a Mac equivalent behind the same interface. |
| `rusqlite` | The three tables | Direct SQLite from the Rust core, which keeps decisions in Rust as the architecture rules require. |
| `tokio` | Running the audio stream and the connection at once | Required by the Deepgram SDK regardless. |
