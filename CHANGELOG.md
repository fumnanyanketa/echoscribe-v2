# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Every entry names the commit it comes from. Rewritten 2026-10-19 to say what
is in the installer people can download and what has landed since; the
previous text described the September state and had fallen a month behind.

## [Unreleased]

Everything below is on `main` and in no installer yet. It becomes v0.1.1.

### Added

- **A local diagnostic log** (`071cf21`, record 0008). The app's diagnostic
  lines, which a release build used to drop into a hidden console, now land
  in `echoscribe.log` beside the database, each with the UTC time, capped at
  1 MB with one previous copy. A crash writes its own last line. Nothing
  about errors or usage leaves the machine, now written down as a rule
  rather than an omission, and a guard across every Rust file refuses a
  diagnostic line that formats a transcript, key, token or code. Proved on
  a dev build; not yet seen on an installed build. The privacy policy
  sentence that names the file awaits the user's approval.

### Fixed

- **The key setup screen stuck on "Checking key" after a key was accepted**
  (`0eb4841`). A synchronous Tauri command runs on the main thread, not a
  worker thread as its comment claimed; it emitted the saved-key signal
  from there, the shell built the dashboard window inside the command, and
  that is a documented deadlock on Windows. The key saved, the dashboard was
  built hidden, and the screen never moved. Every first key save in v0.1.0
  did this. The signal now comes from its own thread. Re-proved live by the
  user on 2026-10-19.
- **The sign-in wordmark drew at 23 px instead of 28** (`6575bd4`). A later
  rule in `sign-in.css` beat the wordmark's own on an element carrying both
  classes. Found by the 2026-10-06 verify, which returned FAIL on that one
  line; fixed and guarded.
- A stale comment named a deleted file, `account-card.js` (`6575bd4`).

### Changed

- **The key setup screen is calm** (`ae230df`): one sentence, no stepper,
  the three steps behind a "Show the three steps" toggle, a quiet Verify, a
  fainter watermark. Guarded by `the_key_setup_screen_stays_calm`.
- Two diagnostic lines reworded so the new log guard passes; neither
  formatted a secret (`071cf21`).

## [0.1.0] - 2026-10-05

The first release. Tagged on `dd2bb96`, published as a GitHub release with
one asset, `EchoScribe-Setup.exe`, and handed out through an email gate on
`https://echoscribe.exentrik.co`. In use by its maker since the afternoon of
release; offered to nobody else yet. What blocks a wider circle is in
[docs/releases/001-readiness.md](docs/releases/001-readiness.md).

### Known at release

- **Every first key save hangs on "Checking key."** Not known on release
  day; found on 2026-10-13 and fixed above. The app recovers on its next
  launch, because the key did save.
- **The installer is not code signed**, by the user's choice on 2026-10-02.
  Windows shows the SmartScreen warning to everyone.
- **Nothing about crashes or usage is reported anywhere**, by design
  (record 0008). A problem on another machine reaches the project only as
  an email to hello@.
- **Not proved on the production build:** no internet at launch, slow
  internet during a dictation, two monitors, the hotkey pressed twice
  quickly. Listed in the readiness record and the sign-off draft.

### Added

- **The whole dictation tool, end to end**, proved live on 2026-10-01
  ([report](docs/evidence/end-to-end-2026-10-01/report.md)): sign in
  through the browser, press the hotkey anywhere, speak in English or
  German, the words type at the cursor, custom words are recognised, the
  language choice holds, history reads back, and all of it survives a
  restart.
- **Sign in** against Clerk's production instance on
  `echoscribe.exentrik.co` (`930d1f0`), with Google and email-code sign-in.
  Sign in always asks who is signing in, even when the browser remembers
  you, and shows no extra consent page (`dd2bb96`, record 0003 AC-19 to
  AC-21).
- **Dictate with a hotkey**: double-tap Ctrl or Alt, the pill with its live
  waveform and the elapsed-and-count chip, typing as character keystrokes
  with the password-field refusal, the two sounds, the 30 second silence
  and 5 minute caps, plain-worded errors for a blocked microphone, a
  refused key and a lost connection. Record 0002, 36 criteria.
- **Settings**: Dictation (the two hotkey choices, the sounds switch) and
  Transcription (the masked Deepgram key, Replace in place, Remove with one
  question on the row).
- **Teach it your words**: a Vocabulary surface, terms sent on every stream
  as `keyterm`, budgeted at 400 characters so it is inside Deepgram's 500
  token limit in any script. Record 0005.
- **Speak in your language**: one chosen language from the 64 in Deepgram's
  nova-3 table plus Multilingual, English the default. Record 0006.
- **See what you have said before**: History, the landing screen, with
  search, a character count and a Copy action per row. Stores nothing of
  its own. Record 0007.
- **The app shell**: a 1200x800 dashboard with the rail, remembering its
  size and place per account, beside the small 760x540 window for sign-in,
  key setup and interruptions. Record 0004. The account row opens a menu
  with the email, the signed-in date and Sign out (`1aefb9a`); the Settings
  sub-nav folds (`43cfcfc`).
- **The look**: matched to the Instrument comp pixel for pixel (`2b791bc`),
  then re-locked as Night, all dark with a teal accent (`fbe8da1`). The
  first-run screens redrawn (`1aefb9a`) with the rail's watermark
  (`6790599`). An app icon (`1e9b5b5`).
- **The public page and privacy policy** at `https://echoscribe.exentrik.co`,
  deployed from `site/` by GitHub Pages (`c6beaac`, `ba103b7`), with the
  download behind an email address (`dba1fd3`) and contact at hello@.

### Fixed

- **The app opened two SQLite files** (`76a5c3d`, guarded in `6461889`).
  Three features said `echoscribe.db` and three said `echoscribe.sqlite3`,
  so history could not be read and neither a custom word nor a language
  choice could be saved. Found by a review on a second model on
  2026-09-11 after 342 passing tests. Now standing rule 19.
- **Two threads could build two dashboards** (`c77ad13`), seen once as a
  ghost window during the 2026-10-01 verify.
- **The pill's transcript line never slid left** (`f063c9b`); it had not
  worked since it was built.
- **Signing out, or closing the small window with its X, left the app
  running with no window** (`5a4b13c`, `10ecae7`, guarded in `157ecf2`).
  The pill window is never destroyed, so Tauri's last-window exit can never
  fire here; every path that takes the last visible window away now ends
  the app itself. Standing rule 18.

### Security

- A real Content Security Policy and no broad capability grant (`3cd8cc7`),
  closing the two findings left open by the September sweep.
- The TLS library patched (`1e9b5b5`).
- Sweeps on 2026-09-11 (`beba190`) and 2026-10-05 (`bee739b`): no Critical
  or High finding open. Personal data purged from git history on 2026-09-24
  before the repository went public.

### Reviewed

- The whole codebase reviewed on a second model on 2026-09-11 (`ea49d6c`),
  which found the two-database blocker above and eleven Major findings,
  most of them timing races. The blocker is fixed; the review's Major list
  has not been walked line by line since, and
  [docs/reviews/2026-09-11-master.md](docs/reviews/2026-09-11-master.md) is
  the record of what is still open from it.
