# 0008. Instruments: what the app records about itself, and where it goes

**Status:** Proposed
**Date:** 2026-10-06
**Weight:** medium
**Plan row:** none. A cross-cutting standard, not a feature. It was owed by the
release readiness list in `docs/plan.md`'s header and by the `/liftoff` gate.
**Supersedes:** nothing

## In one line

EchoScribe sends nothing about errors or usage off a person's machine, ever,
and writes its own diagnostic lines to one small local log file beside the
database so that a problem on somebody else's computer can be read rather
than guessed at. It buys a debuggable app that keeps the promise its privacy
policy already makes. It costs one more file in the app data folder and one
sentence in the privacy policy saying so.

Decided by the user on 2026-10-06, both halves, from a recommended option
each: "No, nothing ever" and "Local log file, always on".

## What this is for

Two things were undecided and both bite at release. First, whether the app
may ever report a crash or count a usage to a service of ours. The privacy
policy published on 2026-10-05 says it does not ("collects no analytics, no
usage statistics and no crash reports"), AGENTS.md says there is no server,
and nothing in the code does it, but nothing recorded that this is a decision
rather than an omission, so a well meant crash reporter could have arrived
with the next bug. Second, where the app's diagnostic lines go today. There
are 91 `eprintln!` calls across 18 files, every one of them written to
explain a failure, and on a Windows release build `main.rs` hides the console
(`windows_subsystem = "windows"`), so they go nowhere. A person who writes to
hello@ saying "it stopped working" has nothing to attach, and the only way to
understand their bug is to reproduce it.

## What happens now

Sampled 2026-10-06, counts from `grep`:

| Way | Count | Where |
|---|---|---|
| `eprintln!("<feature>: <plain sentence>: {e}")`, a feature prefix and one sentence, the library error appended | 91 | every feature folder; sign_in 30, dictate 36, vocabulary 7, language 4, shell 4, history 3 |
| `println!`, `dbg!`, a `log` or `tracing` crate, any network reporting | 0 | nowhere |

One pattern, already universal, already carrying a feature prefix so a line
can be traced to its folder. Four lines print a value with `{:?}`; each was
checked and each is a reason code or an error object the code comments
already certify holds no token (`clerk.rs:303` and `:310` say so in words;
the one 2xx body that could carry a token is deliberately not printed).

## Acceptance criteria

- **AC-1**: While I use EchoScribe, it talks to exactly three places outside
  my machine: Clerk (to sign me in and keep me signed in), Deepgram (my own
  account, for transcription) and my browser (which it opens once at
  sign-in). It never opens a connection to any other host, for any reason,
  including a crash.
- **AC-2**: After I have run EchoScribe once, there is a file named
  `echoscribe.log` in the same folder as `echoscribe.sqlite3`, and every
  diagnostic sentence the app has to say appears in it, each with the time it
  was written.
- **AC-3**: Nothing I dictated, no audio, my Deepgram key, my sign-in session,
  and no sign-in code ever appear in that file. The file is safe to email.
- **AC-4**: The file never grows without limit. It is capped at 1 MB, with one
  previous file kept beside it, and nothing else accumulates.
- **AC-5**: If EchoScribe crashes, the last line in the file says so, with the
  reason the program gave, before the process ends.
- **AC-6**: The privacy policy names the log file, says what it holds and what
  it never holds, and says it stays on my machine unless I send it.

## The decision

**Nothing leaves the machine.** No crash reporter, no error tracker, no
analytics, no usage counter, no "phone home" of any kind, now or later. The
three hosts in AC-1 are the whole list, and a fourth is a new decision record,
not an addition. This is the existing privacy policy sentence made binding on
the code.

**One local log file, always on.** At startup, before anything else runs, the
Rust core points its standard error stream at
`%APPDATA%\com.echoscribe.app\echoscribe.log`, opened for appending. Every
existing `eprintln!` line then lands there unchanged, with no conversion of
the 91 call sites, and so does the message of any panic, because Rust's
default panic output is that same stream. The file is rotated by the app at
startup: if it is over 1 MB it is renamed to `echoscribe.log.1`, replacing the
previous one, and a fresh file is begun. Each line is prefixed with the time
in UTC by the one writer that the stream is pointed at. The one place this
happens is `src-tauri/src/lib.rs`, where `run()` already wires the features in
order; it is not a feature folder's job and it is not a shared module, because
no feature calls it.

**The standard, precisely.** A diagnostic line is `eprintln!` with a feature
prefix, a plain sentence, and at most the library's own error appended with
`{e}`. It never formats a transcript, a dictation's text, a Deepgram key, an
access or refresh token, an authorization code, a code verifier, or raw bytes
of a response body. A `{:?}` is allowed only on a type the line's own comment
certifies to hold none of those. No other logging crate is added.

## What else was considered

| Option | Why not |
|---|---|
| Opt-in crash reports to a service such as Sentry | A new dependency, a new third party holding stack traces that can carry file paths and account ids, a privacy policy rewrite, and a Settings surface to draw and verify. The user declined it; the recommended option was to decline. |
| Always-on crash reports | Breaks the published policy and the no-server rule. Not offered as recommended. |
| Keep stderr only, no file | Costs nothing and keeps nothing. Every bug on another machine stays a reproduction job. Declined by the user. |
| A log file with an off switch in Settings | One more row to design, build and verify for a file that already never leaves the machine. Declined by the user. |
| Converting all 91 lines to a `log` or `tracing` crate | A dependency and 91 edits to reach the same file. Redirecting the stream reaches it with none, and keeps the one pattern that already exists. |
| Rotating by date, or keeping several files | More to explain in the policy, more to cap. One current and one previous answers "what happened just before it broke", which is the only question the file exists for. |

## Position on existing code

New code only, and the existing 91 lines are not converted: the stream
redirect makes them correct as they are. The four `{:?}` lines keep their
certifying comments. No row is added to the plan for conversion.

## Data model

Nothing is stored in the database. The log file is not a record and belongs
to no account: it is a diagnostic stream, and it holds no field that AC-3
forbids. It lives beside the database so that "delete the application data
folder" in the privacy policy removes it too.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The three permitted hosts | AC-1 | Record 0003 (Clerk's production instance), record 0002 (Deepgram's streaming host), record 0003's browser hand-off. Fixed in `sign_in/config.rs` and `dictate/transcribe.rs`; never from a setting. |
| The log file's folder | AC-2 | The same app data directory the database already resolves from, per record 0002 and `lib.rs`. One source, not a second copy of the path: the folder comes from the one place `echoscribe.sqlite3`'s path comes from. |
| The log file's name | AC-2 | This record: `echoscribe.log`, previous `echoscribe.log.1`. |
| The time on each line | AC-2 | The system clock, UTC, written by the redirecting writer. |
| What a line may contain | AC-3 | This record's standard, and the data rules in AGENTS.md. |
| The cap | AC-4 | This record: 1 MB, checked once at startup. A judgement: a line is about 80 bytes, so this is roughly 12,000 lines, days of ordinary use. |
| The crash line | AC-5 | Rust's panic output, which already goes to standard error. A panic hook is only needed if the default message proves not to reach the file; `/develop` proves it by panicking on purpose once. |
| The policy sentence | AC-6 | `site/privacy.html`, "What EchoScribe stores on your computer". Wording is the user's to approve before it is published, per the standing rule on changing what a person reads. |

## Interface surface

None. No command, no screen, no setting, no capability widened. The interface
never writes to the log and never reads it. A person finds the file the same
way they find the database: the privacy policy tells them the folder.

## Enforcement

- A source guard in `lib.rs`'s tests, in the style of the existing ones, that
  fails the build if the redirect leaves `run()` or moves below the first
  feature's `init()`.
- A source guard that reads every `.rs` file and fails if an `eprintln!` line
  formats an identifier named `transcript`, `text`, `key`, `secret`, `token`,
  `code`, `verifier` or `body`. Loose on purpose; a false positive is reworded,
  not exempted.
- AC-1 is enforced by the existing dependency list: there is no HTTP client
  pointed anywhere but the two hosts, and adding one is a new library, which
  AGENTS.md already makes a decision. `/warden` checks the host list on every
  pass.
- The rule goes into AGENTS.md through `/sync`, not here.

## Risk

This touches personal data only in the negative: the log must not hold it.

1. **Worst thing a malicious person could do.** Read the log file and learn
   something a person said or a secret. Answered by AC-3 and the guard: the
   file is designed to be safe to email, so it is safe to read.
2. **What we store that must be protected.** Nothing new. The file holds
   failure sentences and library errors. Account ids may appear in them
   (`user_...`), which is an identifier and not a secret; it is not forbidden
   and is noted here so nobody is surprised.
3. **What the system refuses even if asked nicely.** To send the file
   anywhere itself. There is no upload, no "send report" button, and a
   request for one is a new record that would also have to change the policy.

## What this makes harder

- A person who has never heard of an app data folder has to be told where
  the file is. The policy sentence (AC-6) and the hello@ reply template carry
  that.
- Panics inside the WebView or in Windows' own code do not go through Rust's
  stream and will not be in the file. Known, accepted: those are rare and the
  file still carries everything the app said up to that moment.

## Build plan

One milestone, light enough for one sitting:

1. Redirect standard error in `run()` before the first `init()`, rotate at
   startup, timestamp each line. Prove AC-2 and AC-5 live: launch the
   installed app, find the file, read a real line in it; then a deliberate
   panic in a dev build and the line it leaves.
2. The two source guards above.
3. The privacy policy sentence, drafted for the user's approval, then
   published with the site. AC-6.

## Still open

- Nothing. The cap, the name and the folder are judgements recorded above and
  can be changed by amendment if use shows they are wrong.
