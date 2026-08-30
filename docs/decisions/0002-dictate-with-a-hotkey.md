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
| Whether the focused field is a password field | AC-20 | Windows UI Automation, asked of the focused element at the moment of typing. Best effort, see Still open. |
| 30 second silence, 5 minute cap | AC-8 | This record. Fixed, not settings. Silence means no final wording from Deepgram in that period. |
| Rejected key versus allowance exhausted | AC-13 | The error Deepgram returns, distinguished by its own response. |
| Microphone unavailable, and why | AC-15 | The error the audio layer returns when opening the device, mapped to a named cause. |
| Start time of a dictation | AC-17 | The moment the microphone opened, taken in Rust as UTC. |
| Duration | AC-17 | The moment the microphone closed, minus the start time. |
| Which microphone is opened | AC-1, AC-15, AC-31 | Whichever input device Windows is set to as its default, asked of the system at the moment the microphone opens. This record stores no device and offers no way to choose one. |
| Which of the four microphone errors it is | AC-15, AC-28, AC-29 | The error the audio layer returns when opening the device, mapped to one of four named kinds. Anything that is none of the three known causes is the fourth, `microphone_unavailable`, and is never dressed up as one of the others. Until 2026-08-30 this row said the blocked kind came from the audio layer's permission-denied kind; disproved live, cpal on Windows never produces it. Blocked is now detected by reading the Windows microphone consent switches, see the detection paragraph in The decision. |
| Whether Windows has microphone access switched off | AC-15, AC-29 | The three consent switches the Windows privacy page writes, read directly from Windows by Rust, read only, and only after the microphone has already failed to open for no named reason. Deny on any of the three means blocked; anything else, including the switches being unreadable, does not. Never stored, never logged beyond the named error kind, never shown to the interface as anything but the kind. |
| The sentence shown for each microphone error | AC-15, AC-28 | This record, the four sentence table in The decision. Fixed wording, not a setting, and it never carries a device name, a path, or anything from the audio. |
| Where a microphone error is read | AC-15, AC-28, AC-30 | This record. The EchoScribe window, brought to the front. Never the pill, which has no action and never appears when the microphone did not open. |
| The Windows microphone privacy page | AC-15, AC-29 | A fixed literal address held in Rust, `ms-settings:privacy-microphone`, opened through the Windows shell with the already approved `windows` crate. The interface never supplies or sees it, and it is never built from anything. |

## Interface surface

Everything below is a Tauri command, so the interface asks and Rust
decides. No command takes an account id, because Rust already knows it
from the session. Every command refuses when nobody is signed in.

- `save_deepgram_key(key)` returns the last four characters on success, or
  a named error: rejected by Deepgram, no allowance left, or could not
  reach Deepgram. Used only by the setup screen.
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
- `pill_drag_finished()` tells Rust the person has let go of the pill. Rust
  reads where the window actually ended up, works out its place within that
  screen's working area, and stores it. There is no matching getter,
  because Rust positions the pill itself before showing it. Nothing about
  the position ever crosses into the interface as a number.
- `get_dictation_state()` returns idle, listening, or stopped with a
  reason. Lets the pill recover its state after a reload.
- `open_microphone_privacy_settings()` opens the Windows microphone privacy
  page. It takes nothing and returns nothing. The address is a fixed literal
  in Rust, so the interface can ask for that one page and no other.
- `retry_dictation()` tries to open the microphone again and starts dictation
  if it opens, or comes back with the same named error if it does not. It is
  the Try again action on three of the four microphone errors, and it goes
  through the same path the hotkey does, so there is only ever one way in.

Rust sends these events out to the pill. None carries audio or the key.

- `dictation:opened`
- `dictation:level` with a single loudness number, many times a second
- `dictation:text` with one finalised phrase
- `dictation:closed` with why it closed: you stopped it, silence, the time
  cap, or an error
- `dictation:blocked` when typing was refused, currently only a password
  field
- `dictation:error` with a named kind and a message safe to show

Errors that matter and must each read differently: no key saved, key
rejected, allowance exhausted, cannot reach Deepgram, connection lost mid
dictation, microphone blocked by Windows, microphone in use by another
app, no microphone found, the microphone failing for some other reason, not
signed in.

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
   settings, and every key related error reading correctly.
4. **Transcription and typing.** Streaming to Deepgram, typing finalised
   phrases as simulated keystrokes at the focused cursor, the password
   field refusal, and the reconnect and connection lost behaviour.
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
  use, not by reading code.
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
- **The pill refusing focus is not free, and now never will be.** This
  record first wrote it down as one window style set once, and it turned out
  to need care at three separate layers. Any future change to the pill, its
  markup, its styling, or the library hosting it can break it again without
  anything failing to build. It is the one part of this feature where a
  clean build proves nothing, so a live click test belongs in every pass
  that touches the pill.

## Still open

- **How widely a password field can actually be detected.** Windows UI
  Automation reports this reliably for most modern apps and browsers, and
  not at all for some older or custom ones. When it cannot tell, the app
  types normally, because refusing everywhere it is unsure would break
  dictation in ordinary apps. Settle the real coverage by testing during
  milestone 4 and record what was found.
- **Whether the pill needs its own design pass.** The locked design system
  covers ordinary screens. An always on top floating pill with a live
  waveform may not be covered by it. Settle when milestone 1 starts, and
  run `/canvas` first if it is not.
- **Whether those two device sounds can be played by name. Closed
  2026-08-29. They can.** The machine's own scheme gave the device connected
  sound on open and the device disconnected sound on close. The
  `SystemAsterisk` and `SystemExclamation` fallback was never reached, so it
  is in the app and has never run. It is kept, because on a machine whose
  scheme has no device sounds it is the only thing keeping the open and
  close audible, and that sound is half of how a person knows the microphone
  is on. Proving it is now a step in the re-verify, build plan 1a. Evidence:
  `docs/evidence/dictate-with-a-hotkey/spikes-owed-to-record-0002.md`.
- **Whether the pill can be clicked and dragged without taking focus.
  Closed 2026-08-29, and the answer was no.** Not on the mechanism this
  record used to name. See The decision for what replaced it. What is open
  now is narrower and different: whether the three points named there can
  all be made to hold for a pill that has a web view inside it. Windows' own
  on screen keyboard says the shape is possible; it does not say it is
  possible here. `/debug` finds out, the stop condition says when to stop
  trying, and the answer gets recorded here either way.
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
