# Component registry

The approved component census. `/canvas drift` patrols against this list.

**A component not on this list does not exist.** If a screen needs one that
is not here, that is a design decision: stop, run `/canvas`, add it here
first, then build it.

Status: `drawn` means the comp defines it and nothing has been built yet.
`built` means it exists in `src/` and matches. Everything is `drawn` today,
because no screen has been built with this system yet.

## Shell

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Nav rail | dashboard | drawn | Dark, persistent, left. Not tabs. Account pinned to the bottom. |
| Nav item | nav rail | drawn | Two states, resting and active. Active gets the raised rail fill. |
| Nav count | nav item | drawn | Faint ink. Redundant to the label, never the only signal. |
| Section sub-nav | settings | drawn | Horizontal, inside the white surface, not in the rail. |
| Account block | nav rail foot | drawn | Initials, name, and on Settings the mail address and signed-in date. |

## The pill

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Pill shell | floats over other apps | drawn | 232x44 at its smallest. Fixed geometry across every state. |
| Drag grip | pill | drawn | Grabbable from any edge. 44px hit area, grab cursor. |
| Status label | pill | drawn | Mono, uppercase. The words half of every two-signal state. |
| Breathing ring | pill | drawn | Violet. Mic open only. Pairs with the status label, never alone. |
| Level meter | pill | drawn | 18 bars, amplitude only, flat at silence. Never a spectrogram. |
| Transcript line | pill | drawn | One line. Final text ink, interim grey with a dotted rule. |
| Left fade truncation | transcript line | drawn | Truncates from the left. Never scrolls, never grows. |
| Elapsed and word count | pill | drawn | Appears after 20 seconds. |
| Error pill | replaces pill body | drawn | Same geometry. A mono code and one sentence, and **no action at all**. Corrected 2026-08-30: this row used to say "exactly one action". The pill is a sign, never a control, and nothing but its grip answers the mouse (record 0002, amended 2026-08-30, AC-30). Every action a person can take lives in the EchoScribe window instead. An error that happens before the microphone opens has no pill at all, because a pill on screen means the microphone is open. |

## History

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Search field | history | drawn | Anchored in the rail area, one fixed place. |
| Filter chip | history | drawn | App and date. Resting and applied. |
| Result count | history | drawn | Plain text beside the query. |
| Dictation row | history list | drawn | Timestamp, duration and word count, the text, source app, language tag, copy. |
| Language tag | dictation row | drawn | Mono. A literal code, so mono is correct. |
| Copy action | dictation row | drawn | The only action on a row. |
| History empty state | history | drawn | The first screen a new user sees. Hotkey keys, one paragraph, one action. |

## Settings

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Setting row | settings | drawn | Label, current value, one action on the right. |
| Hotkey display | dictation settings | drawn | Keycaps, plus the hold and tap explanation beneath. |
| Device row | dictation settings | drawn | Device name, connection badge, live input level. |
| Language list | language settings | drawn | Rows with a default marker, plus an add action. |
| Default marker | language list | drawn | Mono tag. |
| Secret field | transcription settings | drawn | Masked, trailing fragment, replace and remove, locally-stored wording. |
| Secret field, empty | first run | drawn | Required badge, paste field, verify action, blocker sentence, help link. |

## First run

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Onboarding window | first run | drawn | 760x540, dark throughout. |
| Blocker card | first run and settings | drawn | Violet wash. States that dictation is off until the key is set. |
| Ready confirmation | first run | drawn | Names the key, the microphone and the language back to the user. |

## Sign in

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Sign-in window | sign in | drawn | Reuses the onboarding window shell: 760x540, dark throughout (`--color-onboarding`), one centered column, no nav rail. The only screen on a fresh install and after sign out (record 0003 AC-1, AC-9). Holds the brand lockup, a 23px heading, one short paragraph in the first-run body colour, one action, and a reassurance caption in the first-run caption colour. |
| Sign-in states | sign-in window | drawn | Four, each drawn not assumed. **Initial**: the empty state, the first thing a new user sees. **Waiting** (`signing_in`): the loading state, waiting indicator plus a cancel action while the browser is open. **Failed** (`auth:sign_in_failed`): the error state, a sign-in error line plus one retry action. **Returned**: a sign-in notice line above the initial content, saying why the person is back here. |
| Sign-in action | sign-in window | drawn | One primary button, on dark, labelled "Sign in". It opens the system browser and nothing else. No email field, no code field, no provider button in the app: all credential entry is on Clerk's hosted pages (record 0003 AC-2, AC-3, AC-17). Pressing it does not leave this view at once: `start_sign_in` checks that Clerk answers first and can take up to five seconds, so the button goes busy, labelled "Checking connection", and the screen moves to the waiting state only when that call comes back (record 0003 AC-15). On the failing path the button leaves busy and the screen becomes the failed state, with no browser ever opened. This supersedes the in-app form in the comp's "Sign in (Clerk)" mock, which predates record 0003; the mock's dark window, type and button treatment still stand. |
| Waiting indicator | sign-in window | drawn | Neutral, never violet, because violet only ever means an open microphone. A quiet pulse in the first-run caption colour with a status sentence beside it. Under `prefers-reduced-motion` the pulse holds still and the words still carry the state. |
| Sign-in error line | sign-in window | drawn | The same information order as an error pill and a microphone error line: a mono code, then one plain sentence of cause. Unlike the pill, which carries no action at all, this line is on a window that takes clicks, so exactly one action follows it. Code and left edge in `--color-danger-on-dark`, sentence in the first-run body colour. One wording per named reason in record 0003: browser closed, timed out, security check failed, Clerk rejected sign-in, could not reach Clerk. |
| Sign-in notice line | sign-in window | drawn | Why the person returned to sign in. Neutral, in the first-run caption colour, for "you signed out". Attention, in `--color-warning-on-dark`, for "your session was ended elsewhere". A marker and words, never colour alone. |

## Microphone error

Record 0002, amended 2026-08-30, AC-28 to AC-30. The first screen in the
EchoScribe window that is not sign in. There is no comp for it, so it is
drawn here out of the shell that already exists, and nothing below invents
a colour, a size or a shape the system does not already hold.

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Microphone error screen | echoscribe window | drawn | Shown when the microphone will not open. Reuses the sign-in and onboarding window shell exactly: 760x540, dark throughout (`--color-onboarding`), one centered column, no nav rail, the brand lockup at the top. It is a whole-window interruption, not a panel, and it stays one when the rail-and-surface shell arrives, because it answers a hotkey press that got nothing and has exactly one way forward. Rust brings the window to the front; no pill appears and neither sound plays (AC-28). Same shape as the AC-9 screen for a missing Deepgram key: a hotkey that could not start dictation, and a window that says why. |
| Microphone error line | microphone error screen | drawn | The same three parts in the same order as the sign-in error line: a mono code, one plain sentence of cause, then the one action beneath. Code and left edge in `--color-danger-on-dark`, sentence in the first-run body colour (`--color-ink-on-dark-secondary`). Two signals and words are one of them, so the danger colour never carries the meaning alone. Exactly four wordings exist, fixed by record 0002's table under "The four microphone errors" and not editable at build time. The sentence never carries a device name, a path, or anything read off the audio. |
| Microphone error code | microphone error line | drawn | Mono, uppercase, the error kind itself so a person can quote it: `MICROPHONE_BLOCKED_BY_WINDOWS`, `MICROPHONE_IN_USE_BY_ANOTHER_APP`, `NO_MICROPHONE_FOUND`, `MICROPHONE_UNAVAILABLE`. Four and only four. A failure that is none of the first three is the fourth, and is never dressed up as one of the others. |
| Microphone error action | microphone error screen | drawn | Exactly one, always, and never two (AC-29). The primary button, on dark, in both wordings. "Try again" on three of the four kinds. "Open Windows settings" on `microphone_blocked_by_windows`, which opens the Windows microphone privacy page and does nothing else. No secondary button, no dismiss, no link away: the window's own title bar close is the way out and is not a designed control. |
| Microphone error, waiting | microphone error action | drawn | Try again opens the microphone, which takes a moment, so the wait is drawn rather than assumed. The button takes the "Primary button, on dark, busy" primitive unchanged: same geometry, label swapped to "Opening microphone", light fill breathing on the 1.4s cadence, never violet, holding still under `prefers-reduced-motion` with the label still carrying the state. "Open Windows settings" has no busy state; it hands off to Windows and the screen stays as it is. |
| Microphone error, retried | microphone error screen | drawn | Both endings are drawn. The microphone opens: the error clears and the window leaves this screen. It fails again: the screen stays and shows the code and the sentence for whatever the failure now is, which may be a different one of the four. The action never disappears, and the screen is never left blank. |

## Deepgram key setup

Record 0002, AC-9 to AC-13, for milestone 3. The comp holds this screen as
the "Add your Deepgram key" artboard on Surface 3. As with sign in, the
record wins on content and the comp wins on look. The Settings rows for the
saved key ("Secret field" and "Secret field, empty" above) are the Settings
variant and stay as they are; this section is the whole-window screen.

| Component | Where it lives | Status | Notes |
|---|---|---|---|
| Key setup screen | echoscribe window | drawn | The guided setup screen of AC-9, and step two of first run. Reuses the onboarding window shell: 760x540, dark throughout (`--color-onboarding`), one centered column, no nav rail. From the comp: a 27px display heading (`--text-2xl`, `--color-ink-on-dark-heading`), one paragraph in the first-run body colour (`--color-ink-on-dark-secondary`) saying what a Deepgram key is and that audio goes from this machine to the person's own Deepgram account and nowhere else, then the key field, then the help link and a caption (`--color-ink-on-dark-caption`). Same shape as the microphone error screen: a hotkey that could not start dictation, and a window that says why (AC-9: the microphone does not open, so no pill and no sound). |
| Step indicator | key setup screen | drawn | From the comp: a mono caption "STEP 2 OF 2" (`--color-ink-on-dark-caption`, `--tracking-caps`) beside two small progress bars. Shown only on the first-run path after sign in. When the screen appears because the hotkey was pressed with no key saved (AC-9), there is no step indicator, because the person is not in a flow. The comp fills the bars with violet; see the judgement call in the patrol report. |
| Key field, dark | key setup screen | drawn | The paste field. A dark well (`--color-pill-raised`, nearest token to the comp's fill), mono placeholder in `--color-ink-on-dark-caption`, a SECRET mono badge, and the verify action sitting inside the field's right edge as in the comp. The pasted key is treated as a secret from the first keystroke: masked as typed, never logged, never echoed back in full (AC-12 and the data rules). Treat the pasted text as hostile input. |
| Key setup action | key field, dark | drawn | The "Primary button, on dark" primitive, labelled "Verify & finish" on the first-run path and "Verify" on the AC-9 path. Verifying is the only way a key is saved: a key that fails the check is never stored (AC-10, AC-11). |
| Key setup, checking | key setup action | drawn | The key being checked against Deepgram. The button takes the "Primary button, on dark, busy" primitive unchanged: same geometry, label swapped to "Checking key", light fill breathing on the 1.4s cadence, never violet, holding still under `prefers-reduced-motion` with the label still carrying the state. The field is not editable while the check runs. |
| Key setup error line | key setup screen | drawn | The same three parts in the same order as the sign-in and microphone error lines: a mono code, one plain sentence of cause, then the one action beneath. Code and left edge in `--color-danger-on-dark`, sentence in `--color-ink-on-dark-secondary`. One wording per cause, fixed at design time. Nothing read off the pasted key ever appears in the sentence. |
| Key setup, rejected | key setup error line | drawn | Deepgram refused the key (AC-11). Mono code `DEEPGRAM_KEY_REJECTED`, a sentence saying the key was not accepted and nothing was saved, and the verify action ready again with the pasted text still in the field so one typo does not cost a fresh paste. When a previously saved key stops being accepted (AC-13), the same line distinguishes the two causes Deepgram reports, key rejected and allowance ran out, each with its own sentence and matching next step. Never dressed up as a network failure. |
| Key setup, no network | key setup error line | drawn | Deepgram could not be reached, so the key is neither accepted nor rejected. Mono code `DEEPGRAM_UNREACHABLE`, a sentence saying the check could not run and nothing was saved, and the one action is retrying the check. The sentence never blames the key when the network is the cause. |
| Get-a-key link | key setup screen | drawn | The AC-9 link out to get a key, "Get a free key from Deepgram". Opens the system browser. Mapped to `--color-accent-on-dark` as the nearest token to the comp's link colour; see the gap noted in the patrol report, since no on-dark link token exists and this uses violet for something that is not an open microphone. |
| Key setup, saved | key setup screen | drawn | The success ending. On the first-run path the already registered "Ready confirmation" row takes over, naming the key, the microphone and the language back to the user. On the AC-9 path the screen simply closes, because the person was mid task and the hotkey now works (AC-10). Both endings drawn, the screen is never left blank. |

## Shared primitives

| Component | Status | Notes |
|---|---|---|
| Primary button | drawn | Dark fill, white label. |
| Secondary button | drawn | Control border, ink label. |
| Text button and link | drawn | Accent link colour. |
| Primary button, on dark | drawn | The sign-in and first-run dark windows. Light fill (`--color-ink-on-dark-heading`), dark label (`--color-onboarding`). Same shape as the light primary button. Already in the comp on the first-run window. |
| Primary button, on dark, busy | drawn | The state while the app is waiting on something before anything else can be shown. Same geometry, same dark label, so the button never resizes or moves. Two signals: the label swaps to what is being waited on, and the light fill breathes between `--color-ink-on-dark-heading` and `--color-ink-on-dark-secondary` on the same 1.4s cadence as the waiting indicator. Never violet. The label carries the state on its own, so under `prefers-reduced-motion` the fill holds at the dim rung and nothing animates. Not clickable while busy, and it keeps its focus ring. Both ends of the pulse are audited in `design/check-contrast.py`; the dim end is the tighter pair at 9.72:1. |
| Secondary button, on dark | drawn | The dark windows. Control edge (`--color-rail-border-control`), neutral label (`--color-ink-on-dark-secondary`). For cancel and retry. |
| Brand lockup | drawn | A violet dot and the "EchoScribe" wordmark. Already in the comp on the nav rail and the first-run and sign-in windows. |
| Keycap | drawn | Mono, used wherever a key is named. |
| Mono badge | drawn | Uppercase status and category tags. |
| Focus ring | drawn | Accent. On every keyboard-reachable control, no exceptions. |

## Not in the registry, on purpose

Modals, toasts, tooltips, tabs, dropdown menus, avatars beyond initials,
icon buttons without a label, illustrations, a mascot, charts, a theme
toggle. None of these are drawn, so none of them may be built. Ask first.
