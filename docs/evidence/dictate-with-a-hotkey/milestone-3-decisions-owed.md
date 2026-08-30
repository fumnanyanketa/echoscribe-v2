# Decisions owed to record 0002 before milestone 3

**Written by `/develop` on 2026-08-30,** before any code for milestone 3, "the
Deepgram key", was written. `/develop` does not edit decision records, so the
five items below are written here instead. All five were owed, all five were
answered by the user in the build conversation, and the build followed those
answers exactly.

`/architect` owes record 0002 an amendment carrying these five into the record
itself, most of them into the `## Value sourcing` table. Until that lands, this
file is the only place they are written down, and milestone 3 is not done.

---

## What the gate found

Milestone 3 covers AC-9 to AC-13. The input coverage test enumerated every
value the milestone must produce, compute or display. Seven had a named source
in the record or the registry. Five did not:

```text
The address the get-a-key link opens      AC-9         NO SOURCE NAMED
What request checks a key against         AC-10, AC-11 NO SOURCE NAMED
  Deepgram before anything is saved
Which Deepgram response means rejected,   AC-11, AC-13 NO SOURCE NAMED
  which means allowance ran out
The sentence shown for each key error     AC-11, AC-13 NO SOURCE NAMED
The mono code and matching next step      AC-13        NO SOURCE NAMED
  for each of AC-13's two causes
```

None of these is wiring. Two of them reach outside the machine, which
AGENTS.md's decision rules stop on by name. Two of them are wording, and this
record already set the precedent that wording is not `/develop`'s to choose:
the four microphone sentences were fixed in a table in The decision for exactly
that reason.

---

## 1. The get-a-key address. ANSWERED.

**AC-9:** "A guided setup screen appears instead, explaining what a Deepgram
key is and linking out to get one."

**Why it was owed.** The record set its own precedent here. It made the Windows
microphone privacy page a fixed literal held in Rust and wrote the exact string
into value sourcing, because an address that leaves the machine is a decision
rather than a detail. It did nothing equivalent for Deepgram.
`design/registry.md` names the link and its label but no address.

**Answered:** `https://console.deepgram.com/signup?jump=keys`

Deepgram's signup page, jumping straight to the keys screen, so the person
lands on the exact thing AC-9 sends them for. Held as a fixed literal in Rust,
opened through the system browser by a command that takes nothing, the same
shape as `open_microphone_privacy_settings`. The interface never supplies or
sees it and it is never built from anything.

**Code area:** `src-tauri/src/dictate/deepgram_key.rs`, `DEEPGRAM_SIGNUP_PAGE`.

---

## 2. What request checks a pasted key. ANSWERED.

**AC-10:** "Pasting a valid key into that setup screen checks it against
Deepgram, saves it, and dictation works from then on."

**Why it was owed.** The record names Deepgram's official Rust SDK for
streaming and names nothing at all for validating a key. AC-10 promises that a
checked key means dictation works, so what the check actually is decides
whether that promise is true.

**Answered:** `GET https://api.deepgram.com/v1/auth/token`, with the pasted key
in an `Authorization: Token <key>` header.

This is Deepgram's own documented way to test a key. It proves the key is real
and that Deepgram is reachable, it sends no audio, and it costs no allowance.

**The gap this leaves, named rather than hidden.** A 200 here does not prove
the key can open a streaming socket. A key that lacks the right scope would
pass this check and fail in milestone 4 with a 403. The user was told this
before answering and accepted it. Milestone 4 must therefore treat a scope
failure on the live stream as a real state and not assume a saved key works.

**Checked against Deepgram's live documentation on 2026-08-30**, per AGENTS.md,
because the `deepgram/skills` install is still broken.

**Code area:** `src-tauri/src/dictate/deepgram_key.rs`, `check_with_deepgram`.

---

## 3. Which response means which cause. ANSWERED.

**AC-11, AC-13.** The record's value sourcing says "the error Deepgram returns,
distinguished by its own response", which names a source but cannot be acted on
until item 2 is settled.

**Answered**, from Deepgram's published error table:

| What comes back | The cause it means |
|---|---|
| 2xx | The key is good. Save it. |
| 401 `INVALID_AUTH` | Rejected by Deepgram |
| 402 `ASR_PAYMENT_REQUIRED` | Allowance ran out |
| No answer at all: connect failure, DNS, timeout | Could not reach Deepgram |
| Anything else: 403, 429, 5xx | The honest catch-all |

**The catch-all is a fourth kind, not a fold-in.** Anything Deepgram answers
that is none of the three above becomes `deepgram_check_failed`, which says the
check did not succeed and names no cause, with Try again as its action. This is
the same shape and the same reasoning as the fourth microphone error the record
blessed on 2026-08-30: telling somebody their key is bad when Deepgram was
merely rate limiting them sends them to replace a key that was fine.

**A consequence worth writing down.** 402 is only ever returned for a
transcription request, so `/v1/auth/token` can never produce it. Allowance
exhausted therefore cannot happen at the setup screen. It is a live-stream
state, which is AC-13's own wording, and it arrives in milestone 4. Milestone 3
builds the kind, the code, the sentence and the action; milestone 4 wires the
trigger.

**Code area:** `src-tauri/src/dictate/deepgram_key.rs`, `KeyError::from_status`.

---

## 4. The sentence for each key error. ANSWERED.

**Why it was owed.** `design/registry.md` says "one wording per cause, fixed at
design time" and then gives no wordings. It names two mono codes and no third
or fourth. The record fixed the four microphone sentences in a table for the
same reason.

**Answered.** Four kinds, four codes, four sentences, fixed here and held in
one place in Rust so no screen can invent its own:

| Kind | Code | Sentence |
|---|---|---|
| `deepgram_key_rejected` | `DEEPGRAM_KEY_REJECTED` | Deepgram did not accept this key. Nothing was saved. |
| `deepgram_no_allowance` | `DEEPGRAM_NO_ALLOWANCE` | This key's Deepgram allowance has run out. |
| `deepgram_unreachable` | `DEEPGRAM_UNREACHABLE` | EchoScribe could not reach Deepgram, so the key was not checked. Nothing was saved. |
| `deepgram_check_failed` | `DEEPGRAM_CHECK_FAILED` | The check did not succeed and Deepgram did not say why. Nothing was saved. |

Two of these codes were already drawn in the registry and are unchanged. The
other two are new and are fixed here.

No sentence carries anything read off the pasted key, and no sentence blames
the key when the network was the cause.

**Code area:** `src-tauri/src/dictate/deepgram_key.rs`, `KeyError::code` and
`KeyError::message`.

---

## 5. AC-13's two next steps. ANSWERED.

**AC-13:** "the message says which of the two happened, the key was rejected or
the allowance ran out, with the matching next step for each."

**Why it was owed.** This is the whole substance of AC-13 and nothing named
either step. The registry says the line carries "each with its own sentence and
matching next step" without saying what either is.

**Answered:**

| Cause | Its one action | What that action does |
|---|---|---|
| Rejected | Replace key | Opens the AC-9 setup screen, so a new key can be pasted |
| Allowance ran out | Open Deepgram console | Opens `https://console.deepgram.com` in the system browser, where the person tops up or makes a new key |

The two steps genuinely differ, which is what AC-13 asks for. Making them both
Replace key was considered and rejected: a person whose allowance ran out gets
nowhere by pasting the same key again.

**This approves a second outside address.** `https://console.deepgram.com` is
held as a fixed literal in Rust alongside the signup page, opened by its own
command that takes nothing. The user approved it as part of this answer.

**Code area:** `src-tauri/src/dictate/deepgram_key.rs`, `DEEPGRAM_CONSOLE_PAGE`
and `open_deepgram_console`.

---

## Two scope facts milestone 3 does not close

Neither is an owed decision. Both are here so that AC-12 and AC-13 are not read
as fully met after milestone 3.

**AC-12's display has nowhere to live yet.** "Settings shows only the last four
characters of the saved key." There is no Settings screen: milestone 5 builds
it, and `design/registry.md`'s Deepgram section says in as many words that the
Settings rows for the saved key stay as they are and are not this section's
business. So milestone 3 stores `key_last_four` and returns it from
`get_deepgram_key_info()`, and nothing displays it. The half that is built is
the storing and the never-showing-it-in-full. The half that is not is the row
in Settings. No screen was invented to fill the gap, because the registry
forbids building a component that is not on its list.

**AC-13 cannot be proved live until milestone 4.** It fires when a saved key
stops being accepted mid dictation, and nothing streams yet. The four kinds,
their codes, their sentences and their actions are built and unit tested.
Milestone 4 wires the live stream's 401 and 402 to them and proves it there.
