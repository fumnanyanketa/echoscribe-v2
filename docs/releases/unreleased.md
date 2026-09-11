# EchoScribe, the next release

_Draft. Not released, and not ready to be._

This is the write-up of everything built since 3 September 2026. It has no
version number because no version can be cut yet, and it has no release
date for the same reason.

**Please read this part first.** Three of the five things described below
are finished as far as the code goes, and do not work when you run them.
Your custom words will not save. Your language choice will not save. Your
history will not open, and history is the first screen you see after
signing in. The cause is known, it is a single mistake, and it is being
fixed. Nothing below is written as though it already works, and nothing
below has been tested by a person driving the real app from end to end.

If you are reading this to decide whether to install, the answer today is
no.

---

## What was meant to be new

### Settings, at last

EchoScribe now has a proper Settings area with its own window, reached from
a bar down the left-hand side.

Under **Dictation** you choose which key starts you talking. There are two
choices, you pick one, and there is a switch for the small sounds that tell
you the microphone has opened and closed. Each choice has a line under it
explaining what it does.

Under **Transcription** you can see the Deepgram key you saved, without ever
seeing the key itself again. EchoScribe shows a row of dots and the last four
characters, so you can tell which key it is without it being readable over
your shoulder. The number of dots is always the same and tells you nothing
about how long your key is, deliberately. You can replace the key, and if
the new one turns out to be no good, your old one keeps working and nothing
is lost. You can also remove it, and EchoScribe asks you once first, on the
row itself, with the Cancel button already selected so a stray second press
backs out rather than deletes.

**This part is not affected by the fault described at the top.** Like
everything here, though, it has not been checked by a person using it.

### Teach it your words

There is now a place to add the words EchoScribe keeps getting wrong: names,
technical terms, anything unusual to you. They are sent along with your voice
every time you dictate, so it knows to expect them.

There is a limit on how much you can add, counted in characters rather than
words. That is on purpose: a word count would have been generous in English
and would have quietly broken dictation in Chinese.

**This does not work yet.** Nothing you add will save.

### Speak in your language

You can choose the language you dictate in, from 64 of them, with a search
box so you are not scrolling. There is a "multilingual" option for switching
between languages mid-sentence. English stays the default, so if you never
open this, nothing about EchoScribe changes for you.

**This does not work yet.** Your choice will not save, and it will always
report English.

### See what you have said before

Everything you dictate is now kept, on your own machine, and there is a
screen to read it back. You can search it. Each entry shows how many
characters it was, and a Copy button puts that text on your clipboard and
nothing else with it.

Nothing you say is stored anywhere but your own computer.

**This does not work yet.** The screen opens and tells you your history
could not be read.

### A clock on the dictation pill

The small pill that appears while you are talking now grows a little chip
after twenty seconds, showing how long you have been going and how much you
have said. It appears in space the pill was already holding, so nothing
jumps or resizes when it arrives.

---

## Fixes

Two faults, both with the same symptom: EchoScribe vanished from your screen
but kept running.

- **Fixed a problem where signing out left EchoScribe with no window at
  all.** The app kept running in the background, still listening for your
  dictation key, with no way to get back to it except ending it through Task
  Manager.
- **Fixed a problem where closing the window with the X, while you were on
  the sign-in screen or the key setup screen, did the same thing.** The
  window closed and the app stayed running, invisibly.

Both are now closed, and the app has automatic checks in place to stop
either coming back.

---

## What we are not telling you

Being straight about the limits of this write-up.

- **Nothing here has been checked by a person using it.** Every part of this
  was read as code and tested piece by piece. No one has yet sat down, pressed
  the key, spoken, and watched all of it work together. Until that happens,
  treat everything above as "should", not "does".
- **Three of the five features are known not to work**, and they are named
  above rather than buried here.
- **A further eleven problems are open**, found by a review on 11 September.
  Most are about timing: two things happening at once where nobody had
  decided which should win. None will be left unfixed.

---

## Upgrade notes

There is nothing to upgrade to. When there is, this section will say what to
do.
