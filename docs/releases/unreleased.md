# EchoScribe, the next release

_Draft for v0.1.1. Not released yet. Rewritten 2026-10-19; the earlier draft
on this page described what became v0.1.0 and is now in CHANGELOG.md._

If you installed EchoScribe from the first download, this is what changes
when you install the next one. Nothing here needs you to do anything: your
history, your words and your key stay where they are.

---

## Fixes

- **Fixed a problem where the key setup screen got stuck on "Checking
  key".** You pasted your Deepgram key, pressed Verify, the key was accepted
  and saved, and the screen never moved on. The close button did nothing
  and the only way out was Task Manager. Every first key save in v0.1.0 did
  this. On the next launch EchoScribe found the saved key and worked, which
  is why it looked like a one-off. It was not. Found on 13 October, fixed,
  and confirmed on 19 October by removing a key and pasting it back: the
  main window opened as it should.
- **Fixed the app's name on the sign-in screen drawing smaller than
  designed.** Cosmetic. It read at 23 pixels instead of 28.

## New

- **A small diagnostic file, for when something goes wrong.** EchoScribe now
  keeps a file called `echoscribe.log` in the same folder as its database.
  Each line is the time and a plain sentence about something that went
  wrong inside the app, such as the microphone not opening. If you ever
  write to hello@ about a problem, attaching that file lets us read what
  happened instead of guessing. It never holds anything you dictated, any
  audio, your Deepgram key or your sign-in. It is capped at one megabyte
  with one previous copy, and it never leaves your computer unless you send
  it. Nothing about errors or usage is sent anywhere, by design, and the
  privacy policy will say so in as many words once its new paragraph is
  approved.

## Changed

- **The key setup screen is calmer.** One sentence, the three steps behind a
  "Show the three steps" link, a quieter Verify button, and a fainter
  background pattern.

---

## What we are not telling you

- **The diagnostic file has been seen working on a development build only.**
  It has not yet been checked on an installed copy, and the one line it
  writes when the app crashes has not been provoked on purpose yet.
- **The installer is still not code signed.** Windows will show the blue
  SmartScreen warning. Press "More info", then "Run anyway".

---

## Upgrade notes

Run the new installer over the old one. Nothing else.
