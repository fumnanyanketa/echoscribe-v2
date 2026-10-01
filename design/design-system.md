# EchoScribe design system

Locked on 2026-08-27 by `/canvas design`, from the comp in
`design/references/`. From this point every screen matches this exactly.
Anything else is drift.

This document holds character, mandate and rules. **It holds no values.**
Every colour, size, space, radius and shadow lives in
[src/styles.css](../src/styles.css) as a token. Two copies of a value drift
apart within a month, so there is only ever one.

---

## The character

> A dark instrument laid on a bright page. Graphite where you navigate,
> white where you read, one violet channel that only ever means the
> microphone is open.

Every decision below comes out of that sentence.

## Why the pill is dark

The pill floats over somebody else's application, usually a white document.
Making it the darkest thing on the screen means it reads as a physical
object set down on the page rather than a panel grafted into it, and it
stays legible without borders, outlines or blur tricks that fight whatever
is underneath.

## The violet rule

Violet means the microphone is open. Nothing else. It is not a brand
colour, not a hover tint, not a way to make a button look important. If
violet appears anywhere that is not about a live microphone, that is a bug.

**Amended 2026-10-01, by the user.** The comp itself paints violet on a
fixed set of statics, and the user chose the comp over the stricter reading
of this rule: the brand dot, the active nav item's lead bar, the account
initials disc, the signed-in dot, the key field's edge and dot on the
first-run screen, and that screen's step bars. Those comp-drawn statics are
the whole exception. Violet still never arrives as a hover tint, a new
decoration, or anything the comp does not draw, and the live channel (ring,
meter, caret) still only ever means an open microphone.

The consequence is that the "off" state is not a grey pill. **Off is the
pill absent from the screen entirely**, so there is nothing idle-looking to
mistake for something listening.

## The mandate

A screen is not done until all of these are true. Any one of them false is
disqualifying, not a nitpick.

1. Every colour, size, space, radius and shadow comes from a token in
   `src/styles.css`, or is the comp's own value stated with a comment naming
   the comp element it copies. **Amended 2026-10-01, by the user**: the comp
   is the authority pixel for pixel, and its values do not all sit on one
   scale, so a component may carry a comp literal where the scale has no
   step for it. A literal with no comp behind it is still a bug.
2. Every text and background pair passes AA, and body copy passes AAA.
   Prove it: `python design/check-contrast.py`. Add the pair to that file
   when you introduce one.
3. Violet appears only where the microphone is open.
4. The screen has an empty state, an error state and a loading or waiting
   state, drawn, not assumed. The empty state is the first thing a real
   user sees.
5. Every state that matters is carried by **two independent signals**, one
   of which is words. A colour alone never carries meaning, because roughly
   one man in twelve will not see it. "Mic open" is a breathing violet ring
   **and** the words MIC OPEN.
6. Every control reachable by keyboard shows the focus ring, and the tab
   order matches the reading order.
7. A boundary that tells you where a control is uses the control border
   token, not the decorative one. Decorative separators are exempt from
   contrast; control edges are not.
8. Nothing inside the app window casts a heavier shadow than the pill. The
   pill is the top of the elevation order, always.
9. No text is truncated without the full text being reachable some other
   way.
10. Motion respects `prefers-reduced-motion`. The reduced path still has to
    convey the state, in words.

## Composition

**The shell is two-tone.** A persistent dark rail on the left where you
navigate, a white surface on the right where you read. Not tabs. History is
a working surface you return to with a query in your head, and Settings is
the section that will keep growing, so the rail absorbs new sections
without re-laying-out a tab bar. Account is pinned to the bottom of the
rail because it is visited twice a year.

**The pill is a measuring instrument, not a chat window.** It is the
smallest thing that can honestly report what the microphone and the network
are doing. It never grows, it never scrolls, and it never becomes a place
you read your own history.

**Type.** Manrope for everything a person reads. JetBrains Mono for things
a machine produced or a person must type back exactly: state codes, key
fragments, hotkey names, language tags. Mono is a signal that something is
literal, so do not use it for decoration.

**Fonts are bundled**, in `src/assets/fonts/`. The interface never calls
Google Fonts or any other host at runtime. Manrope carries no CJK glyphs,
so the sans stack falls through to the system Japanese, Korean and Chinese
faces. That fallback is real and load-bearing: the language list and the
history both show non-Latin text.

## Component rules

### The pill

- One line of text, always. The **locked front of the sentence is ink; the
  still-settling tail is grey with a dotted rule under it.** The dotted rule
  is the promise that the text can still change, and it disappears the
  instant the text is final, so the line visibly hardens left to right.
- **Truncate from the left with a soft fade.** Never scroll, never grow.
  The tail is the only part still in question, so the tail keeps the right
  edge. The real text is already in the document, and history has the rest.
- The level meter is **amplitude only, never a spectrogram**. It means one
  thing: sound is arriving. Flat at silence. It is not a quality reading and
  must never be dressed up as one.
- The drag grip is grabbable from any edge, with a hit area no smaller than
  44px and a grab cursor.
- After 20 seconds a small elapsed and word count appears, so a long
  dictation feels accounted for.

### Error pills

Error states keep **the exact geometry of the working pill**. Only the
violet channel is swapped for one coloured edge. Each carries, in order: a
mono code and one sentence of plain cause, and no action at all. The pill
is a sign, never a control, and nothing but its grip answers the mouse.
Every action a person can take lives in the EchoScribe window instead
(record 0002, amended 2026-08-30, AC-30).

When a connection drops, the pill states **what it managed to keep**.
Silent loss is the worst failure a dictation tool can have, and a user who
does not know what survived cannot recover.

### Empty states

The empty state is a screen in its own right, not a blank rectangle. It
says what has not happened yet, shows the hotkey as keys, explains in one
short paragraph where the words will go and that they stay on this machine,
and offers exactly one thing to do next plus one way to change the setup.

### Secrets

The Deepgram key is shown as a masked field with a short trailing fragment
and the words that say it is stored locally and never leaves this machine.
Never render the key in full after it is saved. When it is missing, the
blocker states plainly that dictation is off until it is set, and links to
where a key comes from.

## What this system deliberately does not have

Naming these stops them being reinvented one screen at a time.

- No light and dark theme toggle. The rail, the pill and the first-run
  window are always dark; the reading surface is always white. That is the
  design, so `prefers-color-scheme` is not honoured.
- No responsive breakpoints. These are fixed desktop surfaces.
- No decorative illustration, no mascot, no gradients, no icon set beyond
  the few glyphs the comp uses. The product's character comes from
  restraint and from the pill, not from artwork.
- No borders where a background change will do.

## Assets

`design/assets/` is empty on purpose. This direction carries no
illustration. If that changes, the signature element gets generated first
and everything else references it.

## Pointers

| What | Where |
|---|---|
| All token values | [src/styles.css](../src/styles.css) |
| Bundled font faces | `src/assets/fonts/` |
| Contrast audit, runnable | [design/check-contrast.py](check-contrast.py) |
| Approved components | [design/registry.md](registry.md) |
| The original comp | `design/references/` |
