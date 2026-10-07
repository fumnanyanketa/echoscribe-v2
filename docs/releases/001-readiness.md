# Release readiness: EchoScribe v0.1.0, written 2026-10-06

**Version:** v0.1.0, tag on `dd2bb96`, GitHub release published 2026-10-05
15:18 UTC with one asset, `EchoScribe-Setup.exe` (4,785,689 bytes), reached
through the stable link
`https://github.com/fumnanyanketa/echoscribe-v2/releases/latest/download/EchoScribe-Setup.exe`
and handed out by the Kit email gate on `https://echoscribe.exentrik.co`.

**Where it is.** This version is already live. The user put it on production
on 2026-10-05 and dictated with it the same afternoon ("We are live"), before
this checklist was run. As of this morning the release asset shows **0
downloads**, so the circle it has reached is the user alone. This record is
written after the fact, so that the next step, letting anyone else have the
link, is a decision with the checklist in front of it rather than behind it.

**Verdict of this checklist: BLOCKED for a wider circle.** Four lines fail.
Nothing here is a reason to pull v0.1.0 from the one person who has it.

## The checklist

| # | Line | State | Proof, or what fails |
|---|---|---|---|
| 1 | The real thing, on a real device | **pass** | The installed release build, not the dev setup: the user's own production dictation on 2026-10-05, visible today as the one History row ("We are live, baby. In production.", 218 characters, `en`) in `docs/evidence/night-shell-2026-10-06/local-only/01-dashboard-at-rest.png`. The whole pipe (sign-in, hotkey, speech, typing, history, language, vocabulary, restart) was proved on 2026-10-01 in `docs/evidence/end-to-end-2026-10-01/report.md`, on a dev build |
| 2 | The ugly cases hold | **fail** | Proved live and filed under `docs/evidence/dictate-with-a-hotkey/`: the microphone dying mid dictation, a rejected key mid dictation, the five minute cap, silence, a password field, the pill with no monitor. **Not proved on the production build:** no internet at launch (record 0003 AC-14, the offline row), slow internet during a dictation, the hotkey pressed twice in quick succession, a monitor unplugged. Route: `/check verify` on records 0002 and 0003 with the user's hands for the network and the cable |
| 3 | Phones | n/a | Windows desktop only, by the brief |
| 4 | Development leftovers are gone | **pass** | Release builds hide the console (`main.rs`), there is no test button, no seeded account and no fake data in the interface (the History fact cards draw real counts only, by the Night rule). Clerk's production instance and Google's production client are the ones in the build (`930d1f0`). The two Google test users were made redundant when the consent screen left testing on 2026-10-05 |
| 5 | Secrets clean, history included | **pass** | `docs/reviews/security-2026-10-05.md`: 0 Critical, 0 High. `cargo audit` 0 vulnerabilities. Full history pattern sweep clean; history rewritten on 2026-09-24 to purge personal data. The one Medium (the browser's own Clerk session after sign-out) was closed by `dd2bb96` (`prompt=login`, record 0003 AC-19 to AC-21). Open and accepted: no dedicated secret scanner is installed, which is a tooling decision owed to the user |
| 6 | The three instruments are wired and emitting | **fail** | Decided today in `docs/decisions/0008-instruments.md`, by the user: error tracking **none**, analytics **none**, both by design and already promised by the privacy policy; logging is **a local file that is not built yet**. Today the app's 91 diagnostic lines go to a hidden console and vanish. A person who writes to hello@ has nothing to attach. Route: `/develop 0008` (one sitting), then the privacy policy sentence |
| 7 | The rollback plan is written and rehearsed | **fail** | Written below. **Not rehearsed.** Route: run the rehearsal below once, then tick this line |
| 8 | Tests green, serious findings resolved | **pass, with one owed** | `cargo test`: 352 passed, 0 failed, 2 ignored, this morning, after the two findings of `docs/evidence/night-shell-2026-10-06/report.md` were fixed in the working tree (the sign-in wordmark cascade, the stale comment) and the five new guards went green. `cargo clippy -D warnings` clean. The 2026-09-11 review's blocker (two databases) was fixed in `76a5c3d`/`6461889` and proved live on 2026-10-01. **Owed:** `/sync` has not reconciled that review's Major list against the code since, and `docs/plan.md`'s header still describes the project as it stood on 2026-09-11 |
| 9 | No unratified assumptions | **pass** | Record statuses: 0001 Accepted; 0002 to 0007 In progress; 0008 Proposed. None is `Assumed` |
| 10 | The running cost is known | **pass** | The app calls one paid service, Deepgram, on every dictation, and every person brings their own key and their own allowance (record 0001). The project's own bill is zero. Clerk is on its free tier; Kit's trial ends about 2026-10-19 and drops to its free plan on its own |

**Weight gates, by plan row.** Rows 1 and 2 are heavy. The contract for heavy
is verify, a review on a different model, a warden sweep, and the user's own
written sign-off in the evidence report. The review (2026-09-11, a second
model) and the sweep (2026-10-05) exist. **No evidence report for row 1 or
row 2 carries the user's written approval.** That is the fourth failing line,
and it cannot be done by anyone but the user: read
`docs/evidence/dictate-with-a-hotkey/` and `docs/evidence/night-shell-2026-10-06/report.md`,
then add a line with the date and initials saying it is approved. Rows 3 to 5
are medium: verified end to end on 2026-10-01, `/test` has not run on them as
rows; their stores are covered by the Rust suite.

## Also accepted by the user, recorded here

- **The installer is not code signed.** Accepted on 2026-10-02 for the first
  release. Windows shows the SmartScreen warning to everyone who runs the
  installer, and a tampered copy would look the same as the real one. Revisit
  before any wider distribution. The options, as of 2026-10-06 (prices from
  memory and may be out of date): Azure Trusted Signing at about 10 USD a
  month, which needs an identity check and is the cheapest path to a
  SmartScreen reputation; a conventional OV certificate from SSL.com or
  Sectigo at roughly 200 to 400 USD a year on a hardware token; or stay
  unsigned and tell people what the warning means on the download email.
- **The verify of the latest interface work is a FAIL on paper and fixed in
  the tree.** `docs/evidence/night-shell-2026-10-06/report.md` found the
  sign-in name rendering at 23px instead of 28px. The fix is one rule moved
  in `src/sign-in/sign-in.css`, guarded by
  `the_sign_in_wordmark_keeps_its_own_size`; it is not in v0.1.0 and has not
  been re-checked live. It goes out with v0.1.1.

## Rollback plan

Two things are shipped, and each rolls back on its own.

**The app.** There is no earlier version to return to, so rollback means
taking the download away and telling the people who have it.

1. Mark the GitHub release as a pre-release or delete its asset:
   `gh release delete-asset v0.1.0 EchoScribe-Setup.exe`. The stable
   `releases/latest/download` link then answers 404, so the Kit download
   email's link stops working within seconds.
2. Edit the Kit confirmation email (form 10005424) to say the download is
   paused and why, so new sign-ups get the truth rather than a dead link.
3. Email everyone who already has the link, from hello@, with what to do:
   uninstall from Windows Settings, which removes the app and leaves their
   history file in place.
4. Rehearsal, owed: do steps 1 and 2 against a throwaway release
   (`v0.0.0-rehearsal`) and a Kit test subscriber, time it, and write the
   timing here.

**The site.** `git revert <commit>` on `main` and `git push`; the Pages
workflow redeploys in about 20 seconds. This has been exercised, not as a
rollback but as the same mechanism, on every site publish since `c6beaac`,
most recently `ba103b7` on 2026-10-05.

**Clerk and Google.** Nothing to roll back. Turning Google's consent screen
"Back to testing" limits sign-in to the two test users within minutes if a
sign-in problem appears; the branding verification survives it.

## What to watch in the first hour after widening the circle

- The release asset's download count, and the Kit list for new addresses.
- hello@ for a reply from anyone whose installer was blocked by SmartScreen
  or whose Deepgram key setup stalled (the one unreproduced bug in memory is a
  key setup window stuck on "Checking key").
- Nothing else can be watched, by design: record 0008 says the app reports
  nothing. The first signal of a crash on someone else's machine is their
  email, with the log file attached once 0008 is built.

## The go decision

v0.1.0 to the user alone: taken by the user on 2026-10-05, in use since.

v0.1.0 or v0.1.1 to anyone else: **not taken.** This record blocks it on
lines 2, 6, 7 and the heavy sign-off. When those are green, the user writes
the go line here, with the date, and the first circle is friends on their own
laptops, told about the SmartScreen warning and that sign-out now always asks
who is signing in.

Go line: _____________________________ (user, date)
