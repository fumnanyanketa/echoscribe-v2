# EchoScribe v2

## What is this, in one sentence a stranger would understand
Speak, and it types: a tool that turns your spoken words into typed text
wherever your cursor is, triggered by a hotkey.

## Who uses it, and what are they doing when they open it
Someone already typing a document, email, or message who wants to switch
to speaking for a stretch without breaking flow, and anyone who can't or
doesn't want to type right now — multitasking, dealing with RSI, or on
the move — and needs the text produced some other way. Both are the kind
of person who reaches for this tool, just depending on their mood,
workflow, or situation at the time.

## The single thing it must do well
Accuracy. Transcription has to be right often enough that fixing mistakes
never costs more time than typing would have.

## What the person has afterwards that they didn't have before
Text, without having typed it, and time back — but more than that, the
relief of not typing. Speaking lets you riff off what's in your head, so
you end up saying more, and getting more of the actual context out, than
typing would have. The feeling EchoScribe should give people is the
ability to say all they can say, without the stress of typing.

## Where it runs
Chosen after weighing the trade-off out loud: Windows first, this machine,
to prove it out, with the core (audio capture, hotkey handling, text
injection) built behind a platform boundary so a macOS build can follow
later without a rewrite. Not a permanent Windows-only choice, a sequencing
one — Mac support was explicitly not ruled out, just deferred.

## How you'd know in three months that it worked
You use it daily, unprompted. You reach for it yourself for real writing,
without needing to remind yourself it exists.

## Not building (v2)
1. Payments / billing
2. Mobile version (iOS/Android)
3. Cloud sync across devices
4. Sharing / collaboration features
5. Notifications
6. Voice commands / voice editing ("delete last sentence", punctuation-by-voice)
7. Local/offline transcription model — relies on a transcription service, not on-device
8. Real-time captions for calls or meetings

Confirmed as building, not cut, so excluded from this list on purpose:
sign-in, deep customization (voice profiles / custom vocabulary),
multi-language transcription, transcript history.

## Probably heavy
- Sign-in — auth and account state, flagged as heavy by its nature.
  Voice profiles/custom vocabulary and transcript history are being built
  but were explicitly not flagged as heavy ("neither, just sign-in").