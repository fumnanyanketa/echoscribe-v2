# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Everything below landed after `9262a9f`, the app shell's failing verify
evidence of 2026-09-03. Nothing below has been released, and nothing below
has passed `/check verify`.

### Known issues

Read this section before the others. It changes what the rest of them mean.

- **The application opens two different SQLite files, and three of the five
  features cannot work because of it.** `sign_in`, `dictate` and `shell`
  open `echoscribe.sqlite3`; `vocabulary`, `language` and `history` open
  `echoscribe.db`. The `account` and `dictation` tables are created only in
  the first file. So history can never be read, and no custom word or
  language choice can ever be saved. History is the screen the app lands on
  after sign-in, so this is the first thing a signed-in person meets. Found
  2026-09-11 by `/check review`, confirmed three ways, not yet fixed. See
  [docs/reviews/2026-09-11-master.md](docs/reviews/2026-09-11-master.md).
  **Every "Added" entry below for vocabulary, language and history describes
  code that exists and does not currently function.**
- Eleven further issues of Major severity are open from the same review,
  most of them timing races. None is fixed.
- No feature in this project has a passing verify run. The last verdict on
  record 0002 was Blocked and the last on record 0004 was Fail.

### Added

- **Settings screen, Dictation section.** The two hotkey choices as one
  radio group, the dictation sounds switch, and a caption under each. A
  refused write draws a shared error line; a failed read draws no control at
  all rather than a control that lies (see record 0002, AC-19, AC-21,
  AC-22).
- **Settings screen, Transcription section.** The saved Deepgram key, shown
  as a fixed run of twenty bullets and the last four characters. The bullet
  count is fixed on purpose and carries no information, because the key's
  length is never stored: a length narrows a secret. Replace opens a paste
  field in place and runs the same check as a first key, writing nothing on
  any failing path. Remove asks once on the row, with focus on Cancel (AC-12,
  AC-34, AC-35).
- **Teach it your words.** A Vocabulary surface for terms Deepgram keeps
  getting wrong, sent on every stream as `keyterm`. The budget is 400
  characters, not a word count: Deepgram accepts 500 tokens and no tokeniser
  emits more tokens than there are characters, so a character budget is
  provably inside the limit in any script where a word count would not be
  (see decision record 0005, 14 acceptance criteria).
- **Speak in your language.** One chosen language from the 64 in Deepgram's
  nova-3 table, with a filter over the list and Multilingual among them.
  English stays the default, so nothing changes for anyone who does not go
  looking (see decision record 0006, 12 acceptance criteria).
- **See what you've said before.** A History screen that reads back past
  dictations. It stores nothing: no table, no column, no migration, no
  write, which makes it the first feature here of which that is true. Paged
  by a `(started_at, id)` cursor rather than an offset, so a dictation
  finishing while someone is reading cannot make a row appear twice. Search
  is SQLite `LIKE` with the query bound as a parameter and its wildcards
  escaped (see decision record 0007, 15 acceptance criteria).
- **A character count on each history row, and a Copy action.** Copy puts
  the transcript on the clipboard and nothing else with it.
- **An elapsed time and character count chip on the dictation pill**,
  appearing at 20 seconds into the band the pill has reserved since
  2026-08-31, so its arrival resizes nothing.

### Changed

- The count of characters is computed once, in Rust, as `chars().count()`,
  and the same figure feeds both the pill's chip and the history row. Neither
  side counts in JavaScript, where `String.length` would call an emoji two
  characters and disagree with Rust about the same transcript.
- The four dictation settings commands now return a structured error
  carrying the shared error sentence, replacing the bare error strings they
  used to return. `get_deepgram_key_info` and `clear_deepgram_key` were
  moved onto the same error, so both settings sections word a refused write
  and a failed read identically, from one place.
- `clear_deepgram_key` now deletes the database row before the credential
  entry. On the old order, a credential-store failure returned an error over
  an account whose key had already gone.

### Fixed

- **Signing out left the app running with no window at all.** The app kept
  the global hotkey with nothing on screen and no way back in. Asking Windows
  to destroy a window is a request carried out on a later turn of the event
  loop, not the destroy itself, so the code ordered the dashboard destroyed
  and then asked whether a dashboard existed. The answer was still yes. The
  branch that orders the close now says so, rather than leaving the next step
  to ask.
- **Closing the small window with its X, while no dashboard existed, left
  the process alive with nothing on screen.** The same end state as the fix
  above, reached by a different door. The underlying cause is that this app
  can never end by itself: the pill window is created once at startup and is
  only ever hidden, never destroyed, so Tauri's own "the last window has
  gone" exit can never fire here. Every path that takes the last visible
  window away must now end the app deliberately.

### Security

- No code changed. A full sweep on 2026-09-11 found no secret in the working
  tree or in any of the 56 commits: all 56 forty-character hexadecimal
  strings in the history are git object SHAs, and no `.env` was ever
  committed. Four medium findings are open, including no Content Security
  Policy and a Clerk development instance still in `config.rs`. See
  [docs/reviews/security-2026-09-11.md](docs/reviews/security-2026-09-11.md).
- **Not a code issue, but it gates publishing:** twelve screenshots in
  `docs/evidence/the-app-shell/` carry a real name, a personal email address,
  a desktop wallpaper and a software inventory, and they are in git history.
  Deleting the files would not remove them. This must be settled before the
  repository is ever pushed.
