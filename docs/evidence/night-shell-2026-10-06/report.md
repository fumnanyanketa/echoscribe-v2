# Verify: the Night shell, the account menu, the sub-nav fold and the redrawn sign-in screen, 2026-10-06

Scope, as asked: the interface work committed between 2026-10-02 and 2026-10-05
(`e765451`, `43cfcfc`, `fbe8da1`, `1aefb9a`, `6790599`, `dd2bb96`). Those commits
carry no new acceptance criteria of their own. What they promise is written in
`design/registry.md` (rows `Account block`, `Account card`, `Section sub-nav`,
`Sign-in window`, `Key setup screen`), in `design/design-system.md` ("Night"), and in
three criteria added to record 0003 (AC-19 to AC-21). Record 0004's AC-2 and AC-7
are exercised because the same surfaces carry them. Memory called the work medium
weight; record 0003's rows are heavy, so its three criteria need the user's own
written sign-off and are not claimed here.

Approach bar: tracer bullet, from AGENTS.md. Everything here is a real screen in
the real installed app; nothing is allowed to be stubbed.

**Verdict: FAIL.** One drawn promise is observed not to match (the sign-in
wordmark size), and five behaviours are blocked, four of them because Claude in
Chrome disconnected at the moment the browser hand-off began. Every other
promise was observed and matches.

## How this was run

- `npm run tauri build` from `ba103b7` (HEAD, clean tree) at 11:45, installed
  silently with `/S` over the previous install, so the binary under test is the
  committed code. The installed `echoscribe.exe` is dated 11:45:44.
- The installed app was launched with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=
  --remote-debugging-port=9333`. Every click below is a real input event through
  WebView2's input pipeline (`Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`),
  not a handler called from script. Script was used only to **read** the page
  (attributes, computed styles, text) and to take `Page.captureScreenshot` frames.
- Windows was asked what windows exist (title, visibility, rect, foreground) with
  a per-monitor-DPI-aware probe. Display scaling is 200%, so 1200x800 logical
  shows as a 2400x1600 client and the 760x540 small window as 1520x1080.
- The database at `%APPDATA%\com.echoscribe.app\echoscribe.sqlite3` was read
  directly for the sign-out claim.
- Frames that show the account's name or address live in `local-only/`, which
  this folder's `.gitignore` keeps off the public repository.
- No test run is scored. `cargo test` is `/test`'s job and a green test is not an
  observation.

## What was promised, and what was observed

| Promise | Outcome | Evidence |
|---|---|---|
| Night: page `#0e0e0f`, cards `#19191b`, accent `#5eead4`, no white reading surface anywhere | met | computed tokens read from the dashboard: `accent #5eead4`, `page #0e0e0f`, `surface #19191b`, body and rail `rgb(14,14,15)`, zero elements with a white background; `local-only/01-dashboard-at-rest.png` |
| Rail: History at the top with its count, Settings and the account at the foot, active item gets the teal icon on a raised pill | met | `local-only/01-dashboard-at-rest.png`, `local-only/05-settings-folded-screen-unchanged.png` |
| `Account block` is a button; pressing the disc and name opens a menu **above** the row holding the mail address, the signed-in line and Sign out | met | after one click on `.account__identity`: `aria-expanded=true`, menu `display:flex`, menu bottom above the row's top, items `account__email`, `account__since` ("Signed in since October 5, 2026"), `account__signout` (role `menuitem`); `local-only/02-account-menu-open.png` |
| The chevron turns when the menu is open; `aria-expanded` carries the state | met | at rest `aria-expanded=false`, `aria-haspopup=menu`, `aria-controls=account-menu`; open: chevron `transform: matrix(-1,0,0,-1,0,0)` (180 degrees) |
| Escape closes the menu and returns focus to the row | met | after `Escape`: `menuHidden=true`, `aria-expanded=false`, `document.activeElement` is the row |
| A click elsewhere closes the menu | met | reopened, then one click on the reading surface: `menuHidden=true`, `aria-expanded=false`; `local-only/03-account-menu-closed-again.png` |
| `Account card` retired: no ACCOUNT card and no Sign out on the Transcription surface; its opening card reads "Your Deepgram key." | met | on Transcription: `accountCard=false`, no element reading `ACCOUNT`, no Sign out button on the surface, first card text "Transcription / Your Deepgram key."; `local-only/06-transcription-no-account-card.png` |
| The only Sign out in the app is the menu's, hidden until the row is pressed | met | one `Sign out` button in the document, class `account__signout`, not visible at rest |
| `Section sub-nav` is hidden while History shows; Settings carries `aria-expanded=false` | met | at rest: sub `hidden=true`, `display:none`, all four sub items inside it; Settings `aria-expanded=false` |
| Pressing Settings from outside opens Dictation and shows the sub-nav (record 0004 AC-2 unchanged) | met | after the press: heading "Dictation", sub `display:flex`, active `settings.dictation`, Settings `within` and `aria-expanded=true`; `local-only/04-settings-open-subnav-unfolded.png` |
| A second press folds the sub-nav **without changing the screen** | met | heading still "Dictation", active still `settings.dictation`, sub `hidden=true`, `aria-expanded=false`; `local-only/05-settings-folded-screen-unchanged.png` |
| A third press unfolds it | met | sub `hidden=false`, `aria-expanded=true`, heading unchanged |
| Leaving the section hides the sub-nav; entering afresh arrives unfolded | met | History: sub hidden, Settings `aria-expanded=false`; Settings again: sub shown, heading "Dictation" |
| Record 0004 AC-2: every rail item opens a screen, none does nothing, none greyed | met | six destinations, each clicked, each heading observed: History, Dictation, Languages, Vocabulary, Transcription; no `disabled` item; `local-only/07-vocabulary-night.png` |
| Record 0004 AC-7, first half: Sign out closes the dashboard and leaves the sign-in screen | met | window probe after Sign out: the `EchoScribe` dashboard window is gone, `echoscribe` (1520x1080) visible and foreground, the pill hidden as always; the dashboard page is gone from the debugger's page list; `session` row 1 emptied (`account_id` null) |
| Record 0004 AC-7, second half: a second account gets its own window place | blocked | needs a second account's sign-in; see AC-19 below |
| `Sign-in window`: dark throughout, the watermark twice (twelve bars, 0.28, entering from each edge), everything centred, one teal Sign in button with a 180px floor and a soft teal shadow, near-black label, the one-sentence caption, no "Sign in to", no password sentence, no settings paragraph | met, except the wordmark size below | body `rgb(14,14,15)`; two `signin__watermark` rows of 12 bars at opacity 0.28, one off the left edge (`left -17`) and one mirrored off the right (`right 777` in a 760px window); button 180x42, `rgb(94,234,212)` on `rgb(14,14,15)` with `rgba(94,234,212,0.45) 0 8px 24px -8px`; caption "Clicking the button opens your browser to sign in."; `/password/`, `/Sign in to/`, `/settings/` all absent; `08-sign-in-screen-redrawn.png` |
| `Sign-in window`: the wordmark "at 28px bold" | **not met** | the name renders at `23px`, weight `500`. The span carries both `signin__heading` and `signin__wordmark`; `src/sign-in/sign-in.css` sets the wordmark to 28px bold at line 111 and the heading to `--text-xl` medium at line 148, and the later rule wins. The stylesheet's own comment says 28px was the intent |
| Sign-in waiting state after pressing Sign in | met | small window text: "Signing in / Waiting for your browser / Cancel"; the app is listening on `127.0.0.1:61138` |
| Record 0003 AC-19: pressing Sign in shows Clerk's page asking who I am even with the browser already signed in | blocked, with one half observed | the first hand-off opened in the user's Chrome while Claude in Chrome was disconnected, and timed out: the small window then read "Sign-in did not finish / TIMED_OUT / Sign-in timed out waiting for the browser. / Try again", the registered error line in its place. On "Try again" the browser tab Chrome opened was titled "My account \| Echoscribe" and its address, copied from Chrome's own address bar, was `https://accounts.echoscribe.exentrik.co/sign-in?redirect_url=https://clerk.echoscribe.exentrik.co/oauth/authorize/continue?...&prompt=login&redirect_uri=http://127.0.0.1:55201/callback&...`. So the app sent `prompt=login` and Clerk answered with its sign-in page rather than a consent page. What this run could not establish is the criterion's condition, that the browser was "still signed in to Clerk" at that moment, and it could not drive the page: the auto-mode classifier refused to open the copied address in a tab the extension can drive. Not judged from the code |
| Record 0003 AC-20: after sign-out nobody gets in by pressing Sign in | blocked | the local half was observed (session emptied, the app on the sign-in screen); the browser half is AC-19's and is blocked with it |
| Record 0003 AC-21: no "allow access" page | blocked | the sign-in was completed in the browser at about 13:14, by the user, as exentrik.co@gmail.com, after a first callback at 55201 was refused because the five minute window (`SIGN_IN_TIMEOUT_SECS = 300`) had passed. The app received the second callback and moved to the signed-in state on its own (record 0003 AC-3's shape: session row `user_3KJh...` established 10:15Z, the small window on the next screen). Whether a consent page appeared on the way was not observed by this run; it is the user's observation to add |
| `Key setup screen` redraw: three numbered steps, "click Verify", no caption, the watermark | met | reached the honest way at 13:15: the user completed the sign-in as a third account (exentrik.co@gmail.com, never used on this machine, no key saved), and the small window showed the key setup screen. Observed: "Step 2 of 2 / Connect your Deepgram key", the paragraph ("free to use because you get to bring your own Deepgram key ... $200 of free credit, no card needed ... three steps, which is about a minute"), an ordered list of three steps ending "Paste it here and click Verify", buttons "Verify" and "Open Deepgram sign-up", no caption element, two `keysetup__watermark` rows of 12 bars at opacity 0.28, heading centred at 23px/600; `09-key-setup-screen-second-account.png`. The database gained `account` row `user_3KJh...` with no `deepgram_credential` and no `shell_window` row, which is why the dashboard did not open: record 0004's invariant, no key means the key setup screen |

Surfaces specced by the rows above: the dashboard rail, the account menu, the
five section screens, the sign-in screen, the waiting state, the key setup
screen. Present: all but the last, which exists in the bundle and was not
reached. Missing: none known.

## Found outside the promises

- **The wordmark finding above is a cascade bug, not a design decision.** One
  reorder or one more specific selector fixes it. Route: `/develop`, then
  `/canvas drift` should re-measure both first-run screens, because the key
  setup screen reuses the same heading classes.
- **A comment names a deleted file.** `src/dictate/transcription-settings.js`
  line 461 says its `formatDate` is "a second copy of `src/shell/account-card.js`'s
  formatter". That file was deleted in `1aefb9a`; the surviving copy is in
  `src/shell/account-block.js`. Wrong comments are how the "two copies held
  together" rule drifts. Route: `/develop`.
- **The installed release build honours `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`.**
  That is how this run drove it, and it means anyone who can set an environment
  variable and launch the app can read its pages. Nothing secret is in those
  pages by the data rules (the key shows as its last four, the session never
  crosses into the interface), and this run confirmed the Transcription surface
  renders only `·caf3`. Worth one line in the next `/warden` pass so it is a
  known property rather than a surprise.
- The sign-in screen showed "You signed out." above the wordmark after the
  sign-out. That is the registered `Sign-in notice line`, in its place.
- Rig note: Git Bash turns the installer's `/S` into a path and the installer
  opens its window instead; run it from PowerShell.

## Not checked, and why

- AC-19's "still signed in" condition, AC-21's consent page, and AC-7's second
  half: the browser part of each was done by the user, not observed by this
  run. AC-7's second half now has its precondition in place (a third account
  with no `shell_window` row) and will be observable the moment that account
  saves a key and its dashboard opens. **The app was left signed in as
  exentrik.co@gmail.com on the key setup screen**: dictation is off for that
  account until a key is saved, and the user's own account (with its key) is
  one sign-out and sign-in away.
- The waiting state times out on its own after five minutes (observed twice
  this run), and a callback arriving after that is refused by the closed
  listener, which Chrome shows as "127.0.0.1 refused to connect". Correct
  behaviour, and worth a sentence on the timed-out screen one day: "the
  browser came back too late" reads better than a refused connection.
- Rig note for next time: a link the app opens lands in Chrome outside the
  extension's tab group. Its address can be read by fronting the Chrome window
  and copying the address bar (`Ctrl+L`, `Ctrl+C`), but opening that address
  in a driveable tab needs the session out of auto mode.
- The key setup screen, for the reason above.
- The dashboard's remembered size and place (AC-3 to AC-5) and the error window
  (AC-6): outside this run's scope and unchanged by these commits.
- Dictation itself: unchanged by these commits; last proved 2026-10-05 by the
  user's own production dictation, visible in the History row.

No plan verify box is ticked by this run.
