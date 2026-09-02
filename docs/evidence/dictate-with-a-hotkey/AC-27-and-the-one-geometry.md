# AC-27 re-proved on the one 472x52 pill, and the geometry measured. 2026-09-02

The twelfth amendment changed the pill's size, and the record's standing line
says any change to the pill's geometry re-opens the AC-27 live click proof. It
was re-run. It passes.

## The geometry, measured

Read from Windows by handle, by a per monitor DPI aware harness:

```text
virtual screen per GetSystemMetrics: 3840 x 2160
pill rect 1901,1624,2909,1860 = 1008x236 device pixels
monitor scale for that window: 2
so in logical pixels: 504x118
```

504 x 118 logical is exactly what the twelfth amendment specifies:

| Piece | Logical px |
|---|---|
| The pill itself | 472 x 52 |
| The counter band reserved beneath it | 34 |
| Transparent margin for the shadow, each side | 16 |
| **Window total** | **504 x 118** |

**One size for its whole life.** The rect was read in every state this sitting
produced: before words, with a live transcript, on the password refusal, on
`MIC STOPPED`, on `DEEPGRAM_CONNECTION_LOST`, on `DEEPGRAM_KEY_REJECTED`, and
across six consecutive open and close cycles. It was 1008x236 every single
time, and its position never drifted between cycles (1901,1624 on all six).
A window that is sized once and never resized is what makes AC-25's promise
structural rather than defended.

**The counter band is reserved and empty.** Visible in every frame as clear
space beneath the pill. The elapsed and word count chip is deliberately not
built and has its own unticked plan row; its arrival will resize nothing,
because the footprint already includes it.

## AC-27: met

A WinForms window holding a real Win32 edit control took focus. Every action
below was synthetic input through the same Windows input queue a person uses.
The foreground window was read by handle at every step and the landed text read
out of the control by `WM_GETTEXT` and compared as an exact string.

```text
[1] click into the scratch box
    fg=[WindowsForms10...] [EchoScribe verify scratch box]
    box text now: []
[2] ctrl double tap, timing the pill
    pill appeared 382ms after the second tap's release
    pill window rect 801,812,1305,930  =  504x118
[3] click the inert part of the pill at 1057,854
    fg unchanged=True
    box now: [abcd]
[4] click the grip at 839,854, no drag
    fg unchanged=True
    box now: [abcdefgh]
[5] drag the grip 150px right
    pill rect 801,812 -> 951,812   moved 150px
    size after the drag: 504x118  (never resized: True)
    fg unchanged=True
    box now: [abcdefghijkl]
[6] ctrl double tap straight after touching the pill (AC-5)
    pill closed 501ms after the second tap
    final box text: [abcdefghijkl]
```

(Step 2's rect is the same window read through a DPI unaware reader, which
returns logical pixels. The DPI aware read above is the same window in device
pixels. The two agree.)

- **A single click on the inert bars**: foreground never changed, `abcd` landed.
- **A single click on the grip**: foreground never changed, `abcdefgh` landed.
- **A 150 pixel drag of the grip**: the window moved exactly 150 pixels, the
  foreground never changed, `abcdefghijkl` landed, and the window did not
  resize.
- **The person's place in the app was not lost**: each burst of four characters
  appended to the previous, in order, with nothing missing and nothing
  reordered.

## AC-24's remembering half, exact

The drag was stored, read by SQL before and after:

```text
pill_x  0.548177083333333  ->  0.626302083333333
```

The difference is 0.078125, which is 150/1920 to the last digit. `pill_y` did
not move for a horizontal drag. The pill's stored spot is a fraction of the
working area and says nothing about its size, which is why the twelfth
amendment needed no migration.

The carrying-across-screens clause of AC-24 still needs a second monitor, so
AC-24 as a whole stays blocked.

## AC-5 and AC-1's timing

- **AC-5**: the pill closed 501 ms after a double tap that arrived straight
  after the pill had been touched, with no click anywhere in between. Across a
  separate run of six consecutive toggles all six landed: opens 350 to 427 ms,
  closes 613 to 987 ms.
- **AC-1's one second**: the pill appeared 340, 375, 382, 388, 395, 401, 413,
  419, 427 and 444 ms after the second tap across this sitting's runs. Every
  open was well inside the second AC-1 allows.
