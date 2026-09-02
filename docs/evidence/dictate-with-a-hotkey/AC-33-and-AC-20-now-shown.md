# AC-33 and AC-20's showing half: the pill speaks. 2026-09-02

The two criteria that failed on 2026-08-31 were both failing on one cause: the
pill rendered no text at all. Both are now met, photographed.

The instrument matters here, so it is named first. Nothing about the transcript
is ever logged, by design, so no log line could ever prove this. The only
instrument that can see it is a photograph of the pill's own screen rectangle,
taken while a person speaks. Every frame below was captured that way, at the
window rect read from Windows by handle, by a harness made per monitor DPI
aware so its rects and its pixels are in the same device pixels the app works
in.

## AC-33: the grey unfinished wording, met

`frames/AC-33-grey-interim-tail.png`, captured 5,539 ms into a live dictation:

- **Final ink**: "The quick brown fox jumps"
- **Grey interim tail, with its dotted rule beneath**: "over the"
- The 18 meter bars moving with the voice, which is also AC-1's waveform half
- The caret at the end of the interim tail

`frames/AC-33-final-ink-line.png`, 8,313 ms into the same dictation, shows the
line fully hardened: "The quick brown fox jumps over the lazy dog." in ink,
with the tail gone. The line visibly hardens left to right, which is what the
criterion asks for.

`frames/AC-33-line-truncates-from-the-left.png` shows the drawn overflow
behaviour once a sentence is longer than the line: the front of the sentence
fades out at the left edge. That is `design/registry.md`'s truncation, working.

### The protective halves, still holding

AC-33 also says the unfinished words appear on the pill and nowhere else.

- **Never typed.** In the same run the landed text was read out of the receiver
  by `WM_GETTEXT` and compared as code points. It held only finalised wording:
  131 UTF-16 units, first code point 84 (`T`), a single 32 at each phrase join
  and no leading space. Grey wording never reached it.
- **Never stored.** The `dictation` table held 0 rows before and after every
  dictation in this sitting, read by SQL. No transcript, interim or final, was
  written anywhere.
- **Never logged.** `grep eprintln src-tauri/src/dictate/transcribe.rs` returns
  four lines, none of which carries a phrase.

`frames/AC-33-interim-never-typed-into-password-field.png` is the sharpest
version of this: a pill carrying four grey interim phrases while the focused
field was a password box, with nothing typed anywhere.

## AC-20's showing half, met

`frames/AC-20-refusal-first-frame-4648ms.png`. A real password edit control
took focus, one sentence was spoken, and the pill showed:

```text
BLOCKED_PASSWORD_FIELD
EchoScribe will not type into a password field.
```

A mono code over one sentence, in the registry's drawn Error pill shape with
its warning accent down the left edge, and **no action at all**, which is what
the ninth amendment fixes and what AC-30 requires of a pill.

### The refusing half, re-proved on this build

- stderr printed `dictate: dictation stopped: BLOCKED_PASSWORD_FIELD`.
- The pill closed itself 6,829 ms after opening, with no closing tap.
- The foreground window was read by handle before and after: unchanged, the
  same password box. **The EchoScribe window did not come forward**, which is
  the one ending the record says must not.
- The focused control's text length was 0 before and 0 after. A password edit
  refuses `WM_GETTEXT` across processes by design, so its length is the one
  readable fact about it, and zero is what AC-20 needs.

## What changed in the code, for the record

The last sitting's finding named the cause: `src/dictate/pill.js` registered
three listeners out of eight events. It now registers all eight, and the
interface side of the event surface went from 3 of 8 read to 8 of 8. That is
the whole of the fix these two criteria needed.
