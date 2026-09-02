# Step 4a's live typing proof: both mechanisms, compared as strings. 2026-08-31

The proof the record's step 4a asks for, done live and filed. The earlier live
proof of 2026-08-31 was performed during the `/debug` sitting and never filed;
this sitting redid it from scratch rather than citing memory of it.

Build: commit `3b4fb62`, clean tree. Model `nova-3`, English, punctuation on.

## Method, and its honest limit

The record's original proof compared what was sent against what landed, using
probes. The probes are gone and the data rules forbid the app logging
transcripts, so nothing in the app can say what was sent. The user chose the
substitute before the sitting started: a fixed script read aloud, each landed
string captured mechanically (`WM_GETTEXT` on the receiving text box, written
to a file, compared as bytes) and diffed against the script. That is weaker
than sent against landed, because a difference can come from transcription as
well as typing and each one costs a diagnosis. Every difference found is
listed below with its attribution.

The script: "The quick brown fox jumps over the lazy dog." then "She sells
fresh bread at the corner market every morning." The user consistently spoke
"lazy white dog", their own variant, confirmed word by word after each run,
so the comparison target below is the script as actually spoken.

## The dictations, and their landed bytes

**D1, the new Windows Notepad, the direct channel receiver.** Landed 50
bytes: `The quick brown fox jumps over the lazy white dog.` Character perfect
against the spoken sentence. (`landed-notepad-d1.txt` in the sitting
scratchpad; the byte dump is in the sitting log.)

**D2, a classic Win32 edit control, the default keystroke path.** A WinForms
text box, which is a real Edit control in an unpackaged process, so the
collapse list's recogniser gets no package identity and the cannot tell rule
must route the default keystrokes. Landed 58 bytes:
`She sells fresh bread at the corner markets every morning.` One difference
against the spoken sentence: `markets` for `market`. Attribution:
transcription, not typing. A typing fault on this path duplicates or drops
whole keystrokes in bursts (see the Notepad collapse finding); a single
plausible-word substitution is Deepgram hearing an s. Nothing else differed.

**D3, both sentences in one dictation into Notepad, for the join.** Landed
108 bytes, character perfect against both spoken sentences, `market` correct
this time. Byte checked: the first byte is `T` with nothing before it, the
join between `dog.` and `She` is exactly one space, and the file contains no
double space anywhere. That is AC-4's join half, held mechanically: one space
between finalised phrases, none before the first.

**D4 (second run), the same script landed once in each receiver in one
dictation** after a mid dictation focus move: Notepad 108 bytes character
perfect, the edit box 109 bytes, the same 108 preceded by one join space,
which is the decided rule applying to a phrase that is not the dictation's
first. Two mechanisms, one dictation, both intact. Full detail in
`AC-7-focus-follows-mid-dictation.md`.

## Which mechanism ran, said honestly

Stderr does not name the mechanism per phrase, so it is not read off a log.
The evidence is: the routing is decided by `collapse_list.rs`, whose exact
match and cannot tell rule are unit tested; the receiver in D1/D3/D4 is the
new Notepad, the one entry on the list; the receiver in D2/D4 has no package
identity, so only the default keystrokes can have run there; and the burst
that collapsed under keystrokes on 2026-08-30 arrived intact in Notepad in
every run today. Inference from behaviour, labelled as such.

## Also proved by these runs

- **AC-3 live**: the user confirmed words arrived in the focused window in
  phrases while still mid speech, no key held. Both receivers.
- **AC-4's never-taken-back half**: the user confirmed typed text only ever
  grew; and in D4's first (discarded) attempt, wording still unfinalised when
  dictation was stopped never landed at all, which is the discard working.
- **AC-1's timing and waveform**: across every open today the pill appeared
  126ms to 276ms after the second tap's press, measured by polling for the
  pill window; the user confirmed the waveform moved with their voice.
- **AC-10's last clause**: the key saved on 2026-08-30 (`64ef`, Credential
  Manager) streamed real audio today. "Dictation works from then on,
  including after closing and reopening the app" now holds end to end.
- **The dictation table held 0 rows after every run**: history is milestone
  5, and no transcript, final or interim, was stored anywhere. Read by SQL.

## Differences found, all attributed

| Where | Landed | Spoken | Attribution |
|---|---|---|---|
| D2 | `markets` | `market` | transcription (single plausible-word substitution; typing faults on this path look like bursts, not grammar) |

No difference in any run was attributable to typing. Across four scored
dictations and 375 landed characters, typing was character perfect in both
mechanisms.
