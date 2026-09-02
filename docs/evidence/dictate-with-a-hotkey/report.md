# Verify: Dictate with a hotkey, record 0002. 2026-09-02

**Live interactive run**, the fifth. The user was at the keyboard for every
voice and ear moment and called the start of each one themselves. Everything
scriptable was driven through the real Windows input queue, and every check was
read mechanically: windows by handle, landed text by `WM_GETTEXT` and compared
as code points, the pill by its own class and title, the pill's contents by
photographing its screen rectangle, the database by SQL, the registry read and
restored by script, stderr read from the dev process.

Nothing below was inferred from reading the code, except where a clause is
explicitly named as taken on the code's word.

Build: commit `3b4fb62` plus its uncommitted working set, 26 files, diff
sha256 `78b9f2c582dda1760a0447a3bf216f3bc9d65a78372e76611121a2c89e46c5b5`.
**Compiled and launched by this sitting**, not inherited: an instance already
running from 10:27 was stopped first, with the user's approval, so the build
under test is one this run made. 159 tests passing with 1 ignored (the step 4a
spike), clippy clean at `-D warnings`. One deliberate restart, for the sound
test.

Earlier reports preserved: `report-2026-08-29-milestone-1.md`,
`report-2026-08-30-milestones-1-3.md`, `report-2026-08-31-milestone-4.md`.

Approach bar: **tracer bullet**, from `docs/plan.md` and AGENTS.md. The whole
path must work end to end for real. It does, repeatedly: hotkey, microphone,
Deepgram stream, grey wording on the pill, and typed words at the cursor in
another app, several times over.

Weight: heavy.

## Verdict: BLOCKED

**No criterion is failing.** The two that were `not met` on 2026-08-31 are both
met, photographed, which is what this sitting existed to settle. Three mid
dictation halves that had never existed to test were forced live and two of
them passed outright. AC-27's click proof was re-run against the new geometry
and passed. Nothing was observed behaving wrongly against any criterion.

But 13 of 33 criteria could not be exercised at all, and 4 of the 12 commands
the record specs do not exist yet. A pass requires every criterion met and
every surface present, so this is BLOCKED, and BLOCKED does not round up.

Of the 13: **six are milestone 5** (AC-12, 17, 18, 19, 21, 22), **four need a
second monitor** (AC-25, AC-26 and clauses of AC-23 and AC-24), **two are
standing user decisions** (AC-29, AC-31), and **one is half forcible** (AC-13).

## The criteria

| Criterion | Outcome | Evidence |
|---|---|---|
| AC-1 pill and moving waveform within 1s | met | ten opens measured, 340 to 444 ms (`AC-27-and-the-one-geometry.md`); bars photographed moving with voice (`frames/AC-33-grey-interim-tail.png`). **Correction**: the last report added a focused-screen clause to AC-1. AC-1 has no such clause; it is AC-23's |
| AC-2 the machine's own device sounds | met | normal pair heard again both ways this sitting; `(None)` half 2026-08-31; deleted-file half heard and logged today (`AC-2-deleted-file-fallback-heard.md`) |
| AC-3 words typed at the cursor while speaking | met | three phrases landed mid speech, read by `WM_GETTEXT` (`AC-33-and-AC-20-now-shown.md`) |
| AC-4 never taken back, only final wording | met | 131 units, code points checked: one space per join, no leading space; grey wording never landed; `dictation` table 0 rows |
| AC-5 second double tap closes, closing sound | met | 501 ms after a tap straight off the pill; six of six toggles landed (`AC-27-and-the-one-geometry.md`) |
| AC-6 Ctrl+C then Ctrl+V does not start | met | re-run today: Ctrl+A, Ctrl+C, Ctrl+V in sequence, no pill, read by handle |
| AC-7 words follow focus to another window | met (standing) | `AC-7-focus-follows-mid-dictation.md`, 2026-08-31. Not re-exercised today |
| AC-8 30s silence or 5 minute cap | met | silence cap fired on its own at 30,641 ms today with nothing typed; 5 minute half `AC-8-five-minute-cap.md` |
| AC-9 no key saved, guided setup screen | met (standing) | `AC-9-10-deepgram-key.md`, 2026-08-30. Not re-exercised today |
| AC-10 valid key checked, saved, dictation works, survives restart | met | the saved key streamed every dictation today, including after this sitting's restart |
| AC-11 invalid key rejected at the setup screen, nothing saved | met (standing) | 2026-08-30. Today's rejection was on the stream, which is AC-13, not this |
| AC-12 Settings shows last four only | blocked | milestone 5. `key_last_four = 64ef` stored, key absent from the database |
| AC-13 saved key stops being accepted | blocked | **rejected half met live for the first time**, a real Deepgram refusal (`AC-13-14-30-mid-dictation-endings.md`). The allowance half cannot be forced, so its next step has never been reached |
| AC-14 connection drops, one reconnect | met | forced live, Wi-Fi cut mid dictation: pill said it, window came forward with Try again, typed words intact. The reconnect count itself is internal and taken on the code's word |
| AC-15 microphone unavailable, cause named, privacy link | met (standing) | `AC-15-28-29-microphone-errors-live.md`, 2026-08-30; `AC-15-mic-error-kinds-standing.md` |
| AC-16 hotkey does nothing signed out | met (standing) | `AC-8-16-silence-cap-and-signed-out.md`, 2026-08-31 |
| AC-17 dictations saved, survive restart | blocked | milestone 5. `dictation` table present, 0 rows after two days of dictating |
| AC-18 second account sees nothing of the first | blocked | milestone 5 |
| AC-19 exactly two hotkeys, either side counts | blocked | settings surface is milestone 5; either-side clause met 2026-08-30 |
| AC-20 password field: nothing typed, pill shows blocked | **met** | both halves. Code and sentence photographed, no action, foreground unchanged, field length 0 either side (`AC-33-and-AC-20-now-shown.md`) |
| AC-21 one sound switch in Settings | blocked | milestone 5. `sounds_enabled` exists and is honoured |
| AC-22 no way to set any other hotkey | blocked | milestone 5, no settings surface to check |
| AC-23 first open bottom centre, focused screen | blocked | a dragged position is stored, so first open cannot recur without resetting the row; focused-screen clause needs a second monitor |
| AC-24 dragged position remembered, carries across screens | blocked | remembering half met exactly again: a 150 px drag moved `pill_x` by 150/1920 to the last digit. Carrying clause needs a second monitor |
| AC-25 pill stays put when focus moves screens | blocked | one monitor. Related fact proved: the window was 1008x236 at 1901,1624 in every state and across six cycles, never resized, never drifted |
| AC-26 screen unplugged or resized | blocked | one monitor |
| AC-27 clicking or dragging never moves my cursor | met | **re-run against the 472x52 pill**: inert click, grip click, 150 px drag, foreground never changed, all twelve characters landed in order (`AC-27-and-the-one-geometry.md`) |
| AC-28 window forward with code, sentence, one action | met (standing) | 2026-08-30 for a microphone that would not open. The same shape was seen three times today on mid dictation endings |
| AC-29 blocked opens privacy page, others Try again | blocked | labels met, and Try again seen as the one action twice today. The page opening and a successful Try again stay untested at the user's standing decision |
| AC-30 nothing asks me to click the pill | **met** | mid dictation half met on a real, unstaged device death: `MIC STOPPED` on the pill, then the window forward with one action. No pill in this sitting ever carried an action |
| AC-31 uses whichever microphone Windows is set to | blocked | "nowhere to choose one" met: no such control exists. The default-change half stays skipped at the user's decision |
| AC-32 an error clears itself once fixed | met (standing) | `AC-32-error-clears-itself.md`, 2026-08-30. Not re-exercised today |
| AC-33 unfinished wording on the pill, grey, never typed | **met** | grey tail with its dotted rule photographed at 5,539 ms, hardening into ink by 8,313 ms; never typed, never stored, never logged (`AC-33-and-AC-20-now-shown.md`) |

**Met: 20** (13 exercised today, 7 standing on cited earlier evidence).
**Not met: 0.** **Blocked: 13.**

The seven standing mets are listed as such on purpose. They were proved on
earlier builds and were not driven again today, and saying so is more useful
than implying a fresh observation.

## The eleventh amendment's 2 second hold

Built, and measured on three different endings: 2,181 to 2,427 ms on the
password refusal, 2,041 to 2,435 ms on the device death, 2,182 to 2,418 ms on
the lost connection. All consistent with a 2 second sleep plus the closing and
sound work after it. Full method and ranges in `word-ending-hold-measured.md`.

## The twelfth amendment's one geometry

The pill window measured **1008 x 236 device pixels on a 3840 x 2160 screen at
200 per cent, which is 504 x 118 logical**: the 472 x 52 pill, the 34 pixel
counter band, and 16 pixels of shadow margin each side. Identical in every
state and across every cycle. The counter band is reserved and photographed
empty; the elapsed and word count chip is deliberately unbuilt and has its own
unticked plan row, and nothing about its future arrival will resize the window.

## Surfaces the record specced

- **Tables: 3 specced, 3 present**, all carrying `account_id`, schemas read by
  SQL. `dictation` holds 0 rows, which is milestone 5's unbuilt history and
  also independent proof that no transcript was ever stored.
- **Commands: 12 specced** (13 minus `pill_drag_finished`, removed by the
  seventh amendment), **8 present**. The four absent are the milestone 5
  hotkey and sound settings pairs: `get_hotkey`, `set_hotkey`,
  `get_dictation_sounds`, `set_dictation_sounds`. Unchanged from the last
  report.
- **Events: 8 specced, 8 present in Rust, and now 8 of 8 read by the
  interface.** Last sitting's headline finding was that the pill read only 3 of
  8. That gap is closed, which is the single change behind three of this
  sitting's four newly met criteria.

## Found outside the criteria

1. **The microphone dies on its own, mid dictation.** Three times in this
   sitting, twice during silent runs and once about 12 seconds into an open
   dictation the user confirmed they did nothing to. Reported as a buffer
   under or overrun and handled correctly as `MICROPHONE_UNAVAILABLE`. The
   handling is right; the event should not be happening, and it ends a person's
   dictation part way through. `finding-microphone-dies-on-its-own.md`. To
   `/debug`, and it is the most serious thing this sitting found.
2. **`DEEPGRAM_KEY_REJECTED`'s sentence ends "Nothing was saved.", which is
   false when it arrives mid dictation** about a key that is saved and stays
   saved. The record made precisely this argument when it refused to reuse
   `DEEPGRAM_UNREACHABLE` for a dropped connection.
   `finding-key-rejected-sentence-mid-dictation.md`. To `/architect`.
3. **The pill's close takes 613 to 987 ms**, and lengthened across six
   consecutive toggles. No criterion times the close, so this is not a failure.
   Recorded because a trend is worth a second look before it becomes one.
4. **The transcript line scrolls the opposite way to expectation.** It
   truncates from the left, exactly as `design/registry.md` draws it, so new
   words arrive at the right and the front of the sentence fades away. On
   hearing it work, the user's unprompted reaction was "oh, it's actually
   scrolling the other way". Working as drawn; worth the registry owner
   knowing that the drawn direction surprises a person seeing it first time.
5. **Deepgram substituted words again**, twice in one dictation: "the lazy
   white dog" for "the lazy dog", and "over to lazy" for "over the lazy".
   Accuracy notes for the record's "accuracy is the one thing" line, in the
   same shape as the single substitution the last sitting recorded. Not a
   typing fault: every landed character matched what Deepgram returned.
6. **Earlier sittings' pill measurements were DPI virtualised.** This machine
   is 3840 x 2160 at 200 per cent. A reader that is not per monitor DPI aware
   sees 1920 x 1080 and reports logical pixels, which is why the 2026-08-31
   report recorded "264x76 in this process's coordinate space". Nothing was
   wrong with it, but future measurements should say which space they are in,
   and this sitting's harness was made DPI aware so its rects and its pixels
   agree.

## Not checked, and why

- Everything blocked in the table above, for the reasons given there.
- **AC-13's allowance half.** Exhausting a real Deepgram allowance is not
  forcible here, so `deepgram_no_allowance` and its Open Deepgram console next
  step have still never been seen.
- **`DEEPGRAM_KEY_NOT_ALLOWED`, the fifth key kind.** An invalid key produced
  `DEEPGRAM_KEY_REJECTED`, not this. Its live trigger remains the first spike
  in the record's Still open, and its mapping remains the documented guess the
  record already calls it.
- **Both original spikes in Still open.** The deliberately narrow key at the
  stream, and how widely `IsPassword` reaches. Both need the user, both
  unchanged.
- **The password cannot-tell fallback.** Could not be forced on this machine;
  the rule is unit tested.
- **A double tap arriving during the 2 second hold.** The code holds one
  thread so it lands after, and that is taken on the code's word.
- **The four microphone error kinds live on this build.** Two stand proved from
  2026-08-30, two remain unforcible (`AC-15-mic-error-kinds-standing.md`).
  Today's device death classified to the catch-all, not to blocked-by-Windows.
- **The sound `CannotTell` path.** No natural example of such a path form
  exists on this machine.

## What this sitting changed on the machine, and put back

Recorded because a verify run should leave no trace, and this one had to make
three changes to force real failures. All three were approved by the user and
all three were restored and verified.

1. The dev instance running from 10:27 was stopped, and a build made by this
   sitting was launched in its place.
2. `deepgram_credential.credential_target` was pointed at a throwaway
   credential holding an invalid key for exactly one dictation, then restored
   to `deepgram:user_3IXiPaRho7Jkw48Yq8MMHCwCLyC` and the throwaway entry
   deleted. The real key entry was never read, moved or overwritten. Both
   verified afterwards.
3. `DeviceConnect\.Current` was pointed at a missing file for the sound test,
   then restored to `C:\WINDOWS\media\Windows Hardware Insert.wav`. Verified
   afterwards.

A final dictation with the real key was run at the end to prove the machine was
left working: three phrases landed, character perfect, correct joins.

## Heavy weight sign off

Not sought. Record 0002 is heavy and needs the user's own written approval in
this report, but that belongs to a pass. Thirteen criteria could not be
exercised and four specced commands do not exist. The natural moment is after
milestone 5, when all 33 can be judged together.

## Hand-off

The plan row's verify boxes stay unticked, because a tick means a pass.

Next, in order:

1. `/debug` on the microphone dying by itself. It is the only thing found this
   sitting that damages the core promise, and it needs reproducing before
   anything else is built on top.
2. `/architect` on the rejected-key sentence arriving mid dictation. Small, and
   the record owns every sentence in this feature.
3. **Milestone 5**, history and settings, which unblocks six criteria and the
   four missing commands in one go.
4. A second monitor, whenever one exists, for the four screen clauses.
5. `/test` for the newly proved behaviour, so the pill's eight listeners, the
   2 second hold and the three mid dictation endings are held by tests and not
   only by this sitting's frames.
6. The two spikes, whenever the user has time for them.

One cheap thing worth doing before milestone 5: AC-23's first-open placement is
blocked only because a dragged position is stored. Clearing `pill_x` and
`pill_y` for the account would let the bottom-centre placement be observed
directly. It is one row, and it was not done here because the user's approval
covered a different write.
