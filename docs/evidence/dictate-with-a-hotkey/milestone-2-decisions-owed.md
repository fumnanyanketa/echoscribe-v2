# Decisions owed to record 0002 after milestone 2

**Written by `/develop` on 2026-08-29,** at the end of building milestone 2,
"the microphone and the waveform". `/develop` does not edit decision records,
so the three items below are written here instead. Two of them are open and
need `/architect`. One is closed and only needs writing down.

The gate stopped on all three before any code was written. The user chose what
to do about each, and the build followed those choices.

---

## 1. What "no speech" means before Deepgram exists. OPEN.

**AC-8:** "After 30 seconds with no speech, or after 5 minutes of dictation,
the microphone closes on its own and the pill disappears."

**What the record says the source is.** Value sourcing, "30 second silence, 5
minute cap": *"This record. Fixed, not settings. Silence means no final
wording from Deepgram in that period."*

**Why that is a problem here.** Deepgram arrives in milestone 4. In milestone 2
nothing on the machine can say whether a person spoke. Loudness says sound is
arriving, which a desk fan, a keyboard and a conversation in the next room also
do.

**What was built.** The 5 minute cap, live and working. The silence cap is
built, tested and **unarmed**: `limits.rs` has both, and the milestone 2
constructor is `Deadlines::without_deepgram`, which never fires the silence
half. The armed behaviour has four passing tests already, so the day it is
switched on it is not new code.

**What milestone 4 has to do.** Two lines. Build `Deadlines` with
`watching_for_silence` instead of `without_deepgram`, and call `speech_heard`
on every result Deepgram marks final.

**What is owed.** Nothing, if milestone 4 arms it as the record already
describes. This is here so that AC-8 is not read as fully met after milestone
2. Half of it is not.

---

## 2. Where a person reads AC-15's message. OPEN, and needs `/architect`.

**AC-15:** "If the microphone is unavailable, the pill never appears and the
message names the actual cause, with a link straight to the Windows microphone
privacy setting when access is blocked."

**The conflict.** Two things the project has already decided cannot both hold:

- `design/registry.md` puts errors in the pill: *"Error pill | replaces pill
  body | Same geometry. Mono code, one sentence, exactly one action."*
- Record 0002's amendment of 2026-08-29 makes the pill unclickable everywhere
  except its grip, and `pill_mouse.rs` implements that by taking the pill's web
  view out of the mouse path entirely. The pill therefore has no surface that
  can carry a link, and giving it one means reopening the fix for the AC-27
  focus defect.

There is no third place named. The main EchoScribe window exists and takes
clicks, but no drawn comp puts an error there, and the registry says errors
live in the pill.

**What was built.** The half that is fully sourced:

- The microphone is opened **before** the pill. If it will not open, the pill
  never appears and neither sound plays. Both of those mean "the microphone is
  on", so neither may happen when it is not.
- The cause is classified into record 0002's three named kinds plus one
  honest catch-all, and goes out on `dictation:error` with a kind and a
  sentence. Nothing is swallowed.

**What was not built.** Anywhere for a person to read it, and the link to the
Windows privacy setting. Today the message reaches the event and the dev
console and no further.

**What is owed.** `/architect` decides where the message is read given that the
pill cannot be clicked. The obvious candidates, none of them free:

| Option | Cost |
|---|---|
| Bring the main window forward with the error in it | No drawn comp for it, and it disagrees with the registry putting errors in the pill |
| Give the pill one clickable area besides the grip | Reopens the AC-27 fix, which is a live defect's fix and currently has no automated test at its most important layer |
| A pill with no action, and the instruction in words only | Keeps the pill inert, but the registry says an error pill has exactly one action, and AC-15 asks for a link |
| A Windows notification | AGENTS.md puts notifications out of scope |

The wording of the four sentences in `microphone.rs::MicError::message` is
also `/architect`'s to confirm, not `/develop`'s to have chosen.

---

## 3. Which microphone is used. CLOSED, and only needs writing down.

Record 0002 names no microphone anywhere: no device column in the data model,
no device field in `dictation_setting`, no device command in the interface
surface, and no acceptance criterion about choosing one. So the microphone
Windows is already set to use is the only one this feature has, and that is
what `microphone.rs` opens.

`design/registry.md` does draw a "Device row | dictation settings | Device
name, connection badge, live input level", which suggests a device picker
exists somewhere later. Nothing in record 0002 describes it, so it is not
milestone 2's and probably belongs with milestone 5's settings screen or a
record of its own.

---

## A fourth thing, smaller: one error kind the record does not name

Record 0002's interface surface names three microphone errors: blocked by
Windows, in use by another app, no microphone found. cpal can also fail in ways
that are none of those, for example an unsupported stream configuration.

`microphone.rs` maps those to a fourth kind, `microphone_unavailable`,
deliberately rather than forcing them into one of the three. Telling somebody
another app is using their microphone when it is not sends them looking in the
wrong place, and the record's own reason for making each error read differently
is that each has a different next step. `/architect` should either bless the
fourth kind or say what should happen instead.
