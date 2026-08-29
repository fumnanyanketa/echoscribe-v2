# 0003. Sign in

**Status:** In progress
**Date:** 2026-08-28
**Weight:** heavy
**Plan row:** 1
**Supersedes:** nothing

## In one line

EchoScribe signs people in by opening the system browser to Clerk and
running a standard OAuth flow with PKCE, so no secret ships in the app; the
Rust core keeps the resulting refresh token in Windows Credential Manager
and renews it silently, so a person stays signed in across restarts and can
still use the app offline. It buys a real account that every setting and
every dictation belongs to, with no server of ours. It costs a long lived
identity token sitting on the machine, a sign-in flow hand wired against
Clerk with no official desktop SDK, and Clerk being required for the very
first sign-in.

## What this is for

Someone opens EchoScribe and needs an account, so their settings, their
custom words and their dictation history belong to them and stay theirs
across restarts. This is the feature that gives every other feature a
person to attach data to. Record 0002 (dictate with a hotkey) is blocked
on it: that record's AC-16, AC-17 and AC-18 all need the signed-in account
id this feature produces, and its data rule that no row is written without
an account id needs the account this feature defines.

## Acceptance criteria

- **AC-1**: On a fresh install with nobody signed in, the app opens to a
  sign-in screen and nothing else. No hotkey, no history, no settings are
  reachable.
- **AC-2**: Choosing "Sign in" opens the system browser to Clerk's hosted
  sign-in page, and the app shows a waiting state with a way to cancel.
- **AC-3**: Entering my email and the one-time code Clerk sends signs me
  in. The browser shows a page telling me to return to EchoScribe, and the
  app moves to the signed-in state on its own within a few seconds, with no
  restart.
- **AC-4**: With no account yet, the same flow lets me create one by
  entering an email, the code, and a name, and I end up signed in.
- **AC-5**: If I do not give a name, the account area shows the part of my
  email before the @, with initials taken from it.
- **AC-6**: After signing in, I fully close the app, reopen it, and I am
  still signed in. No browser, no code, straight into the app.
- **AC-7**: The signed-in state shows my name or email and the date I
  signed in.
- **AC-8**: As long as I do not sign out, I am never asked to sign in again
  on this machine, across any number of restarts.
- **AC-9**: Signing out returns the app to the sign-in screen. My history,
  my custom words and my saved Deepgram key stay on the machine.
- **AC-10**: After signing out, signing back in as the same account shows
  my history, words and key exactly as they were.
- **AC-11**: If a different person signs in on this machine, they see none
  of my dictations, none of my words, and cannot use my Deepgram key.
  Signing back in as me restores all of mine. (Record 0002 AC-18 seen from
  the sign-in side.)
- **AC-12**: If I close the browser without finishing, the app leaves the
  waiting state within a bounded time, says the sign-in did not finish, and
  offers to try again. Nothing partial is saved.
- **AC-13**: If the code is wrong or expired, Clerk's page says so and the
  app stays signed out. I can start again without restarting.
- **AC-14**: Once signed in, if I later open the app with no internet or
  with Clerk unreachable, I still reach the app with my identity from last
  time, history and settings work, and the app shows it is working offline.
- **AC-15**: Signing out always works, even with no internet. Signing in
  always needs the internet; attempted offline it says so, does not open
  the browser, and changes nothing.
- **AC-16**: If my session is ended elsewhere, by signing out on another
  device or by it being revoked, the next time the app reaches Clerk it
  returns to the sign-in screen and says the session was ended, rather than
  silently continuing online.
- **AC-17**: The web interface never receives the session token, the
  refresh token, or my Clerk code. Inspecting what the interface is sent
  shows none of them.
- **AC-18**: A brand-new person reaches record 0002's Deepgram key setup
  only after they are signed in, never before.

## The decision

EchoScribe is registered as an OAuth application in Clerk: a public client,
no client secret. Signing in is the OAuth 2.0 Authorization Code flow with
PKCE, which Clerk added for public clients in November 2025.

The Rust core drives it. On "Sign in" it first checks that Clerk is
reachable, by fetching Clerk's public discovery document at
`/.well-known/openid-configuration` with a five second limit. If that
does not answer, the attempt stops there: no browser opens, no listener
binds, nothing is stored, and the screen says Clerk could not be reached
with a way to try again (AC-15). If it does answer, the core generates a
PKCE verifier and challenge and a random state value, binds a listener on a
free port on 127.0.0.1, and opens the system browser to Clerk's authorize
endpoint with scopes `openid profile email offline_access` and a redirect
back to that loopback address. `offline_access` is the scope that makes
Clerk return a refresh token; without one, AC-6, AC-8 and AC-14 cannot
hold, so it is not optional.
The person signs in on Clerk's hosted pages with a one-time email code.
Clerk redirects to the loopback address with an authorization code, the
listener takes it and shuts down, and the core checks the returned state,
then exchanges the code and the PKCE verifier at Clerk's token endpoint for
an access token, a refresh token and an ID token. The core reads the
account id, email and name from Clerk's userinfo endpoint, writes the
`account` and `session` rows, and stores the refresh token and current
access token in Windows Credential Manager. The browser and interface never
see any token.

On every launch the core reads the single `session` row. If it names an
account, the app shows that account straight away, then tries a silent
refresh in the background. A refresh that Clerk refuses as revoked or
expired signs the person out and returns to the sign-in screen with a
reason. A refresh that fails because Clerk cannot be reached leaves the
person signed in with their cached identity and marks the state offline.
While the app runs, the core renews the access token before it expires.

Signing out empties the `session` row, deletes the Credential Manager
entry, and best effort revokes the refresh token at Clerk's
`/oauth/token/revoke` endpoint if it is reachable. It never deletes an
`account` row or any account owned data. One account is active at a time;
signing in as someone else signs the first out and leaves their local data
in place for when they return.

This feature has exactly one notion of being online: Clerk answered. It
never asks Windows whether a network exists, because a connected machine
behind a hotel portal or a filtering work network is not a machine that can
sign in, and answering that question wrongly is worse than not asking it.
There is one mechanism, used in two places and no more. Starting a sign-in
asks the discovery document, because there is no other call to learn it
from. Launching the app asks nothing extra, because the silent refresh is
itself the probe: it succeeds, it is refused, or it cannot be reached, and
the third case is what puts the app in `signed_in_offline` for AC-14. The
check is never cached between attempts, since the whole point is that
network conditions changed since last time.

## What else was considered

| Option | Why not |
|---|---|
| Hand-built Clerk Frontend API client in Rust, mimicking clerk-js | Clerk does not document this for non-browser clients. Any change to how clerk-js manages a client could break it, with no SDK or changelog to follow. The OAuth application path is a documented, stable contract. |
| Unofficial community Tauri Clerk plugin | Unindexed, unread, unsupported by Clerk. The stack decision already declined unread third-party skills for this project. |
| Embed clerk-js in a hidden WebView and pass tokens to Rust | Would work, but the session would pass through and briefly live inside a WebView, which the architecture rules say it must never do. |
| Custom URL scheme, echoscribe://, for the handoff | Needs registering a protocol at install time, and another installed program can register the same scheme and intercept the handback. A random loopback port cannot be claimed by another process mid-flow. |
| Device authorization (device code) flow | Clerk treats it as a fallback for headless machines. Making a person read and type a code is worse than opening a browser when the app can open one. |
| Email plus password sign-in | Brings password reset flows and the credential attackers most often guess, phish or reuse. Passwordless email code removes a whole class of attack for a personal tool. |
| Session tied to the Windows login, or expiring on a fixed schedule | More re-sign-in friction for little gain on a personal desktop tool. The bar is close the app, reopen it, still be signed in. |
| Wipe local data on sign out | Simple, but breaks the promise that a person's words belong to them. Sign out keeps the data; explicit account deletion, a later feature, removes it. |
| Block the app whenever Clerk is unreachable | Safest against a revoked session, but locks a person out of their own history over a flaky connection. Cached identity for at most one app session is the accepted trade. |
| Encrypted file for the tokens instead of Credential Manager | The app would hold the encryption key next to what it protects. The same reason record 0002 rejected this for the Deepgram key. |
| Asking Windows whether there is an internet connection, before sign-in | Instant and sends nothing, but it answers the wrong question. Windows reports a hotel portal or a filtering work network as connected, so the browser would open anyway and the person would still wait out the five minute timeout. It also needs new Windows-only code behind the platform boundary, for a worse answer than one HTTPS request gives. |
| Opening the browser regardless and letting it fail | What the milestone 1 build does today. The browser shows its own no-internet page while EchoScribe sits on the waiting screen for the full five minutes before saying anything. Cheapest to build, and it leaves AC-15 unmet. |
| Probing something neutral, such as a well-known connectivity URL | Tests the internet rather than the dependency, and it means talking to a host that has nothing to do with EchoScribe. Clerk's own discovery document is public, tiny, needs no token, and is exactly the server that has to answer. |

## Data model

**Table `account`**: one row per Clerk user that has ever signed in on this
machine.

| Field | Type | Required | Notes |
|---|---|---|---|
| `id` | text | yes | The Clerk user id, the `sub` from Clerk userinfo. Primary key. Never generated locally, never reused. This is the `account_id` every record 0002 table references. |
| `email` | text | yes | Primary email from Clerk. Drives the account block and the initials fallback. |
| `display_name` | text | no | Name from Clerk, if given. Empty means fall back to the part of the email before the @. |
| `first_signed_in_at` | text | yes | UTC ISO 8601. First sign-in on this machine. Never changes. |
| `last_signed_in_at` | text | yes | UTC. Updated on every successful sign-in. The "signed in since" date the design shows. |

**Table `session`**: a single row. Who is active, and where the token
lives.

| Field | Type | Required | Notes |
|---|---|---|---|
| `account_id` | text | no | The active account's `id`, or empty when signed out. References `account.id`. |
| `credential_target` | text | no | Name of the Windows Credential Manager entry holding the refresh token and cached access token. Empty when signed out. |
| `established_at` | text | no | UTC. When the current session began at sign-in. |
| `last_verified_at` | text | no | UTC, may be empty. Last time Clerk confirmed the refresh token still works. Lets the app reason about a stale offline session. |

Relationships: `account` has many `dictation`, zero or one
`dictation_setting`, zero or one `deepgram_credential`, all defined in
record 0002. The single `session` row points at zero or one `account`.

Rules that must always hold:

- `account.id` is the Clerk user id exactly as Clerk issues it. Never
  minted locally, never reused for a different person.
- Exactly zero or one `session` row exists at any time. An empty
  `account_id` means signed out.
- No token is ever stored in the database. The refresh token and the
  current access token live only in Windows Credential Manager;
  `session.credential_target` holds only the entry name.
- Signing in as an account that already has a row updates
  `last_signed_in_at` only. It never touches that account's dictations,
  settings or Deepgram credential.
- Signing out empties the `session` row and deletes the Credential Manager
  entry. It deletes no `account` row and no account owned data.
- Deleting an account, defined in record 0002 and built by a later
  feature, removes its `account` row, all its `dictation` rows, its
  `dictation_setting`, its `deepgram_credential`, and its Credential
  Manager entries for the session token and the Deepgram key.
- `account_id` on every record 0002 table references `account.id`. The app
  keeps this consistent and SQLite foreign keys are switched on.
- No email, display name, Clerk id or token is ever written to a log or to
  any file other than the SQLite database.

One migration creates `account` and `session`. It runs before record
0002's migration, because that record's tables reference `account.id`.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| Whether anyone is signed in | AC-1, AC-6, AC-8, AC-18 | Presence and `account_id` of the single `session` row, read at launch. |
| Sign-in screen versus app, at launch | AC-1, AC-6 | `get_auth_state()`, computed in Rust from the `session` row and the last refresh result. |
| Whether a sign-in can start at all | AC-15 | A GET of Clerk's discovery document, `https://<instance domain>/.well-known/openid-configuration`, made in Rust inside `start_sign_in` before anything else happens. It answers: proceed. It does not: stop, and return `cannot_reach_clerk`. Checked fresh on every attempt, never cached. |
| The limit on that check | AC-15 | A fixed five seconds in this record, beside the five minute sign-in timeout. Long enough for a slow phone hotspot, short enough that a dead connection is not a stare. Fixed, not a setting. |
| Clerk's hosted sign-in URL | AC-2 | Built in Rust from Clerk's authorize endpoint, the OAuth client id, the instance domain, the `openid profile email offline_access` scopes, the loopback redirect, and a fresh PKCE challenge and state. Client id and instance domain are configuration, set once. |
| The loopback redirect URL | AC-2, AC-3 | `http://127.0.0.1:<port>`, where `<port>` is a free port the Rust core binds at the start of each sign-in. Not stored. |
| The one-time code | AC-3, AC-4, AC-13 | Entered by the person on Clerk's page. Never seen by EchoScribe. |
| The authorization code | AC-3 | Returned by Clerk to the loopback URL after sign-in. Held in memory only, exchanged once. |
| PKCE verifier and state | AC-3, AC-12 | Generated in Rust at the start of each sign-in, held in memory for that attempt only, never stored. |
| Access token and refresh token | AC-3, AC-8, AC-14, AC-16 | Clerk's token endpoint, in exchange for the authorization code and the PKCE verifier. The refresh token comes back only because `offline_access` was among the requested scopes. Stored in Windows Credential Manager, read only by Rust. |
| The account id | AC-11, AC-18, and record 0002 AC-16, AC-17, AC-18 | The `sub` from Clerk userinfo, taken at sign-in. Written to `account.id` and `session.account_id`. Never passed in from the interface. |
| Email | AC-5, AC-7 | Clerk userinfo, fetched with the access token at sign-in and on refresh. Stored in `account.email`. |
| Display name | AC-5, AC-7 | Clerk userinfo `name`, same fetch. Stored in `account.display_name`. Empty if none was given. |
| Name and initials shown when there is no display name | AC-5 | Derived in Rust from `account.display_name` if set, otherwise from the local part of `account.email`. |
| "Signed in since" date | AC-7 | `account.last_signed_in_at`, set in Rust as UTC on each successful sign-in. |
| Still signed in across restarts | AC-6, AC-8 | On launch, a non-empty `session.account_id` plus a successful, or when offline a skipped, refresh exchange. |
| Silent refresh | AC-8 | Rust exchanges the stored refresh token for a new access token before expiry, and when a call is rejected as unauthorised, using the `oauth2` crate. |
| Offline identity | AC-14 | The `account` row for `session.account_id`, shown without contacting Clerk when the network or Clerk is unreachable. |
| "Working offline" indicator | AC-14 | Rust returns `signed_in_offline` from `get_auth_state()` when this launch has had no successful refresh yet. |
| Sign-out completion | AC-9, AC-15 | Rust empties the `session` row and deletes the Credential Manager entry locally, then best effort revokes at Clerk's `/oauth/token/revoke`. The local steps never depend on the network, and no reachability check gates them. |
| "Session ended elsewhere" | AC-16 | Clerk's token endpoint rejecting a refresh as an invalid grant, told apart from a network failure, which leaves the session in place. |
| Data separation between accounts | AC-10, AC-11 | Every record 0002 table is filtered by `account.id`, and the Deepgram credential entry is named per account. Enforced in Rust. |
| Reaching the Deepgram setup only after sign-in | AC-18 | Record 0002's key check runs only in the `signed_in` or `signed_in_offline` state, which this feature owns. |
| The bound on an abandoned sign-in | AC-12 | A fixed timeout in this record: five minutes with no callback, after which the listener closes and `auth:sign_in_failed` fires. Fixed, not a setting. |

Configuration and secrets: the OAuth client id and the Clerk instance
domain are embedded in the app and are safe to embed, because the app is a
public client using PKCE. No Clerk secret key exists anywhere in the app.

## Interface surface

Every item below is a Tauri command or event. The interface asks, Rust
decides. No command takes an account id; Rust knows it from the session.
Every command except `get_auth_state`, `start_sign_in` and
`cancel_sign_in` refuses when nobody is signed in.

Commands:

- `get_auth_state()` returns one of `signed_out`, `signing_in`,
  `signed_in`, or `signed_in_offline`. The two signed-in states also carry
  the display name, email, initials and signed-in-since date. Lets the
  interface draw the right screen on load and after a reload.
- `start_sign_in()` first checks that Clerk answers, then generates the
  PKCE verifier and state, binds the loopback listener, opens the system
  browser, and returns once the browser is open, or a named error if Clerk
  could not be reached, the browser could not be opened, or no loopback
  port was free. Does nothing if already signed in. The reachability check
  lives inside this command, not in the interface: the interface asks to
  sign in and is told what happened, exactly as with every other decision
  in this feature. It can take up to five seconds to return, so the screen
  keeps the person on the sign-in view with the action showing it is busy,
  and moves to the waiting view only once this call has come back
  successfully. Nothing is stored and no browser opens on the failing path
  (AC-15).
- `cancel_sign_in()` abandons an in-progress sign-in and shuts the
  listener. For a cancel affordance on the waiting screen.
- `sign_out()` empties the session, deletes the Credential Manager entry,
  and best effort revokes at Clerk. Always succeeds locally, even offline.

Events Rust sends to the interface. None carries a token.

- `auth:signed_in` with display name, email, initials and
  signed-in-since date.
- `auth:sign_in_failed` with a named reason safe to show: browser closed
  before finishing, timed out waiting, security check failed, Clerk
  rejected sign-in, could not reach Clerk.
- `auth:signed_out` with why: you signed out, or your session was ended
  elsewhere.
- `auth:offline` when the app is signed in from cached identity and has not
  reached Clerk yet this launch.

Errors that must each read differently: browser could not be opened, no
free loopback port, sign-in abandoned, sign-in timed out, security check
failed, Clerk rejected sign-in, cannot reach Clerk, session ended
elsewhere.

Who may do what: before sign-in the interface can call only
`get_auth_state`, `start_sign_in` and `cancel_sign_in`. Everything else in
the app, the hotkey, dictation, history, settings and the Deepgram key
screen, is gated on `signed_in` or `signed_in_offline`, enforced in Rust,
per record 0002 AC-16.

Capabilities: nothing is widened, and nothing needs to be. Tauri 2 does not
gate commands this app defines itself; capabilities gate Tauri's own APIs
and plugin commands. The four commands above are reachable because they are
registered on the app, and the four events flow on `core:default`, which
`src-tauri/capabilities/default.json` already grants. That file still holds
`core:default` and nothing else, and the milestone 1 build did not touch
it. The system browser is opened by Rust through the `open` crate rather
than a Tauri plugin, so no opener capability is granted either. Any future
request to add a permission here is still the security decision AGENTS.md
says it is; the point is only that this feature never needed one.

## Risk

Assessed because: weight heavy, and this feature is sign in and personal
data.

**Worst case abuse.** Two are real and both are treated as real.

1. Steal the stored refresh token from Windows Credential Manager and use
   it to impersonate the person against Clerk, indefinitely, until they
   sign out. The refresh token is the only long lived secret and it never
   touches the interface, a log, or a plain file. It sits behind the same
   OS protection as the Deepgram key. Signing out revokes it.
2. Race another local program to intercept the browser handoff, or point
   the app at a fake Clerk to phish the code. A random loopback port with
   PKCE and a state check makes a stolen authorization code useless without
   the in-memory verifier, and calls to Clerk are HTTPS to a fixed domain
   with certificate validation.

A second person with file access to this machine's user profile can read
the local SQLite database. This is the accepted local-first model, the
same as record 0002. Nothing about what a person said leaves the machine.

**Data we must protect.** The refresh token and access token, in Windows
Credential Manager only, never in the database, a log or a plain file. The
account email and display name, in the local database. The link between a
person and every dictation they have made, which is the `account_id` on
record 0002's `dictation` table. There is no password, because sign-in is
passwordless. No payment data. No audio.

**The system must refuse to:**

- Write a token or an email to a log, an error message, or any file other
  than the SQLite database.
- Hand the session token, the refresh token, or the authorization code to
  the web interface, ever.
- Ship or store a Clerk secret key. The app is a public OAuth client only.
- Keep making authenticated Clerk calls after a refresh has been refused as
  revoked. Showing cached identity offline is allowed; continuing online is
  not.
- Accept a sign-in redirect whose state or PKCE does not match the attempt
  in progress.
- Sign a person in without a deliberate action by them. No automatic or
  background sign-in, no silent account creation.

The reachability check adds one outbound request, to a service this
feature already calls. It is an unauthenticated GET of a public document
at the same fixed Clerk domain over HTTPS with certificate validation. It
carries no token, no email, no account id and no query, and its response
is used only as answered or did not answer. It is not a new service and
not a new kind of data leaving the machine.

Checked by /warden after the build against these three.

## Build plan

1. **The sign-in screen and the browser handoff.** The signed-out screen,
   `start_sign_in` opening the browser to Clerk, the loopback listener,
   PKCE and state, the code exchange, tokens into Credential Manager, the
   `account` and `session` tables and their migration, and the account
   block showing name and date. Ends signed in. No dictation yet; record
   0002 stays gated. This milestone proves the part the stack decision
   flagged as unproven.
2. **Staying signed in, and signing out.** Reading `session` on launch,
   silent refresh, `sign_out` with best effort revocation, and the session
   ended elsewhere path. Ends: close and reopen stays signed in; sign out
   returns to the screen. Verify AC-8 against the real Clerk refresh token
   lifetime here.
3. **Offline, and a second account.** The `signed_in_offline` state and
   cached identity, the discovery-document reachability check that stops a
   sign-in before the browser opens, the busy-then-failed path on the
   sign-in screen, and account switching that keeps each account's data
   separate. Ends: all 18 acceptance criteria met, and record 0002 is
   unblocked. AC-15 is checked by pulling the network cable, or disabling
   the adapter, and pressing Sign in: within about five seconds the screen
   says Clerk could not be reached, and no browser window appeared.

## What this makes harder

- **The refresh token is a long lived key on the machine.** It is now the
  most valuable secret EchoScribe stores, more than the Deepgram key,
  because it is the person's identity. Every change near token storage or
  logging needs the same scrutiny as the first one.
- **No official Clerk desktop SDK.** The OAuth application flow is
  standard, but token refresh, revocation and the userinfo call are all
  hand wired against Clerk's endpoints. A breaking change on Clerk's side
  lands on this project, with no SDK changelog to follow.
- **The loopback listener runs for a few seconds during sign-in.** It
  binds a local port and accepts one request. A small surface, present
  only during sign-in, but present.
- **Offline cached identity means a revoked session keeps working until
  the app next reaches Clerk.** A deliberate trade for not locking people
  out of their own history. The window is at most one app session.
- **Every sign-in now costs a round trip before anything visible happens.**
  Up to five seconds of a busy button on a slow connection, where the old
  behaviour opened the browser immediately. A genuinely slow but working
  connection can be told it is offline; the person presses Try again and it
  works, which is confusing rather than harmful. That is the accepted price
  of not stranding someone on a waiting screen for five minutes.
- **The reachability check is a promise, not a guarantee.** Clerk answering
  at the moment the button is pressed does not mean the network survives
  the next thirty seconds. A connection that dies mid-flow still ends at
  the five minute timeout in AC-12. This check removes the common case, not
  every case.
- **"Never sign in again" depends on Clerk's refresh token lifetime**,
  which the research could not pin down. If Clerk caps it, the person
  re-signs-in when it finally expires, and AC-8 degrades to that.
- **Clerk is a hard dependency for the first sign-in.** With no network
  and no prior session, EchoScribe cannot be used at all. This follows
  from the stack decision but is worth stating.

## Still open

- **Clerk's refresh token lifetime, and whether it can be made rolling or
  long lived.** Now half answered: Clerk's documentation says refresh
  tokens issued under `offline_access` do not expire, and the instance
  advertises the `refresh_token` grant. That is a documented claim, not a
  result, so this stays open until milestone 2 exercises a real refresh
  against the real instance and records what happened. If it turns out to
  be capped, AC-8 degrades as noted above.
- **Whether the Clerk instance is configured for email one-time-code
  sign-in and collects a name at sign-up.** Still unproven, and it did not
  settle in milestone 1 as this record expected. The browser already held a
  valid Clerk session from earlier attempts, so Clerk skipped straight past
  the code and the name. That means AC-3, AC-4 and AC-5 have never actually
  been seen. Settle by signing out in the app, then signing out at Clerk's
  own hosted pages, then signing in from nothing. That is a `/check verify`
  step, not a design question, but it stays here because until it is done
  three criteria are unevidenced.
- **A busy state for the primary button on dark.** The reachability check
  can take up to five seconds, and `design/registry.md` names no busy or
  disabled variant of that button. `/canvas` owes one entry before
  milestone 3 draws it. Small, but it is the only thing on screen during
  those seconds.

### Settled during milestone 1

Kept here rather than deleted, so a later reader can see what was open and
what closed it.

- **The exact Clerk OAuth endpoint paths, and public client support.**
  Settled from the live discovery document at
  `https://<instance domain>/.well-known/openid-configuration`. The
  paths in this record are correct as written: `/oauth/authorize`,
  `/oauth/token`, `/oauth/userinfo`. Revocation, which this record had not
  named, is `/oauth/token/revoke`. The instance advertises support for
  public clients with PKCE, which is what the whole decision rests on. Two
  things had to be true and only the instance-wide one was: the OAuth
  application itself was registered as confidential, and Clerk refused
  every token exchange until it was set to `public`. `pkce_required` was
  set at the same time, because this record's Risk section relies on PKCE
  actually being enforced, and an instance merely permitting it does not
  deliver that.
- **Whether opening the system browser needs a new dependency.** It does.
  The scaffold has no opener. The `open` crate version 5 was added during
  milestone 1 and is now in the library table below.
- **The sign-in screen and its states.** Done. `design/registry.md` carries
  the sign-in window, its four states, the waiting indicator, the error
  line and the notice line, all marked drawn. Only the button busy state
  above is outstanding.

## New libraries

Approved during this decision. Installed during milestone 1, except where
a row says otherwise. Each is load bearing, not convenience.

| Library | For | Why this one |
|---|---|---|
| `oauth2` | The Authorization Code plus PKCE flow, the token exchange, and refresh | The established Rust OAuth client crate. Handles PKCE, state and refresh correctly so this project does not hand-roll security-critical code. |
| `tiny_http` | The few-second loopback listener that receives the redirect | Small, synchronous, no async runtime needed for a listener that accepts one request and shuts down. The usual choice for exactly this. |
| `open` | Opening the system browser to Clerk's sign-in page | Not in the scaffold, and needed by AC-2. One small crate that hands a URL to the OS default browser, with a Mac path behind the same call. Chosen over a Tauri opener plugin because Rust opens the browser here, not the interface, so the plugin's capability surface would buy nothing. |
| `reqwest` | HTTPS calls to Clerk's token, userinfo, revocation and discovery endpoints | The standard Rust HTTP client, and the one `oauth2` pairs with. TLS with certificate validation by default. Not a direct dependency: it arrives through `oauth2`'s `reqwest-blocking` feature and is used through its re-export, so there is one version of it and one TLS choice, `rustls`. |
| `keyring` | Storing the refresh token and access token in Windows Credential Manager | Already approved in record 0002 for the Deepgram key. One dependency covers both secrets, with a Mac equivalent behind the same interface. Reused, not new. |
| `rusqlite` | The `account` and `session` tables and their migration | Already approved in record 0002. Reused, not new. |

## References

- Clerk as an OAuth and OIDC identity provider:
  https://clerk.com/docs/advanced-usage/clerk-idp
- PKCE support for custom OAuth, 2025-11-12:
  https://clerk.com/changelog/2025-11-12-pkce-support-custom-oauth
- Adding Clerk auth to a CLI, the loopback listener pattern:
  https://clerk.com/blog/adding-clerk-auth-to-your-cli
- Session tokens: https://clerk.com/docs/guides/sessions/session-tokens
- Customise redirect URLs:
  https://clerk.com/docs/guides/development/customize-redirect-urls
- Account Portal overview:
  https://clerk.com/docs/guides/account-portal/overview
- RFC 8252, OAuth 2.0 for Native Apps:
  https://datatracker.ietf.org/doc/html/rfc8252
- The instance's own discovery document, which settled the endpoint paths,
  the revocation path and public client support:
  `https://hopeful-snail-1633.clerk.accounts.dev/.well-known/openid-configuration`
- Tauri 2 capabilities, which gate Tauri and plugin commands rather than
  commands an app defines itself:
  https://v2.tauri.app/security/capabilities/

Full research notes: `docs/.agent-cache/research/signin-0003.md`
(cached 2026-08-28, reuse for 30 days).
