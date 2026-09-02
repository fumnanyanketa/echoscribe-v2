# AC-7: focus moved mid dictation, the words followed. 2026-08-31

Build: commit `3b4fb62`. One continuous dictation; the focus move was a real
mouse click on the scratch edit box, fired by the harness twelve seconds in,
with the pill confirmed still up immediately after.

## The scored run

Both receivers byte-read after the close (`landed-notepad-d4.txt`,
`landed-editbox-d4.txt`):

- Notepad, focused first: 108 bytes, the full spoken script once, character
  perfect, nothing after the focus move.
- The scratch edit box, focused after the move: 109 bytes, one leading space
  then the full spoken script once, character perfect.

The user spoke the script twice, once per window, and the split falls exactly
at the click: no interleaving, no phrase in the wrong window, nothing lost.
The leading space in the second window is the record's own join rule doing
what it says: a single space is typed before every finalised phrase after the
dictation's first, and the phrase after the move was not the first. Worth
knowing, not a defect: a mid dictation focus move means the new window's text
begins with one space.

## The two discarded attempts, kept for honesty

- Attempt 1: the user split one sentence per window on ambiguous
  instructions; each landing matched what was spoken, nothing lost, but the
  planned comparison could not be scored. Discarded.
- Attempt 2: the dictation opened before the user was ready and transcribed
  their setup words ("Let's try that one more time...") into both windows,
  split at the focus move, again correctly. Discarded as a scored run, but
  both discarded attempts independently show the same mechanics: phrases land
  wholly in whichever window held focus when they were finalised, and the
  pill survives the move.

## Also seen

In attempt 2, wording still unfinalised when the closing tap landed was
discarded and never typed, which is AC-33's protective half and AC-4's only
final wording rule, both holding under an abrupt stop.
