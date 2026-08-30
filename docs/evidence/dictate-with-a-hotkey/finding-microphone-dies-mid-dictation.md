# Finding: the microphone can die mid dictation and the pill keeps saying MIC OPEN

**Observed live, 2026-08-30**, outside the acceptance criteria. Build: commit
`5ffc7f2`, clean tree.

## What was done

With the pill open and the microphone genuinely capturing, the user turned
**Let desktop apps access your microphone** OFF in Windows Settings, while the
pill was on screen.

## What happened

- The pill **stayed up, still reading `MIC OPEN`**.
- The meter bars **stopped moving**. Speaking did nothing.
- The EchoScribe window did nothing.

App stderr:

```
dictate: the microphone stream reported an error: Access is denied. (os error -2147024891)
```

## Why it matters

`microphone.rs` passes cpal an error callback that does one thing:

```rust
|e| eprintln!("dictate: the microphone stream reported an error: {e}"),
```

A stream that dies after it opened is printed to stderr and nothing else. The
pill goes on claiming the microphone is open, and because the meter is designed
to read flat at silence, a dead microphone is visually identical to a quiet
room. The person has no way to tell.

Record 0002 defers AC-30's mid-dictation half to milestone 4 in as many words,
so this is **not scored as a failure of AC-30 here**. It is recorded because it
turns a theoretical case into an observed one, and because the record's own
framing of a stale error screen applies in reverse: a pill saying `MIC OPEN`
over a dead microphone is a lie on screen, of exactly the kind the record
refuses elsewhere.

## What milestone 4 has to take on

The record hands milestone 4 the mid-dictation error path for Deepgram errors:
connection lost, key rejected, allowance spent. This is a fourth source it does
not currently name: the audio device itself failing after a successful open.
It wants the same ending, the pill saying so in words and closing.

## Route

`/architect`, because the record's AC-30 mid-dictation list does not currently
include a device that dies after opening, and then `/develop` in milestone 4.
