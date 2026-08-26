can you # 0001. The stack

**Status:** Accepted
**Date:** 2026-08-25
**Weight:** heavy
**Plan row:** none (this record governs the whole project, not one row)
**Supersedes:** nothing

## In one line

EchoScribe is built as a Tauri desktop app (Rust core, web-based interface)
so the interface can mostly carry over to a future Mac version; it signs
people in with Clerk, transcribes live through Deepgram with each person
using their own free Deepgram account, and keeps everything else on the
person's own machine in a single SQLite file — no server of our own to
build, host, or pay for.

## What this is for

A new project with no code yet needs its foundation chosen once, in a way
that everything else — every feature, every future `/architect` and
`/develop` run — can build on without re-litigating it. This is that
choice.

## Constraints gathered before any option was offered

- **Budget:** near-free running costs today. No comfort with a fixed
  monthly bill yet.
- **Who touches this:** just the user, working with AI assistants. No
  handoff to another developer planned.
- **Refusals:** none. Nothing ruled out by prior bad experience.
- **Who runs it:** just the user for now, but they already know they want
  to let other people install and use it later — so "just me" shortcuts
  that would box that out were avoided even in phase 1.
- **The product feel that drove the framework pick:** a small always-on-top
  "pill" that shows a live waveform while dictating, and a strong emphasis
  on speed — words appearing while still talking, not after a pause.

## The decision

**Application framework: Tauri** (Rust for the core, a web-based interface
for the screens, including the floating pill). Chosen over Electron
(same cross-platform interface reuse, but a much heavier app sitting in
the background all day — 150 to 300 MB and more memory, because it bundles
a full browser) and over a Windows-native `.NET`/WinUI build (smallest and
snappiest, and the best-documented option for exactly the OS features this
app needs, but the interface would not carry over to Mac at all — that
becomes a second app built from scratch). Tauri's web-based interface is
also the fastest realistic path to a good-looking animated waveform pill,
which was the clearest thing the user described wanting.

**Local storage: SQLite**, a single file on the person's machine. Decided
without a separate question — this layer is cheap to change later, and
SQLite is the established, boring choice for exactly this. No cloud
database, because the brief already puts cross-device sync out of scope
for v2.

**No backend server of our own.** Because there's no cross-device sync to
support, the app can talk to Clerk directly for identity and to Deepgram
directly for transcription, and keep all actual data (words, history)
local, partitioned by whichever account is signed in. This removes an
entire layer (hosting, a server language, a server framework) that would
otherwise have been its own decision.

**Sign-in: Clerk.** No sign-in provider available today has a true
plug-and-play desktop kit; all of them, Clerk included, end up using the
same well-established pattern for native apps — the system browser opens
to sign in, then hands a session back to the app, the same way GitHub
Desktop or Spotify's desktop app do it. Given that, the choice came down
to cost and documentation quality. The user chose Clerk directly (over the
free-tier-cheaper Supabase Auth) for its documentation and polish,
accepting that its docs lean web-first and the native flow will need to be
built by hand either way.

**Transcription: Deepgram**, using streaming (words appear while still
talking), because that live feel is the "very fast" experience the user
pointed to. Deepgram was the only option confirmed under 200ms of
streaming delay, the cheapest per minute streaming, and comes with the
most generous free credit. It covers 36+ languages, the narrowest list of
the three researched, which is a real limit to revisit once feature 4
("speak in your language") is designed — the target language list should
be checked against Deepgram's coverage before that feature is built.

**Who pays for transcription once this isn't just the user: each person
brings their own Deepgram account.** Deepgram's $200 free credit and all
usage billing is scoped to one Deepgram account (confirmed against
Deepgram's own docs), not per end-user of an app built on it — so a
shared, developer-owned key would mean the user's own account absorbs
every future user's usage, with no billing feature planned to recover
that cost (billing is explicitly out of scope for v2). Instead, each
person who signs into EchoScribe provides their own Deepgram API key,
getting their own 433-hour free allowance, and the app never pays for
anyone's usage but validates and stores the key it's given. This costs
the project nothing as it grows, at the price of one extra setup step
for every new person, and a place in settings to hold that key.

**Frontend UI approach: plain HTML, CSS, and JavaScript**, no framework,
for the pill and any settings screens. Decided without a separate
question — a small number of simple screens does not need a framework's
overhead, and this keeps the dependency count down. Revisit only if the
interface grows past a handful of simple views.

## What else was considered

| Layer | Option | Why not |
|---|---|---|
| Framework | Electron | Same Mac-reuse benefit as Tauri, but 2 to 3x the footprint for an app that runs in the background all day |
| Framework | .NET / WinUI | Best native fit and best-documented for hotkeys and text injection, but the interface doesn't carry over to Mac at all |
| Framework | Flutter Desktop | Text injection support on Windows could not be verified during research; global hotkeys rely on community plugins rather than an established core |
| Sign-in | Supabase Auth | Cheaper at scale and open source, but the user preferred Clerk's documentation and polish despite the higher per-user cost |
| Sign-in | Firebase Auth | Strongest non-web SDK story of the group, but ties the project into the wider Google ecosystem for no benefit this project needs |
| Transcription | Google Cloud Speech-to-Text | Far wider language coverage (125+), but costs more per minute and its streaming speed could not be confirmed as fast |
| Transcription | OpenAI | The most recognizable name, decent coverage and price, but its live-streaming speed for this use case could not be confirmed |
| Transcription billing | One shared developer-paid account | Simplest experience for new users, but the free credit is shared across everyone, could run out in weeks with a handful of daily users, and there is no billing feature planned to recover the cost |

## Value sourcing

Not applicable in the usual sense — this record has no acceptance criteria
of its own, since it governs the whole project rather than one feature.
The one value worth naming a source for because it recurs in every
feature after this: **the signed-in account's identity**, used to
partition local storage (words, history) per person. Comes from the
session Clerk hands back to the app after the browser-based sign-in flow.
The Deepgram API key used for a given person's transcription calls comes
from what that person enters in settings after signing in, stored locally.

## Agent skills

- Deepgram: installed the official `deepgram/skills` skill (6 skills:
  api, docs, examples, recipes, setup-mcp, starters). Scanned safe/low
  risk by Gen and Socket; Snyk flagged "Med Risk" on the `api` and
  `setup-mcp` skills specifically, not blocking, worth a glance before
  relying on those two.
- Clerk: this environment already has several Clerk skills installed
  (clerk-backend-api, clerk-cli, clerk-custom-ui, clerk-expo-patterns,
  clerk-orgs, clerk-setup, clerk-testing, clerk-webhooks). Not
  re-installed; use these when building the sign-in feature.
- Tauri: no official skill exists as of 2026-08-25. Two third-party
  candidates found (`dchuk/claude-code-tauri-skills`,
  `full-stack-skills/tauri-skills`), neither read or installed — the
  user chose to skip and rely on Tauri's own documentation at build
  time instead of trusting an unread third-party skill.

## MCP servers

None adopted. Nothing surfaced during this decision that needs a live,
permanently-connected server: Clerk and Deepgram are both called through
normal SDK/API calls made from the app itself, not something a session
needs standing access to on every request.

## What this makes harder

- **Rust is the core language.** It has fewer training examples than
  JavaScript or C#, so AI-written changes to the core (hotkey handling,
  audio capture, text injection) may need more iteration to get right the
  first time.
- **BYOK (bring your own key) adds friction.** Every new person needs a
  Deepgram account and a key pasted into settings before dictation works
  at all — there is no "just sign in and go" path for transcription,
  which cuts against the daily, unprompted-use goal in the brief if that
  setup step is clunky. The dictate-with-a-hotkey feature record should
  treat that first-run setup as a real, load-bearing screen.
- **No server means no place to put anything that needs to live outside
  one machine.** If a future feature needs something server-side (shared
  state, a scheduled job, anything not tied to one signed-in person on one
  device), that is a new decision, not a small addition.
- **Deepgram's 36+ languages is a real ceiling**, narrower than Google's
  125+. If "speak in your language" needs a language Deepgram doesn't
  cover, that reopens the transcription choice.
- **Clerk's desktop story is unverified in practice.** Its documentation
  leans web and mobile; the native browser-handoff flow will need to be
  built and proven during the sign-in feature, not assumed to work
  smoothly from the docs alone.

## Still open

- The exact list of languages "speak in your language" must support,
  which determines whether Deepgram's coverage is actually sufficient.
  Settle this when that feature is designed.
- The first-run experience for getting a person their own Deepgram key
  (a guided setup screen, a link out to Deepgram's signup, how the key is
  validated and stored). Belongs to the dictate-with-a-hotkey or sign-in
  feature record, not this one.
- Whether the third-party Tauri skills are worth reading and installing
  once real Tauri build work starts and their absence is actually felt.

## References

- Framework comparison: [PkgPulse 2026 desktop frameworks
  guide](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026)
- Speech-to-text pricing and benchmarks: [FutureAGI speech-to-text APIs
  2026](https://futureagi.com/blog/speech-to-text-apis-in-2026-benchmarks-pricing-developer-s-decision-guide/),
  [AssemblyAI vs Google Cloud
  comparison](https://www.assemblyai.com/blog/google-cloud-speech-to-text-alternatives)
- Auth provider comparison: [DesignRevision auth providers
  compared](https://designrevision.com/blog/auth-providers-compared),
  [BuildMVPFast authentication API
  costs](https://www.buildmvpfast.com/api-costs/authentication)
- Deepgram pricing and billing scope: [Deepgram
  Pricing](https://deepgram.com/pricing), [Deepgram: Managing
  Projects](https://developers.deepgram.com/guides/deep-dives/managing-projects)
- Free-forever tiers: [Google Cloud Speech-to-Text
  Pricing](https://cloud.google.com/speech-to-text/pricing), [Azure AI
  Speech
  Pricing](https://azure.microsoft.com/en-us/pricing/details/cognitive-services/speech-services/)
- Skill sources: [deepgram/skills](https://github.com/deepgram/skills),
  [dchuk/claude-code-tauri-skills](https://github.com/dchuk/claude-code-tauri-skills)

Full research notes:
`docs/.agent-cache/research/stack-0001.md` (cached 2026-08-25, reuse for
30 days).
