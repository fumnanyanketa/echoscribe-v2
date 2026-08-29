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
| Error pill | replaces pill body | drawn | Same geometry. Mono code, one sentence, exactly one action. |

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
| Sign-in error line | sign-in window | drawn | The same information order as an error pill: a mono code, one plain sentence of cause, exactly one action. Code and left edge in `--color-danger-on-dark`, sentence in the first-run body colour. One wording per named reason in record 0003: browser closed, timed out, security check failed, Clerk rejected sign-in, could not reach Clerk. |
| Sign-in notice line | sign-in window | drawn | Why the person returned to sign in. Neutral, in the first-run caption colour, for "you signed out". Attention, in `--color-warning-on-dark`, for "your session was ended elsewhere". A marker and words, never colour alone. |

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
