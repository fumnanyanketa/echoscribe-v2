# Decisions owed to record 0002 before milestone 4

**Written by `/develop` on 2026-08-30,** before any code for milestone 4,
"transcription and typing", was written. `/develop` does not edit decision
records, so the items below are written here instead. Six were owed, all six
were answered by the user in the build conversation, and the build follows
those answers exactly.

`/architect` owes record 0002 an amendment carrying these into the record
itself, most of them into the `## Value sourcing` table. Until that lands,
this file is the only place they are written down, and milestone 4 is not
done.

One thing the user asked for before the build was checked first and needed no
action: **the mid dictation device failure is already closed.** Record 0002's
fifth amendment of 2026-08-30 names the microphone dying after it opened,
fixes its ending, its classification and its words, and hands milestone 4 the
proof step. Nothing was owed there.

---

## What the gate found

Milestone 4 covers AC-3, AC-4, AC-7, AC-8's silence half, AC-13's live
trigger, AC-14, AC-20, AC-30's mid dictation half, and AC-32's third clearing
row. The input coverage test enumerated every value the milestone must
produce, compute or display. Six had no named source anywhere in the record,
decision 0001 or `design/registry.md`:

```text
Which Deepgram model, and what is asked      AC-3, AC-4     NO SOURCE NAMED
  of it on the stream
Whether the pill shows unfinished words      AC-3           RECORD AND REGISTRY
                                                            DISAGREE
What a person reads when the key is not      the record's   NO SOURCE NAMED
  allowed to stream                          own "state of
                                             its own"
What a person reads when the connection      AC-14          NO SOURCE NAMED
  is lost
What happens to speech during the one        AC-14          NO SOURCE NAMED
  reconnect attempt
What the pill shows when typing is           AC-20          NO SOURCE NAMED
  refused into a password field
```

A seventh was found and needed no asking, so it is recorded rather than
decided: **the Windows property that names a password field is `IsPassword`
on the focused element.** There is no second candidate, and the record already
settles the rule around it, that a field it cannot read is typed into normally.

---

## The six answers

### 1. The model, and what is asked of it

**`nova-3`.** Deepgram's current general model. Chosen over `nova-2` and over
`flux-general-en`, on three grounds:

- AGENTS.md says the one thing this app must do well is accuracy, and the
  model is the largest single lever on it. Deepgram's own published figure for
  nova-3 is a 54.2% reduction in word error rate on streaming.
- It supports 70+ languages against nova-2's 40+. Plan row 4, "speak in your
  language", inherits whatever is chosen here, and AGENTS.md already flags
  Deepgram's language coverage as unproven. Choosing nova-2 would narrow that
  before it is even looked at.
- `flux-general-en` is built for voice agents and has turn detection built in.
  Turn detection decides when a person has finished speaking in a conversation,
  which is the wrong instinct entirely for somebody dictating a paragraph: it
  would cut them off mid thought.

It costs more per minute than the legacy models, and that is spent from the
person's own Deepgram allowance, never the project's.

Asked of it on the stream: English, fixed here, per the record's Still open
note that plan row 4 owns making it a choice. Punctuation on, because the words
go straight into a document and a person who has to add every full stop by hand
has not saved any time. Interim results on, which point 2 settles.

### 2. Unfinished words on the pill

**Yes. The pill shows them, in grey.**

This was not a free choice between two silences. It was a disagreement between
two documents that both already exist, and one of them had to be wrong:

- `design/registry.md` draws a "Transcript line" on the pill: *"One line. Final
  text ink, interim grey with a dotted rule."*
- Record 0002's interface surface has exactly one event carrying text,
  `dictation:text`, *"with one finalised phrase"*. Nothing could ever feed the
  grey half.

Nothing in `src/dictate/pill.js` or `pill.html` renders text at all today, so
neither had been built and neither had been caught.

The drawn design wins. So milestone 4 adds a second event carrying unfinished
wording to the pill, and `interim_results` is switched on at Deepgram.

**What does not change.** Only wording Deepgram marks as final is ever typed.
AC-4 is untouched, and no text is ever taken back at the cursor. The grey line
is a sign on the pill and never a keystroke.

**What this costs, named rather than hidden.** A partial transcript now crosses
into the interface, which nothing in this feature did before. The data rules
allow it: they forbid a partial transcript being written to any table, any log
or any file, and this is none of those. It is still more transcript surface
than existed yesterday, and the grey line must never be stored, logged, or read
by anything but the pill that draws it.

### 3. A key that is not allowed to stream

**A fifth key kind of its own.** Not folded into `deepgram_key_rejected`.

| Kind | Code | Sentence | The one action |
|---|---|---|---|
| `deepgram_key_not_allowed` | `DEEPGRAM_KEY_NOT_ALLOWED` | This key is not allowed to transcribe live audio. | Open Deepgram console |

The pill shows that code and that sentence, then closes, and the closing sound
plays. The EchoScribe window comes forward carrying the one action, because
there is something the person can do about it.

**Why the console and not Replace key.** A key's permissions are changed in
Deepgram's console and nowhere else. Replace key opens a paste field, which is
the right door only if the person is going to make a whole new key. Ticking a
permission on the key they already have is the shorter road, and it starts in
the same place.

**Why a fifth kind and not a fold in.** The record refuses this fold twice
already, once for the fourth microphone error and once for the fourth key
error, on the same grounds each time: telling somebody their key is bad when it
is fine sends them off to replace something that was never the problem.
Deepgram accepted this key at the setup screen. Saying it did not is false.

**This is the state record 0002 handed milestone 4 in as many words** and never
named, in its Still open bullet on scope. It is now named. It is not yet
*proved*, which is spike 1 below.

### 4. The connection dropping

| Kind | Code | Sentence | The one action |
|---|---|---|---|
| `deepgram_connection_lost` | `DEEPGRAM_CONNECTION_LOST` | Dictation stopped because the connection to Deepgram was lost. | Try again |

Same shape: the pill shows the code and the sentence, then closes with the
closing sound, and the EchoScribe window comes forward with Try again, which
goes through `try_start` like every other way into dictation.

Words already typed stay exactly where they are, which AC-14 requires outright.

`DEEPGRAM_UNREACHABLE` was weighed as a reuse and rejected. Its sentence ends
"the key was not checked. Nothing was saved.", which is false about a
connection that dropped mid sentence, and changing that sentence would break it
on the setup screen where it is already right.

### 5. Speech during the one reconnect attempt

**Held in memory and sent once the connection is back.**

- The microphone stays open for the attempt, so the pill does not flicker and
  the person is not interrupted for a blip that fixed itself.
- Audio is held **in memory only**, never on disk, never in a log. The data
  rules are unchanged: it is captured, sent for transcription and discarded.
- **The attempt is capped at 5 seconds, and the held audio is capped at the
  same 5 seconds.** One cap, used twice, so a long outage can never grow a
  buffer. When the cap is reached the reconnect has failed, and point 4 above
  is what happens next.

Dropping the audio outright was weighed. It is the structurally safer reading
of "audio is transient", and it is simpler. It costs a silent hole in the
middle of a sentence with nothing on screen saying where the words went, on a
connection that recovered fine.

### 6. Typing refused into a password field

| Code | Sentence |
|---|---|
| `BLOCKED_PASSWORD_FIELD` | EchoScribe will not type into a password field. |

The pill shows both, taking the registry's already drawn "Error pill" shape: a
mono code and one sentence, and no action at all. Then dictation stops and the
pill closes.

**The EchoScribe window does not come forward.** AC-30 brings it forward only
when there is something the person can do, and there is nothing to do here
except not dictate into a password box. Bringing a window forward over a
password field would also be the single worst moment in the app to move
somebody's focus.

Dictation stopping rather than merely skipping the keystrokes is not a choice
made here. Record 0002's Risk section already fixes it: *"Type into a password
field. Dictation stops and says it was blocked."*

---

## What is still blocked after all six

Answering these did not unblock the whole build. `/canvas` is owed four things,
and record 0002 says twice that it goes first and that none of this may be
invented during a build:

| Not drawn | Owed by |
|---|---|
| The pill's `MIC STOPPED` state | AC-30 |
| The microphone catch-all's second sentence, "The microphone stopped working." | AC-30 |
| A mid dictation Deepgram error screen on the EchoScribe window | AC-13, AC-14, and point 3 above |
| The two new codes, `DEEPGRAM_KEY_NOT_ALLOWED` and `DEEPGRAM_CONNECTION_LOST` | points 3 and 4 above |

`design/registry.md` also has one row built but never drawn, found on the way
past and not milestone 4's to fix: `key-setup.js` renders an "Open Deepgram
console" action that no registry row describes.

---

## The two spikes record 0002 asked milestone 4 to run

Neither is answered yet, and neither can be answered by `/develop` alone.

### Spike 1: does a key the setup check accepts always open a stream

**Not answerable from documentation.** Checked on 2026-08-30:
`https://developers.deepgram.com/docs/errors` documents HTTP errors only and
has no section on websocket streaming failures at all. It does establish that
Deepgram returns **401 for both an invalid key and a key with insufficient
permissions**, so the status code alone cannot tell those two apart. Whatever
distinguishes them has to be read off the body or the close frame, and only a
live run will show which.

Needs from the user: a Deepgram key deliberately made **without** streaming
permission, created in their own console. Then point it at the stream and
record what came back, verbatim.

Until that runs, `deepgram_key_not_allowed` is a kind with a decided wording
and an undecided trigger.

### Spike 2: how widely a password field is really detected

The property is settled, `IsPassword`. The coverage is not. Needs a real
machine and a list of real applications: Chrome, Edge, Firefox, a Windows
credential prompt, a password manager, and at least one older desktop app.

The record already fixes the rule for when it cannot tell, which is that the
app types normally, so this spike changes no behaviour. It measures how often
AC-20's promise actually holds, which is worth knowing before anybody trusts it.
