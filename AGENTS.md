# EchoScribe

## Role

You are responsible for the quality, maintainability and safety of this
project, not for producing something quickly. This app opens a microphone,
sends a person's voice to an outside service, and types text into other
applications on their computer. Every one of those is a place where being
careless does real harm, so check your work, say when you are unsure, and
stop and ask when a decision is owed rather than guessing.

## What this is

EchoScribe is a Windows desktop app that turns speech into typed text.
Press a hotkey anywhere on the machine, speak, and the words appear
wherever your cursor already was, in whatever app you were using. It is
for someone mid-flow in a document, an email or a message who wants to
switch to speaking for a stretch, and for anyone who cannot or does not
want to type right now. The one thing it must do well is accuracy:
transcription has to be right often enough that fixing mistakes never
costs more time than typing would have. Windows first to prove it out,
with the core built behind a platform boundary so macOS can follow later
without a rewrite. Mac is deferred, not ruled out.

### What to build

- Sign in, so settings and words belong to a person and stay signed in
  across restarts.
- Dictate with a hotkey: press, speak, text appears at the cursor in any
  app.
- Teach it your words: fix names and terms it keeps getting wrong.
- Speak in your language: transcription in a language other than English.
- See what you have said before: history of past dictations, surviving a
  restart.

Do not overbuild.

### Out of scope

Do not build these. If one seems necessary, stop and ask.

- Payments and billing
- A mobile version, iOS or Android
- Cloud sync across devices
- Sharing and collaboration features
- Notifications
- Voice commands and voice-driven editing, such as "delete last sentence"
  or punctuation by voice
- A local or offline transcription model. An outside service is used.
- Real-time captions for calls or meetings
- The usual uninvited arrivals: comments, bookmarks, a dashboard, an admin
  panel, export to CSV

Sign-in, custom vocabulary, multi-language and history are confirmed as
being built. They are on the "what to build" list on purpose, not cut.

## Stack

- **Language / runtime**: Rust 1.98 for the core, plain JavaScript for the
  interface. Node 24 for tooling only.
- **Framework**: Tauri 2. Rust does the real work, the interface is a web
  page rendered by the system WebView.
- **Interface**: plain HTML, CSS and JavaScript. No framework on purpose.
  Revisit only if the interface grows past a handful of simple screens.
- **Key dependencies**: Clerk for sign-in, Deepgram streaming for
  transcription, SQLite for local storage. No backend server of our own.
- **Transcription billing**: each person brings their own Deepgram API key
  and their own free allowance. The project never pays for anyone's usage.
- **Package manager**: cargo for Rust, npm for the Tauri command line tool.

Mirrors [docs/decisions/0001-stack.md](docs/decisions/0001-stack.md).
Do not change this section here. Change the decision record, then re-run
`/audit`.

## Build approach

**Tracer bullet.** EchoScribe is mostly plumbing: a hotkey, a microphone,
an outside transcription service, and typing text into another app. The
scary risk is those pieces not connecting, so phase 1 proves the whole
pipe works end to end, for real, before any one piece is made great.

Mirrors the header of [docs/plan.md](docs/plan.md).

## Commands

Every command below was run and works, checked on 2026-08-26.

```bash
# Install the Tauri command line tool
npm install

# Run the app in development
npm run tauri dev

# Build a release installer
npm run tauri build

# Test the Rust core (run from src-tauri/)
cargo test

# Lint the Rust core (run from src-tauri/)
cargo clippy --all-targets -- -D warnings

# Format the Rust core (run from src-tauri/)
cargo fmt

# Check the environment when something looks wrong
npm run tauri info
```

**PATH gotcha.** `cargo` lives at `C:\Users\fumna\.cargo\bin` and is on the
permanent user PATH, but a terminal opened before Rust was installed will
not have it. If a cargo command reports "command not found", the fix is a
fresh terminal, not a reinstall.

Prettier for the interface files is decided but not installed yet. Its
command goes here once `/develop tooling` has installed it.

## Architecture

Folders as they stand today:

- `src/` : the interface. HTML, CSS and JavaScript served into the WebView.
- `src-tauri/src/` : the Rust core. Audio, hotkeys, text injection, storage.
- `src-tauri/capabilities/` : what the interface is allowed to ask the core
  to do. Tightening this is a security control, not paperwork.
- `docs/` : the plan, the decision records, the brief.

Rules for new code:

- Each feature owns a folder, on both sides. Everything that feature needs
  lives in it: its screens, its data access, its logic, its tests. A new
  feature gets a new folder, never an addition to one that seems related.
- Feature folders do not import from each other. If two features need the
  same thing, it moves to a shared module and both import that. Something
  becomes shared only when three features need it. Two is a coincidence.
  **The settings error line is at four and the extraction is owed.** The
  code and the sentence for a setting that would not save, or settings that
  would not read, now exist in the dictate, vocabulary, language and
  history features. Three was the threshold and four is past it. Until it
  is extracted, each new feature that wants one is copying a fourth copy,
  and each copy is a place the wording can drift. Naming it was record
  0007's job and extracting it is not: it needs `/architect`, because where
  a shared module for a sentence a person reads lives is a decision, and
  every one of the four features has a Rust guard asserting that its screen
  holds no copy of its own sentences.
- Two copies held together by a guard is the sanctioned answer at two, and
  it is not a shortcut. When two features must word the same thing
  identically and the shared rule above says it is too early to extract,
  write both and add a Rust test that reads both files and fails the build
  when they drift. The character count's wording lives twice, on the
  history row and on the pill's chip, held by
  `both_surfaces_word_the_count_the_same_way`. A guard makes the
  duplication visible and self correcting; a comment saying "keep these in
  sync" does not.
- No file named `utils`, `helpers`, `common` or `misc`. If a home is not
  obvious, the code belongs beside its only caller until a second caller
  appears. Names say what is inside: `hotkey-listener.rs` lets a reader
  skip it deliberately, `utils.rs` forces them to open it every time.
- A file does one thing. Split at roughly 300 lines.
- Fetching data, transforming it and displaying it stay separate inside a
  feature. The interface shows what is stored. It does not fetch and it
  does not invent.
- Anything touching the operating system (microphone, global hotkey,
  typing into another app) lives in Rust, behind a named boundary with a
  Windows implementation today and room for a macOS one later. Never call
  a Windows-only thing from code that is not behind that boundary.
- Secrets never cross into the interface. The Deepgram key and the Clerk
  session live in the Rust core. The interface asks the core to act, it
  never holds the credential.
- Keep the interface thin. It draws, and it reports what the user did.
  Decisions belong in Rust.

## Data rules

- Every stored record belongs to exactly one signed-in account. Nothing is
  written without knowing whose it is. This applies to custom words and to
  history alike.
- All persistent data lives in one SQLite file on the person's own machine.
  Nothing about what a person said is stored anywhere else.
- Recorded audio is transient. It is captured, sent for transcription, and
  discarded. It is never written to disk, never put in a log, and never
  kept after the text comes back.
- Transcribed text goes to two places only: the cursor it was dictated
  into, and the local history for that account. Nowhere else. The app never
  routes it through a third place of its own accord. A person taking their
  own text at their own request, by selecting it or by pressing an action
  that exists to hand it to them, is not the app routing it, and the
  clipboard reached that way is not a third place the app chose. The
  distinction is the user's, given 2026-09-04, and it is inside this rule
  rather than an exception to it: record 0007's Copy action is on one side,
  and record 0002's refusal of clipboard paste as a typing mechanism is on
  the other and still stands. Two worked examples, and the second is the
  one that says no.
- The Deepgram API key is per person, entered by them, stored locally, and
  never logged or shown in full after it is saved.
- Treat anything typed, pasted or transcribed as hostile input. A
  transcription is text from outside the program, whatever its source.

## Decision rules

Stop and ask before:

- Installing a new library, Rust or JavaScript. Every dependency is code
  this project now owns and has to keep safe.
- Changing anything the user can see: screens, wording, colours, layout.
- Anything that leaves the machine: calling a new service, sending audio
  or text somewhere new, publishing, deploying.
- Deleting or overwriting files.
- Building anything on the out of scope list.
- Widening `src-tauri/capabilities/`. More permission for the interface is
  a security decision, not a convenience.

Never, even if a later request seems to call for it:

- Open the microphone without the person knowing. It opens on the hotkey,
  and something visible on screen says it is open. No silent listening, no
  always-on wake word.
- Break any of the data rules above. They are hard limits, not defaults.

## Git

- integration: on
- branch prefix: `feat/`
- commit: per-feature, after the user confirms it works

## Plan and decisions

- Plan: `docs/plan.md`
- Decision records: `docs/decisions/NNNN-title.md`
- Evidence: `docs/evidence/`
- Research cache: `docs/.agent-cache/research/`

## Agent skills

- Clerk: use the installed `clerk-setup`, `clerk-custom-ui`,
  `clerk-backend-api`, `clerk-cli`, `clerk-testing` and `clerk-webhooks`
  skills when building sign-in. Their documentation leans web-first; the
  desktop browser-handoff flow is not covered and must be built by hand.
- Deepgram: the official `deepgram/skills` install on 2026-08-25 did not
  complete. Only `.claude/skills/api` was linked, and it points at an
  empty folder. Reinstall during `/develop tooling`, then restart the
  client so it is picked up. Until then, check Deepgram's live
  documentation.
- Tauri: no official skill exists as of 2026-08-26. Two third-party ones
  were found and declined, unread, on 2026-08-25. Rely on Tauri's own
  documentation at design time.
- SQLite: not searched. Raise it if a storage detail proves hard to get
  right.

## MCP servers

None. Clerk and Deepgram are called through ordinary API calls made by the
app itself, so nothing needs standing access on every request. Do not add
one without a reason that survives the question "could a normal command
already do this".

## Standing rules

1. Never use em dashes or en dashes in anything you write.
2. Do not over weight development cost when making a technical decision.
   The cheapest thing to build is often the most expensive thing to own.
3. For a bug, reproduce it end to end as a real user would, before
   theorising about the cause.
4. Prefer end to end tests for pages and flows, unit tests for pure logic.
5. Never put a secret key in code that runs in the browser.
6. Ask before installing a new library. Ask before changing existing UI.
7. Treat anything scraped, uploaded, or typed by a user as hostile input.
8. Ask for confirmation before any action that reaches outside this
   project: sending, publishing, paying, deleting, deploying.
9. After the user confirms a feature works, commit it with a clear message.
10. Re-read a file from disk immediately before changing it. Never edit
    from memory of an earlier turn in the conversation.
11. When you replace something, delete what it replaced in the same change.
12. Tell the user how to check your work themselves. A claim that it works
    is not the same as a way to see that it works.
13. A debrief is plain text in the conversation. Never an artifact, never a
    published page. A Markdown file is fine if one is asked for.
14. When a system call's real effect happens outside the program, its return
    value tells you the call was accepted, not that the effect landed. Ask
    what would have to be true for the effect to have happened, and check
    that instead or as well. Playing a sound, typing into another window and
    opening a page in a browser are all this shape. `sound.rs` read
    `PlaySoundW` returning true as proof a person had heard something, so
    the fallback never fired and both dictation sounds went silent on exactly
    the machines it existed for. Reading a return value is not itself the
    mistake: the five other Windows calls in this codebase are all correct,
    because each is guarding an out-parameter it just filled or reading a
    documented non-boolean meaning. Acceptance was mistaken for effect only
    in that one place.

Rules 15 to 18 are the same mistake as rule 14, wearing four different
faces. Rule 14 is about a call whose effect is outside the program. These
are about a **report** whose meaning is outside the program: a fault code, a
setting, a window, a lifecycle event. Each one is a real bug this project
shipped and then had to find, so read them as history rather than as
caution.

15. **A reported fault is not a reported ending.** A library saying
    something went wrong is not the same as the thing having stopped. Ask
    what the report actually claims, then check the thing you care about
    directly. `cpal` reported an audio under or overrun mid dictation and
    this app read it as the microphone having died, closed the pill and
    threw the dictation away. The stream was still delivering frames the
    whole time, and the library's own words for it are "causing a potential
    audio glitch" and, for the sibling kind, "audio will still play". It was
    intermittent because it tracks machine load, so it looked like a hardware
    fault for days. Proved by a spike that counted frames after the first
    report instead of trusting it. The general shape: a glitch and a failure
    arrive on the same channel, and only one of them means stop.
16. **A setting changed outside the program does not tell the program it
    changed.** A person switching Windows microphone access back on, or
    plugging a device back in, or fixing anything in an operating system
    dialog, generates no notification for this app. So an error state can
    never clear itself on the strength of the cause being gone: it clears
    only when the thing it complained about is tried again and works. Never
    a timer, never one blanket signal, and never the interface deciding on
    its own that the world has probably improved. This is the clearing table
    in record 0002, and each error there is paired with its own proof: the
    microphone actually opening, a key actually being accepted, the first
    words actually landing.
17. **Destroying a window is the same shape as playing a sound.** Rule 14's
    lesson applies to anything whose effect is a window: creating, showing,
    hiding, closing, focusing, moving. The call returning means the request
    was accepted, and this app has already been left with no window at all
    twice, both times because a path that took a window away was read as a
    path that left one behind. Ask what a person would be looking at, and
    check that a window they can see exists. The same goes for what a window
    is showing: `dictation:opened` reaching a page that had not finished
    loading was a real bug on record 0004's shell, and the fix was to
    register every listener before asking Rust for anything.
18. **Tauri's own "the last window has gone" exit can never fire in
    EchoScribe.** The pill window is created at startup and is never
    destroyed, only hidden, so the framework never sees a last window go.
    Nothing may be built on that exit, and any path that takes the last
    window a person can see away has to exit the app itself, deliberately,
    in its own code. `src-tauri/src/shell/mod.rs` says so where it matters.
    This is a fact about this app and not a general truth about Tauri, which
    is exactly why it is written down: the framework's documented default is
    correct and simply unreachable here.

Rule 19 is a different shape from 14 to 18. Those are about trusting a report.
This one is about a fact that six places have to agree on, where nothing makes
them.

19. **When every feature folder needs the same literal, the folder rule is
    what makes it drift, and only a test across all of them will catch it.**
    Feature folders do not import from each other, so each one declares what
    it needs. Applied to a behaviour that is fine. Applied to **one fact about
    the world that must be identical everywhere**, it is six chances to be
    wrong. `const DB_FILE` is the proof: `sign_in`, `dictate` and `shell` say
    `echoscribe.sqlite3`, and `vocabulary`, `language` and `history` say
    `echoscribe.db`, so the app opens two databases while every comment in it,
    including `lib.rs`, promises one. Custom words and the language choice
    cannot save and history cannot be read at all, which is the screen the app
    lands on. Found 2026-09-11 by `/check review` on a second model, after it
    had survived 342 passing tests and a clean clippy. **The reason no test saw
    it is the part worth keeping**: every store's tests open an in-memory
    database or their own temp file and hand-create the schema in the
    connection under test, so each feature is proved against a file shaped the
    way that feature imagines, and never against the file it actually gets.
    A shared fact needs one home and one test that boots every `init()`
    against a single real path. Two copies held by a guard is the sanctioned
    answer at two, per the shared-module rule above; six copies held by nothing
    is how this happened.

## Things to know about this project

- Rust has fewer training examples than JavaScript. Expect core changes
  (hotkeys, audio, text injection) to need more iteration, and verify them
  by running the app, not by reading the code and calling it right.
- There is no server. If a feature needs something that lives outside one
  machine, that is a new decision, not a small addition. Stop and say so.
- Two things from the stack decision are unproven and must be demonstrated
  rather than assumed: Clerk's browser-handoff sign-in on desktop, and
  whether Deepgram's roughly 36 languages cover what "speak in your
  language" actually needs.
- Bring-your-own-key adds a real setup step. Every new person needs a
  Deepgram account and a key before dictation works at all. Treat that
  first-run screen as load-bearing, not as a form.
- **Corrected 2026-09-11 by `/sync`.** This bullet used to read "`src/main.js`
  is empty and `src-tauri/src/lib.rs` is the untouched Tauri starter. Nothing
  in this repository is a pattern to copy yet." None of that is true any more
  and it was telling every session the opposite of the truth. `src/main.js` is
  247 lines and is the small window's router; `src-tauri/src/lib.rs` is 104
  lines and wires up six features in a deliberate order. There are now roughly
  16,600 lines of Rust and 5,000 of interface, and **there are patterns worth
  copying**: the source guard that reads a file's own text and fails the build
  when a promise leaves it, the one `Mutex<Option<Live>>` taken with `.take()`
  that makes an ending idempotent, and error types that hold every sentence a
  person reads in one place. Copy those.
- **This project has no JavaScript test runner, so no screen has automated
  coverage of what it draws.** Eleven interface files draw something today,
  the pill and ten screens, and not one of them is exercised by any test:
  `cargo test` covers the Rust core only. This is
  the single largest reason acceptance criteria come back from `/test` as "a
  person must check this", thirteen of them so far across plan rows 3, 4 and
  5. What a Rust test can do about a screen is read its source as a string
  and fail the build when a promise disappears from it, which is how
  `dir="auto"`, the no-markup rule, the count's wording and the one
  clipboard write are all held today. Those guards are worth writing and
  they are not coverage: they prove a line of code is present, never that
  the screen behaves. Installing a runner is a new dependency and therefore
  the user's decision, and it has not been asked yet.

## Final reminder

Read this file before every feature. It wins over habit.

---
*Drafted by /audit on 2026-08-26. Lines below this point may have been edited by
a person; treat anything you did not write as curated and do not rewrite it.*
