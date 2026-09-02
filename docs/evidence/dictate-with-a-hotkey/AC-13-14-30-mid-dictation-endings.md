# The three mid dictation endings, live for the first time. 2026-09-02

AC-13, AC-14 and AC-30's mid dictation halves had never existed to test before
this sitting. All three were forced live. Two are met; AC-13 is met on one of
its two halves and blocked on the other, and that is scored honestly rather
than rounded.

Every ending has the same shape in the record: the pill says what happened in
words, holds it, closes, the closing sound plays, and the EchoScribe window
comes forward carrying exactly one action, unless there is nothing to do.

## AC-30's mid dictation half: met, on a real device death

Not staged. The microphone died on its own, 12 seconds into a dictation. The
user was asked directly what they had done and answered: nothing.

stderr:

```text
dictate: the microphone stream reported an error: A buffer underrun or overrun occurred.
dictate: the microphone died mid dictation: The microphone stopped working. (Unavailable)
```

- `frames/AC-30-last-frame-mic-open-11777ms.png`: still `MIC OPEN`, meter live.
- `frames/AC-30-mic-stopped-first-frame-12171ms.png`: **`MIC STOPPED`**, in
  warning ink, meter flattened, warning accent down the left edge. The drawn
  MIC STOPPED state, carrying no code and no action, because both belong to the
  window that follows.
- `frames/AC-30-window-forward-one-action.png`: the EchoScribe window showing
  `MICROPHONE_UNAVAILABLE`, "The microphone stopped working.", and one action,
  **Try again**. A short code, one sentence naming the cause, exactly one
  action.

The pill never asked to be clicked, in this or any other frame in this sitting.
Nothing the pill can show carries an action at all.

That the death happened by itself is a finding of its own, recorded separately
in `finding-microphone-dies-on-its-own.md`. It made this criterion easier to
prove and the feature harder to trust.

## AC-14: met

Forced by the user switching Wi-Fi off mid dictation, after a sentence had
already landed.

stderr:

```text
dictate: dictation stopped: DEEPGRAM_CONNECTION_LOST (Dictation stopped because the connection to Deepgram was lost.)
```

- `frames/AC-14-last-frame-transcript-34657ms.png`: the transcript line still
  live, showing the sentence with its front truncated.
- `frames/AC-14-connection-lost-first-frame-34893ms.png`: the pill showing
  `DEEPGRAM_CONNECTION_LOST` over "Dictation stopped because the connection to
  Deepgram was lost.", no action.
- `frames/AC-14-window-forward-try-again.png`: the EchoScribe window forward
  with the same code, the same sentence and one action, **Try again**. The
  foreground window was read by handle and **was** the EchoScribe window, so
  the coming forward is mechanical here and not judged by eye.
- **Words already typed stayed exactly where they were.** Read out of the
  receiver by `WM_GETTEXT` after the ending: `In the country of the blind, a
  one eyed man is the king.`, 56 UTF-16 units, intact. AC-14 requires this
  outright.

**One clause was not separately observable**: that the app tries to reconnect
once. The reconnect attempt is internal, capped at 5 seconds, and produces no
signal of its own. What was observed is its consequence, which is what the
criterion describes visibly. The count of attempts is taken on the code's word
and its unit tests, and that is said here rather than implied.

## AC-13: the rejected half met live, the allowance half blocked

Forced without destroying the real key. A throwaway Credential Manager entry
was created holding a deliberately invalid 40 character key, and
`deepgram_credential.credential_target` was pointed at it for exactly one
dictation, with the user's explicit approval. **Deepgram itself refused the key
over the real network.** The real key entry was never read, moved or
overwritten; the column was restored to
`deepgram:user_3IXiPaRho7Jkw48Yq8MMHCwCLyC` and the throwaway entry deleted,
both verified afterwards.

stderr:

```text
dictate: dictation stopped: DEEPGRAM_KEY_REJECTED (Deepgram did not accept this key. Nothing was saved.)
```

- `frames/AC-13-key-rejected-on-the-pill.png`: the pill showing
  `DEEPGRAM_KEY_REJECTED` over "Deepgram did not accept this key. Nothing was
  saved.", no action. Pill closed 3,330 ms after opening.
- `frames/AC-13-window-forward-next-step.png`: the EchoScribe window forward on
  the **Add your Deepgram key** screen, carrying the error line and the matching
  next step: a box to paste a key into, a Verify button, and "Get a free key
  from Deepgram". The foreground was read by handle and was the EchoScribe
  window.

**Why this is blocked and not met.** AC-13 asks that the message say *which of
the two* happened, the key was rejected or the allowance ran out, *with the
matching next step for each*. One of the two was exercised. Exhausting a real
Deepgram allowance is not something this sitting could force, so the other half
has never been seen and its next step, Open Deepgram console, has never been
reached. Half a criterion is not a criterion.

**Also worth naming**: the invalid key produced `DEEPGRAM_KEY_REJECTED`, not the
fifth kind `DEEPGRAM_KEY_NOT_ALLOWED`. That kind's live trigger is still the
first spike in the record's Still open section, and its mapping in
`deepgram_key.rs` remains the documented guess the record already calls it.
This sitting did not narrow that.

The sentence on the rejected ending reads oddly when it arrives mid dictation.
That is recorded in `finding-key-rejected-sentence-mid-dictation.md`.
