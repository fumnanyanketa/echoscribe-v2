# The 2 second hold, measured on three different endings. 2026-09-02

The eleventh amendment settled one value that was named nowhere: how long the
pill holds its last words before it closes. Two seconds, measured from the
words being shown, for every ending that puts words on the pill. It was
unbuilt then. It is built now, and this is it timed.

## How it was measured

The pill was photographed on a loop, each frame stamped with the milliseconds
since the opening tap, and the window was polled until Windows reported it no
longer visible. The hold is the gap between the first frame carrying the ending
and the moment the window went invisible. Frame spacing sets the precision, so
each ending gives a range rather than a single number: the ending could have
appeared at any point after the last frame that did not carry it.

## Three endings

| Ending | Last frame without it | First frame with it | Pill invisible | Hold |
|---|---|---|---|---|
| `BLOCKED_PASSWORD_FIELD` | 4,402 ms | 4,648 ms | 6,829 ms | 2,181 to 2,427 ms |
| `MIC STOPPED` (device died) | 11,777 ms | 12,171 ms | 14,212 ms | 2,041 to 2,435 ms |
| `DEEPGRAM_CONNECTION_LOST` | 34,657 ms | 34,893 ms | 37,075 ms | 2,182 to 2,418 ms |

All three land on 2 seconds plus the work that follows the sleep: closing the
window, reading the sound setting, and playing the closing sound. Nothing here
suggests a hold that is off by a factor, and nothing suggests the old
behaviour, where the pill hid in the same instant the words were emitted and
the live sitting of 2026-08-31 saw the password refusal "just disappear".

A fourth ending, `DEEPGRAM_KEY_REJECTED`, was also captured but is not timed
here: it arrives as the stream opens rather than mid dictation, so the frame
before it carries nothing to compare against. Its pill closed 3,330 ms after
opening, which is consistent with the same hold.

## What rides on it

AC-20 and AC-30 both need a person to be able to read what the pill says
before it goes. On the evidence above, they can.

The blocking sleep is deliberate, per the code's own comment: every open and
close goes through one thread, so nothing can race the pill while it holds, and
a double tap during the hold lands after it. That last property was not
separately tested, because forcing a tap inside a 2 second window that only
opens after a live refusal needs a voice and a stopwatch in the same hand. It
is named here as untested rather than assumed.
