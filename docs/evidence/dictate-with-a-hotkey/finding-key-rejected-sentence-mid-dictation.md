# Finding: "Nothing was saved." is false mid dictation. 2026-09-02

Found while proving AC-13's rejected half live for the first time. Not a code
defect: a wording decision the record has already made once, one door along,
and has not yet applied here.

## What was seen

A key Deepgram refuses produced, on the pill and again on the window that
followed:

```text
DEEPGRAM_KEY_REJECTED
Deepgram did not accept this key. Nothing was saved.
```

Photographed in `frames/AC-13-key-rejected-on-the-pill.png` and
`frames/AC-13-window-forward-next-step.png`.

## Why the sentence is wrong here

That sentence is right at the setup screen, where a person has just pasted a
key: Deepgram said no, and nothing was written. It was designed for that
screen, by the sixth amendment.

Arriving mid dictation it says something false. The person's key **is** saved,
has been saved for days, and is still saved after the refusal: the row and the
credential entry are both untouched. What actually happened is that a key which
used to work has stopped being accepted. Telling that person "Nothing was
saved." invites them to conclude their key was never stored, and sends them
looking for a problem that is not there.

## The record has already made this exact argument

When the eleventh set of decisions settled the connection dropping, it weighed
reusing `DEEPGRAM_UNREACHABLE` and refused, in these words:

> its sentence ends "the key was not checked. Nothing was saved.", which is
> false about a connection that dropped mid sentence, and changing that
> sentence would break it on the setup screen where it is already right.

That is the same sentence fragment, the same falseness, and the same reason. A
new kind was minted there rather than bending a sentence that was right
somewhere else. `DEEPGRAM_KEY_REJECTED` arriving mid dictation has the same
shape and did not get the same treatment, because until this sitting it had
never been seen arriving mid dictation at all.

## What this is not

Not a defect in `deepgram_key.rs`, which maps Deepgram's refusal correctly. Not
a failure of AC-13's rejected half, which is otherwise met: the code, the
sentence and the matching next step all arrived, and the next step is right.
The person does land on the screen where a key gets pasted, which is what they
need to do.

## Routing

`/architect`, against record 0002. It is a decision about wording, and the
record owns every sentence in this feature. The likely shapes are a second
sentence for the mid dictation arrival, or a kind of its own, and choosing
between them is not this skill's call.
