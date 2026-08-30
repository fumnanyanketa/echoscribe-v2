# AC-9, AC-10, AC-11: the Deepgram key

**Live, 2026-08-30.** Build: commit `5ffc7f2`, clean tree.

## Starting from no key at all

The saved key was removed by this session before the test, with the user's
agreement, so AC-9's path could fire for real:

```
sqlite> delete from deepgram_credential;   -- 1 row, 0 remaining
cmdkey /delete:"deepgram:user_3IXiPaRho7Jkw48Yq8MMHCwCLyC.com.echoscribe.app"
CMDKEY: Credential deleted successfully.
```

The sign-in session credential was left untouched.

## AC-9, the hotkey with no key saved

Notepad focused, double tap Ctrl. What the user saw:

- The EchoScribe window **came to the front on its own**.
- It showed the guided setup screen, asking for the API key.
- It carried a button offering to take them to Deepgram to get one.
- **No pill appeared. No sound played.**

The microphone was never touched: `try_start` checks `has_deepgram_key` before
the device, which is why there was no pill and no sound rather than a pill that
appeared and then went away.

The link out itself, `https://console.deepgram.com/signup?jump=keys`, was not
clicked during this sitting. The user reports having used it during the
2026-08-30 build session and that it landed them on the keys screen, and chose
not to repeat it. Recorded on their observation, not this session's.

## AC-11, an invalid key

Not re-exercised this sitting. The user reports it working from the 2026-08-30
build session and chose not to repeat it. Recorded on their observation.

Note on what this session can and cannot say: the saved row afterwards reads
`key_last_four = 64ef`, which is the real key. That is consistent with nothing
having been saved for a bad key, but it is not proof, because no bad key was
submitted during this sitting.

## AC-10, a real key accepted and saved

The user pasted their real Deepgram key and pressed Verify during this sitting.

What this session read afterwards, from the live database and Credential
Manager:

```
deepgram_credential:
user_3IXiPaRho7Jkw48Yq8MMHCwCLyC|64ef|deepgram:user_3IXiPaRho7Jkw48Yq8MMHCwCLyC|2026-08-30T10:55:32Z|2026-08-30T10:55:32Z

cmdkey /list:
Target: LegacyGeneric:target=deepgram:user_3IXiPaRho7Jkw48Yq8MMHCwCLyC.com.echoscribe.app
```

Both were recreated after the deletion above. The row carries the account id,
the last four characters, the vault entry name and the two timestamps. **The
key itself is not in the database.** That is the data rule holding, checked
directly rather than assumed.

## AC-10, dictation working from then on

- Immediately after saving: Notepad focused, double tap Ctrl. The pill
  appeared and the opening chime played. No setup screen.
- **After a full restart**: this session stopped the process and started a new
  one. Notepad focused, double tap Ctrl. The pill appeared and the setup screen
  did **not** ask for a key again.

## The clause that cannot be proved here

AC-10 promises that a checked key means "dictation works from then on".
Nothing streams until milestone 4. What is proved is that a checked key is
saved, survives a restart, and stops the setup screen standing between the
hotkey and the microphone. Whether that key can open a Deepgram streaming
socket is not proved and cannot be: `GET /v1/auth/token` returning 200 does not
prove scope, which is the gap record 0002 names in its own Still open section
and hands to milestone 4.

## Outcome

AC-9: **met**. The link-out clause rests on the user's build-session
observation, not this run.
AC-10, save and persistence: **met**, first hand, with the database and
Credential Manager checked directly.
AC-10, "dictation works from then on" in the full sense: **blocked**. Milestone
4. Not rounded up.
AC-11: **met** on the user's build-session observation, not re-observed here.
