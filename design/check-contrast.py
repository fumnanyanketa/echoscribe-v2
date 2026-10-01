"""Contrast audit for EchoScribe.

Reads the token values straight out of src/styles.css, so this can never
disagree with what the interface actually uses. Run it after any colour
change. /canvas drift runs it too.

    python design/check-contrast.py

Exits non-zero if any pair fails, so it can go in a check later.
"""
import re
import sys
import pathlib

CSS = pathlib.Path(__file__).resolve().parent.parent / "src" / "styles.css"


def tokens():
    text = CSS.read_text(encoding="utf-8")
    return dict(re.findall(r"(--[a-z0-9-]+):\s*(#[0-9a-fA-F]{6})\s*;", text))


def _channel(value):
    value = value / 255
    return value / 12.92 if value <= 0.03928 else ((value + 0.055) / 1.055) ** 2.4


def luminance(hex_colour):
    h = hex_colour.lstrip("#")
    r, g, b = (int(h[i:i + 2], 16) for i in (0, 2, 4))
    return 0.2126 * _channel(r) + 0.7152 * _channel(g) + 0.0722 * _channel(b)


def ratio(fg, bg):
    a, b = luminance(fg), luminance(bg)
    hi, lo = max(a, b), min(a, b)
    return (hi + 0.05) / (lo + 0.05)


# name, foreground token, background token, kind
# kind "text" needs 4.5:1, "large" needs 3:1 (19px semibold and up),
# "nontext" needs 3:1, "decor" carries no meaning and is exempt.
PAIRS = [
    ("Final transcript in pill", "--color-ink-on-dark", "--color-pill", "text"),
    ("Interim transcript in pill", "--color-ink-interim", "--color-pill", "text"),
    ("Pill status label", "--color-accent-on-dark", "--color-pill", "text"),
    # The comp's drawn value, restored 2026-10-01 by the user. 2.9:1: a
    # non-text decorative handle with a 44px hit area and a grab cursor,
    # exempt under 1.4.11, in the comp's own words.
    ("Pill drag grip", "--color-ink-grip", "--color-pill", "decor"),
    ("Level meter fill", "--color-accent", "--color-pill", "nontext"),
    ("Warning label in pill", "--color-warning-on-dark", "--color-pill", "text"),
    ("Error label in pill", "--color-danger-on-dark", "--color-pill", "text"),
    ("Text in pill well", "--color-ink-on-dark", "--color-pill-raised", "text"),
    ("Elapsed and word count chip", "--color-ink-on-dark-soft", "--color-pill", "text"),
    ("Pill hint", "--color-ink-hint", "--color-pill", "text"),
    ("Interim dotted rule", "--color-ink-interim-rule", "--color-pill", "decor"),
    ("Error sentence on fault fill", "--color-ink-on-dark", "--color-pill-danger", "text"),
    ("Error code on fault fill", "--color-danger-on-dark", "--color-pill-danger", "text"),
    ("Control edge in pill", "--color-pill-border-control", "--color-pill", "nontext"),

    ("Heading and body", "--color-ink", "--color-surface", "text"),
    ("Secondary body", "--color-ink-secondary", "--color-surface", "text"),
    ("Caption and metadata", "--color-ink-caption", "--color-surface", "text"),
    ("Faint counts", "--color-ink-faint", "--color-surface", "text"),
    ("Caption on a row", "--color-ink-caption", "--color-surface-subtle", "text"),
    ("Faint on a row", "--color-ink-faint", "--color-surface-subtle", "text"),
    ("Caption in a well", "--color-ink-caption", "--color-surface-sunken", "text"),
    ("Faint in a well", "--color-ink-faint", "--color-surface-sunken", "text"),
    ("Link", "--color-accent-link", "--color-surface", "text"),
    ("Danger text on light", "--color-danger-on-light", "--color-surface", "text"),
    ("Control edge on surface", "--color-border-control", "--color-surface", "nontext"),
    ("Control edge on a row", "--color-border-control", "--color-surface-subtle", "nontext"),
    ("Focus ring on surface", "--color-accent", "--color-surface", "nontext"),
    ("Focus ring on pill", "--color-accent", "--color-pill", "nontext"),

    ("Nav item on rail", "--color-ink-nav", "--color-rail", "text"),
    ("Nav item on active row", "--color-ink-nav", "--color-rail-active", "text"),
    ("Active nav item", "--color-surface", "--color-rail-active", "text"),
    ("Control edge on rail", "--color-rail-border-control", "--color-rail", "nontext"),
    # The comp's 2026-10-01 additions to the rail: the version badge, the
    # count at a nav item's right edge, the account name, and the initials
    # disc the user restored to violet.
    ("Version badge on rail", "--color-ink-hint", "--color-rail", "text"),
    # 4.24:1, deliberate, per the comp's own contrast table: "redundant
    # numbers beside already-labelled nav items". The label alone carries
    # the destination; the count is never the only signal for anything.
    # The user chose the comp's drawn value on 2026-10-01.
    ("Nav count on rail", "--color-ink-count", "--color-rail", "decor"),
    ("Account name on rail", "--color-ink-on-dark-soft", "--color-rail", "text"),
    # 4.35:1, deliberate: the comp draws white initials on the violet disc,
    # and the display name sits in full beside the disc, so the two letters
    # are a marker and never the only way a person is named. The user chose
    # the comp's drawn disc on 2026-10-01 over the earlier neutral fill.
    ("Initials on violet disc", "--color-surface", "--color-accent", "decor"),
    # The offline row in the account block (record 0003 AC-14, drawn 2026-09-03).
    # Its two parts are measured against the rail ground rather than borrowing the
    # on-dark pairs above, which are all measured on --color-onboarding.
    ("Offline badge on rail", "--color-warning-on-dark", "--color-rail", "text"),
    ("Offline sentence on rail", "--color-ink-nav", "--color-rail", "text"),

    ("First run heading", "--color-ink-on-dark-heading", "--color-onboarding", "large"),
    ("First run body", "--color-ink-on-dark-secondary", "--color-onboarding", "text"),
    ("First run caption", "--color-ink-on-dark-caption", "--color-onboarding", "text"),

    ("Sign-in primary button", "--color-onboarding", "--color-ink-on-dark-heading", "text"),
    ("Sign-in secondary button edge", "--color-rail-border-control", "--color-onboarding", "nontext"),
    ("Sign-in primary button busy, dim end of pulse", "--color-onboarding", "--color-ink-on-dark-secondary", "text"),
    ("Sign-in primary button busy, shape on the field", "--color-ink-on-dark-secondary", "--color-onboarding", "nontext"),
    ("Sign-in error code", "--color-danger-on-dark", "--color-onboarding", "text"),
    ("Sign-in session-ended notice", "--color-warning-on-dark", "--color-onboarding", "text"),
    ("Focus ring on onboarding", "--color-accent", "--color-onboarding", "nontext"),

    # Deepgram key setup (record 0002 AC-9). The field is a dark well on the
    # first-run window, so its own pairs are audited separately from the pill's.
    ("Key setup field text", "--color-ink-on-dark", "--color-pill-raised", "text"),
    ("Key setup placeholder and SECRET badge", "--color-ink-on-dark-caption", "--color-pill-raised", "text"),
    ("Key setup field while checking", "--color-ink-on-dark-secondary", "--color-pill-raised", "text"),
    ("Key setup field control edge", "--color-rail-border-control", "--color-pill-raised", "nontext"),
    ("Key setup get-a-key link", "--color-accent-on-dark", "--color-onboarding", "text"),
    ("Key setup verify button, nothing pasted yet", "--color-onboarding", "--color-ink-on-dark-secondary", "text"),

    ("Blocker label on wash", "--color-accent-on-light", "--color-accent-wash", "text"),
    ("Blocker edge on wash", "--color-accent-border", "--color-accent-wash", "decor"),

    # The Settings screen (record 0002 AC-12, AC-19, AC-21, AC-22). Its two new
    # controls sit on the white reading surface, and the chosen hotkey row is a
    # --color-surface-subtle fill, so the keycap and badge inside it get their
    # own pairs rather than borrowing the ones measured against white.
    ("Keycap text on a row", "--color-ink", "--color-surface-subtle", "text"),
    ("Hotkey CHOSEN badge", "--color-ink-secondary", "--color-surface-subtle", "text"),
    ("Focus ring on a row", "--color-accent", "--color-surface-subtle", "nontext"),
    # The sound switch. Its meaning is knob position plus the word beside it, so
    # these three are about seeing the control and where the knob sits, never
    # about carrying the state. The off track keeps the --color-border-control
    # edge already audited against the surface: that same edge on
    # --color-surface-sunken is only 2.87:1 and would fail, which is why the off
    # track is the subtle fill and not the sunken one.
    ("Sound switch track, on", "--color-ink", "--color-surface", "nontext"),
    ("Sound switch knob, on", "--color-surface", "--color-ink", "nontext"),
    ("Sound switch knob, off", "--color-ink-faint", "--color-surface-subtle", "nontext"),

    # The saved Deepgram key on Settings, Transcription (record 0002 AC-12,
    # AC-34, AC-35). Added 2026-09-04. Its label, mask, fragment, badge, edge
    # and error line all reuse pairs above, named in the row that draws them.
    # These five are the ones nothing had measured yet, and four of them are
    # the light Primary button primitive, which had gone unaudited since it was
    # registered: the pill, sign in and first run all use the on-dark twin.
    ("Primary button label", "--color-surface", "--color-ink", "text"),
    ("Primary button, nothing to do yet", "--color-surface", "--color-ink-faint", "text"),
    ("Primary button busy, dim end of pulse", "--color-surface", "--color-ink-secondary", "text"),
    # The paste field on the light surface takes the accent edge that
    # "Secret field, empty" draws, because a field waiting to be typed into is
    # the one thing on that surface asking for attention.
    ("Paste field edge on surface", "--color-accent-on-light", "--color-surface", "nontext"),
    ("Accent sentence on surface", "--color-accent-on-light", "--color-surface", "text"),
]

NEEDED = {"text": 4.5, "large": 3.0, "nontext": 3.0, "decor": 0.0}


def main():
    values = tokens()
    failures = []
    width = max(len(p[0]) for p in PAIRS)
    for name, fg_token, bg_token, kind in PAIRS:
        fg, bg = values.get(fg_token), values.get(bg_token)
        if fg is None or bg is None:
            missing = fg_token if fg is None else bg_token
            failures.append((name, "token %s is not in styles.css" % missing))
            print("%-*s  MISSING %s" % (width, name, missing))
            continue
        value = ratio(fg, bg)
        need = NEEDED[kind]
        if kind == "decor":
            verdict = "exempt"
        elif value < need:
            verdict = "FAIL, needs %.1f" % need
            failures.append((name, "%.2f:1, needs %.1f:1" % (value, need)))
        elif kind == "text" and value >= 7:
            verdict = "AAA"
        else:
            verdict = "AA"
        print("%-*s  %s on %s  %5.2f:1  %s" % (width, name, fg, bg, value, verdict))

    print()
    if failures:
        print("%d failing pair(s):" % len(failures))
        for name, why in failures:
            print("  %s: %s" % (name, why))
        return 1
    print("All %d pairs pass." % len(PAIRS))
    return 0


if __name__ == "__main__":
    sys.exit(main())
