# Finding: a non-BMP character can be split across two `SendInput` calls

**Found 2026-08-30** by the user, while checking `typing.rs` during the
investigation into garbled dictation. Not the cause of that, and deliberately
**not fixed** at the time it was found. Recorded here because it lands on
**plan row 4, speak in your language**, which is where it should be fixed.

Build: commit `140dbe5` plus the uncommitted crash fix of the same day. Found by
reading, not by observation. It has not been seen happen.

## What is wrong

`send()` in `src-tauri/src/dictate/typing.rs` batches the phrase in chunks of
32 UTF-16 units, and each chunk becomes one `SendInput` call.

A character outside the basic range is two UTF-16 units, a surrogate pair. When
such a character starts at unit 31 of a batch, its two halves fall in different
chunks and go to Windows in two separate calls.

The module comment at the top of the file says such a character "arrives as two
units and is sent as two, in order, which is what Windows expects". That is true
**within** a batch and not **across** one, so the comment currently claims more
than the code delivers.

## Why it matters

Between two `SendInput` calls the input queue is open. Another process can
inject an event, or the receiving application can process what it has so far and
see a lone half of a character. The half is not a character, so the result is
the character being dropped or drawn as a replacement box.

## How likely, honestly

Low today, and worth stating plainly rather than inflating.

- It needs a non-BMP character landing exactly on a 32-unit boundary.
- Deepgram transcribing English does not produce them.
- Most of the languages plan row 4 brings in are still inside the basic range,
  including Cyrillic, Greek, Arabic, Devanagari and everyday CJK. The characters
  at risk are the rarer CJK extensions, some historic scripts, and emoji.

So it is a latent defect with a real mechanism and a narrow trigger, not
something a person is likely to hit this week.

## The shape of the fix

Split the phrase on character boundaries rather than on UTF-16 unit boundaries,
so a surrogate pair can never straddle two calls. The batch then varies slightly
in size, which is fine: `BATCH` exists to keep any one call small, and it does
not need to be exact.

The module comment needs correcting in the same change, so it stops claiming
something the code does not do.

## Owner

**Plan row 4, speak in your language.** Routed there by the user on 2026-08-30,
on the reasoning that row 4 is what widens the range of characters Deepgram can
return and is therefore what makes this reachable.
