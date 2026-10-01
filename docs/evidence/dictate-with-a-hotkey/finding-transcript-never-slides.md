# Finding: the transcript line did not slide. 2026-10-01

The user noticed during a real dictation that the pill's one line of text
stayed on the opening words instead of following the newest ones.

## Reproduced

A 229 character sentence was dictated via text to speech while the pill
window was photographed every two seconds. Frames at 8 s and 14 s show the
same opening words, clipped at the right edge, with no fade. The newest
words were the hidden ones for the whole dictation. Frames:
`frames/AC-33-stuck-before-fix-8289ms.png` and
`frames/AC-33-stuck-before-fix-14429ms.png`. This layout is unchanged since
2026-08-31, so the slide had not worked on any build.

## Cause

Two halves, in `src/dictate/pill.css` and `src/dictate/pill.js`.

1. The inner transcript row was an ordinary flex item, free to shrink down
   to its `min-width: 100%` floor. It therefore always measured exactly as
   wide as the outer box. The outer box saw no overflow, its `flex-end`
   alignment had nothing to align, and the text spilled out of the inner
   row to the right, where the outer box clipped it.
2. The fade check compared the outer box's `scrollWidth` to its own
   `clientWidth`. Even with half one fixed, that stays false: the intended
   overflow hangs out on the left side, and `scrollWidth` does not count
   left side overflow in left to right writing.

## Fix

`flex: none` on the inner row, so it grows to its content and the outer
box's `flex-end` pins its right edge; the fade check now measures the inner
row's width against the outer box. Guarded by
`the_transcript_line_can_actually_slide_left` in
`src-tauri/src/dictate/pill_window.rs`, which fails the build if either
half is simplified away.

## Re-proved

The same sentence, the same camera. At 8 s the line is mid sentence with
the front fading off the left edge; at 14 s the sentence's final words sit
at the right edge with the caret. Frames:
`frames/AC-33-sliding-after-fix-8294ms.png` and
`frames/AC-33-sliding-after-fix-14437ms.png`. 345 tests pass, clippy clean.
