# 0006. Speak in your language

**Status:** In progress
**Date:** 2026-09-04
**Weight:** medium
**Plan row:** 4
**Supersedes:** nothing

**Decided without the user, on their own instruction.** Same standing
instruction as record 0005, written up there: the user was away, `/architect`
was told to decide anything with no named source, take the most reversible
option, and write the reasoning down for later ratification. Every such
decision is marked **decided on the user's behalf** where it is made, and all
of them are collected at the end under "Decided on the user's behalf".

## In one line

A Languages section on the settings surface offers every language Deepgram's
`nova-3` model can transcribe, one row per language with the one in force
marked, and the chosen one is what each dictation asks Deepgram for; English
stays the default so nothing changes for anybody until they choose. It buys the
app for everyone who does not work in English. It costs a fixed list of 64
entries that has to be kept in step with Deepgram's, and a promise this project
can only ever prove for a handful of them.

## What this is for

Record 0002 fixed the transcription language to English, in code, and said so:
"the language the transcription runs in is fixed to English here. Plan row 4
owns making it a choice." Until that changes, EchoScribe is an English app,
which is not what it claims to be. Someone who thinks in Polish, dictates in
Japanese, or moves between two languages in the same message cannot use it at
all.

It is plan row 4's `Done when:` exactly: pick a language other than English,
speak, and the text comes out correctly in that language.

## Acceptance criteria

- **AC-1**: Settings, Languages lists the languages EchoScribe can transcribe,
  with the one in force clearly marked. Until I change it, that is English.
- **AC-2**: I pick another language and it is in force from my next dictation,
  with no restart, and it is still in force after closing and reopening the app.
- **AC-3**: I speak in the language I picked and the words come out in that
  language, in its own writing, typed at my cursor in whatever app I am in.
- **AC-4**: A dictation that is already running when I change the language is
  not changed. The change applies to the next one.
- **AC-5**: One of the choices is Multilingual, and its row says which
  languages it covers. With it picked I can move between those languages inside
  one dictation.
- **AC-6**: Typing in the filter narrows the list to matching languages, and
  clearing it shows them all again. The list only ever holds languages
  EchoScribe can really transcribe in.
- **AC-7**: Each dictation saved to history records the language it ran in.
- **AC-8**: My language is my own. A second account signing in on the same
  machine has its own choice, and changing theirs does not change mine.
- **AC-9**: If my language cannot be read when the screen opens, the screen is
  one error line and draws no control at all: no list, no filter, nothing
  holding a guessed value.
- **AC-10**: A change that cannot be saved leaves the marked language exactly
  where it was on screen, and says so on the same one error line.
- **AC-11**: Text in a language that reads right to left is shown the right way
  round on the pill, and is typed at my cursor in the order I said it.
- **AC-12**: The words I have added under Teach it your words still apply in the
  language I picked.

## The decision

**One chosen language per account, sent as Deepgram's `language` parameter on
the stream that record 0002 already opens.** Nothing is installed, no new
outside address is called, and no new library is needed: the Deepgram Rust SDK
already takes a language on the options builder, and `transcribe.rs` already
passes one, hard coded to English. This feature replaces that constant with a
stored value.

**One language, not a list of languages with a default.** This is the load
bearing shape decision and it is **decided on the user's behalf**.
`design/registry.md` draws `Language list` as "Rows with a default marker, plus
an add action", which describes a person keeping several languages and marking
one as usual. That shape has nowhere to put the others: switching language
mid-dictation would need a control on the pill, and nothing is drawn for one
and AC-30 of record 0002 forbids the pill asking to be clicked at all; a second
hotkey is refused outright by AC-22. So the non-default rows in that drawing
would do nothing a person could reach. The registry's own precedent covers
this: its `Hotkey display` row was corrected for exactly this reason, because
"a settings screen that displays the hotkey without letting a person choose it
cannot meet AC-19", and the rule this project applies is that the record wins
on content and the comp wins on look. `/canvas` therefore owes `Language list`
a correction, not this record a bigger feature.

**The person who genuinely switches languages is served by Deepgram's own
multilingual model, not by a list.** `language=multi` on `nova-3` transcribes
English, Spanish, French, German, Hindi, Russian, Portuguese, Japanese,
Italian and Dutch, switching between them inside one stream. So "Multilingual"
is one of the choices in the list rather than a second mechanism, which is
AC-5, and it means the multi-language case costs one row instead of a feature.

**The list is every language Deepgram's `nova-3` table names, one row per
language, plus Multilingual: 64 entries.** Read from
`https://developers.deepgram.com/docs/models-languages-overview` on 2026-09-04.
The source is Deepgram's own table and the rule for reading it is:

- One row per language named in that table, using the first code its row gives.
  So Portuguese is `pt`, not `pt-BR` and `pt-PT` as well.
- Where Deepgram's own table names a variant as a distinct language, it is a
  distinct row, because that is Deepgram's distinction and not one this record
  invented: Chinese (Cantonese, Traditional) `zh-HK`, Chinese (Mandarin,
  Simplified) `zh`, Chinese (Mandarin, Traditional) `zh-TW`, Flemish `nl-BE`,
  German (Switzerland) `de-CH`.
- Regional variants of one language are **not** offered. This is **decided on
  the user's behalf**. Arabic alone has 17 in Deepgram's table, and English six,
  so offering them would roughly double a list that is already long, to make a
  distinction about accent that no acceptance criterion asks for. Named in Still
  open as the thing to revisit if a real person's accent comes out wrong.

**English stays the default, and this feature changes nothing for anybody until
they choose.** **Decided on the user's behalf.** The alternative, guessing from
the language Windows is set to, was considered seriously and refused: the
language a person's operating system is in is a poor guide to the language they
dictate in, because plenty of people run an English Windows and speak something
else, and the cost of a wrong guess here is the worst failure this app has.
Deepgram given the wrong language does not fail, it returns confident nonsense,
and this app types it straight into a person's document. So the guess would be
invisible when right and destructive when wrong. Defaulting to English keeps
record 0002's behaviour exactly and puts the choice where a person can see it.

**A change is in force from the next dictation and never inside one.** The
language is read once, in Rust, at the moment the microphone opens, exactly as
record 0005 reads the vocabulary, and the same value is used by the one
reconnect attempt record 0002's AC-14 allows. Reading it once is what makes
AC-4 a simple promise instead of a race, and it means no event and no listener
anywhere.

**A new feature folder on both sides**, `src-tauri/src/language/` and
`src/language/`, with its own table and its own connection to the same SQLite
file. Same reasoning as record 0005: AGENTS.md forbids adding a feature to a
folder that seems related, and record 0004 set the precedent.

**The dictate feature reads the language, and this feature never pushes it.**
One named function, `language_for_dictation`, read only, in one direction, the
same door record 0005 opens for the terms and the same direction record 0004
established when the shell read the dictate feature's classifier.

**The chosen language is matched against the fixed list before anything is
sent.** A stored value that is not in the list reads back as English, never as
an instruction, which is the rule record 0002's data model already applies to a
stored hotkey and a stored pill position. And `set_transcription_language`
refuses anything not in the list outright, before a write, which is the same
refusal `set_hotkey` makes and the same reason: the value arrives from outside
the core, so it is hostile input (AGENTS.md rule 7), and the only thing done
with it is matching it against known values. Nothing in this project ever puts a
string from the interface into a request to Deepgram.

**History records the language a dictation ran in, which is record 0002's
`dictation` table gaining one column.** Record 0002's Still open says in as many
words that plan row 4 "will add a language field to `dictation`", so this is
that, carried by this record's amendment to record 0002. It stores the code the
dictation was asked with, including `multi` when Multilingual was chosen.
`design/registry.md`'s `Dictation row` already draws a `Language tag` for it and
plan row 5 is what reads it. **What is stored is what was asked for, never what
Deepgram detected.** Deepgram can report a detected language per result on the
multilingual model, and reading it would mean a history row whose language
disagrees with the setting that produced it, for no gain that plan row 5 has
asked for. **Decided on the user's behalf.**

**The two sentences a person can read live in Rust, in this feature, and
nowhere else**, the same division record 0002's fourteenth amendment fixed and
record 0005 follows. Both are **decided on the user's behalf**.

| State | Code | Sentence |
|---|---|---|
| The change would not save | `LANGUAGE_NOT_SAVED` | "This language could not be saved, so it is unchanged." |
| The choice would not read | `LANGUAGE_NOT_READ` | "Your language could not be read, so none is shown." |

**Three refusals carry no sentence at all**: nobody signed in, the core not
being up, and a language that is not in the list. The first two cannot happen on
a screen that only exists while somebody is signed in, and the third cannot
happen while the rows come from what `get_transcription_language` handed out.
All three are the shape record 0002's settings commands already have, and the
third is `unknown_hotkey` exactly: the machine reason is kept for a log and the
value itself is never printed.

**Two pieces of static wording are the screen's own**, in
`src/language/language.js`, not Rust's, on the same division as record 0002's
two captions. Both **decided on the user's behalf**.

- The Multilingual row's explanation, which is the one row in the list that
  cannot explain itself: "Switches between English, Spanish, French, German,
  Hindi, Russian, Portuguese, Japanese, Italian and Dutch inside one
  dictation." Those ten are Deepgram's own documented set for `multi`, read
  2026-09-04.
- The caption beneath the list: "This is the language Deepgram is told to
  expect. A change applies to your next dictation, not to one already running."
  It says the two things a person cannot see for themselves, which is what every
  caption on this surface is for: that the choice travels with each dictation,
  and AC-4's timing.

**A filter over the list, because 64 rows cannot be scanned.** **Decided on the
user's behalf.** It narrows the rows as a person types and matches on the name
they can see. It is not a new kind of thing: `design/registry.md` already draws
a `Search field` for history, and `/canvas` registers this surface's variant of
it. It is part of making a list of this length usable rather than a feature of
its own, and it is the most reversible half of the whole record: a list keeps
working with the filter taken away, so if it turns out to be clutter it goes in
one change. The filter is the screen's own business entirely. It asks Rust
nothing, changes nothing, and a language hidden by a filter is still the one in
force.

**The list is sorted by the name a person reads, and only the screen can do
that.** Rust hands out codes, `design/registry.md` and the screen own the
wording, so alphabetical order by name is only knowable on the side that holds
the names. The screen sorts its own labels. Multilingual takes its place in the
alphabet under M rather than being pinned above a heading, because its own row
explains what it is wherever it sits, and pinning it would need a grouping
component nothing draws.

**Right to left text is handled once, by letting the browser decide per
string.** The pill's transcript line, its grey interim tail and the language
rows carry `dir="auto"`, so a line of Arabic or Hebrew is laid out right to
left and a line of English is not, without either one being declared anywhere.
This is **decided on the user's behalf** and it is the one place this record
touches something a person can already see, which AGENTS.md rule 6 would
normally have me ask about first: it is required rather than chosen, because
AC-11 cannot hold otherwise, and the alternative is showing a person their own
sentence backwards. The typed text needs nothing: `typing.rs` sends each UTF-16
unit of the finalised phrase through `KEYEVENTF_UNICODE`, in order, and never
derives a key code from a character, so it is already script independent by
construction and the receiving application lays the text out itself. AC-11's
second half is therefore a check that something already true stayed true.

**Punctuation stays on for every language.** Deepgram documents `punctuate` as
supported for "all available languages" on streaming, read 2026-09-04. What it
does not document is what happens for a language that does not support it, and
this record does not pretend to know: see Still open. Nothing is built for that
state, because the honest position is that no language is known to refuse it,
and record 0002's Deepgram classifier already turns any refusal into a named
kind with a sentence and an action.

**The custom vocabulary is sent in every language, with no fallback.**
**Decided on the user's behalf.** Deepgram documents keyterm as working on
`nova-3` for both monolingual and multilingual transcription, read 2026-09-04,
and that is a named source. The alternative considered was to detect a refused
request and quietly retry without the terms, and it was refused: it would be
code that silently changes what was asked of Deepgram, which is the exact class
of mistake AGENTS.md's standing rule 14 exists for, and the loud failure is the
one a person can actually diagnose. AC-12 is the criterion that catches it, and
if it fails in a real language, an amendment adds the fallback with the evidence
in hand.

**The rail gains a fourth destination, `settings.languages`, and record 0004 is
amended for it.** That record says each of plan rows 3, 4 and 5 amends it to add
its own item as it lands. Its place is the comp's order, Dictation, Languages,
Vocabulary, Transcription, which `design/registry.md`'s `Section sub-nav` row
already names. **This is the second of two amendments to that one list in one
session**, record 0005's being the other, and each names the other so that
neither is read as the whole change. Nothing else about record 0004 changes: no
new command, no new event, and `dashboard.json` is untouched.

**The section is called Languages, in the plural, and that is deliberate.** The
comp and `design/registry.md`'s `Section sub-nav` both name it that, and this
project's rule is that the registry owns wording. A section named for its
subject reads correctly even when the setting inside it is one choice, in the
same way Settings holds one screen today.

**Nothing on the out of scope list is built.** No language detection from the
audio, no per-application language, no translation of any kind. Deepgram can
detect a language and this record deliberately does not use it: it would put the
app in the position of deciding what language a person just spoke, and a wrong
answer types nonsense into their document. Translation is not on the plan and
would be a different product.

## What else was considered

| Option | Why not |
|---|---|
| A list of languages you use with one marked default, as `design/registry.md` draws it | The non-default rows would do nothing reachable: switching mid-dictation needs a control on the pill, which nothing draws and record 0002's AC-30 forbids, or a second hotkey, which AC-22 refuses. Deepgram's own `multi` covers the person who switches, as one row instead of a mechanism. |
| Guessing the language from what Windows is set to | Invisible when right and destructive when wrong: Deepgram given the wrong language returns confident nonsense and this app types it into the person's document. The operating system's language is also a poor guide to the language somebody dictates in. |
| Detecting the language from the audio, with Deepgram's own detection | It makes the app decide what language a person just spoke, on every dictation, with no way for them to correct it before the words are typed. A setting they chose once is a promise the app can keep. |
| Offering every regional variant Deepgram lists | Roughly doubles a 64 row list, with 17 rows of Arabic and six of English, to make a distinction about accent that no criterion asks for. Kept in Still open in case a real accent is mis-transcribed. |
| Only the ten languages Deepgram's `multi` covers, about a dozen rows | Genuinely tempting: a shorter list, no filter needed, and it covers most people. It denies the app to a Polish or Ukrainian speaker whose language Deepgram supports perfectly well, which is exactly the person this plan row exists for. |
| Storing the language on record 0002's `dictation_setting` row | Puts a new feature's data inside another feature's table, which AGENTS.md's folder rule exists to prevent and record 0004 already set the opposite precedent for. The one column this record does add to record 0002's schema is on `dictation`, which that record's own Still open asked for by name. |
| Storing what Deepgram detected on each history row instead of what was asked for | A history row whose language disagreed with the setting that produced it, for a gain plan row 5 has not asked for. What was asked for is a fact this app owns; what was detected is a guess it would be repeating. |
| Retrying without the custom vocabulary when a request is refused | Code that silently changes what was asked of Deepgram, so a person's words would quietly stop being used with nothing saying so. The loud failure is the one that can be diagnosed. |
| Reusing record 0002's `SETTING_NOT_SAVED` sentence and its type | Three features now want a settings error line, which is exactly the threshold AGENTS.md sets for making something shared. Doing it now means moving a shipped feature's type and rewriting its tests for no criterion, so the question is named in Still open instead and each feature keeps its own two sentences. |

## Data model

One new table for this feature, and one new column on a table record 0002 owns.
Everything is in the same SQLite file, per AGENTS.md. The `account` table is the
sign in feature's, plan row 1, and is referenced here.

| Table | Key | Fields | Relationship |
|---|---|---|---|
| `transcription_language` | `account_id` | `language` text, required, one of the 64 in the fixed list, defaulting to `en`. `updated_at` text, required. | One account has zero or one |

And, by this record's amendment to record 0002:

| Table | Change |
|---|---|
| `dictation` | Gains `language` text, required, defaulting to `en`. Written at the close of every dictation that leaves a row. |

Rules that must always hold:

- `account_id` is never empty. Nothing is written until we know whose it is.
- `language` is refused on write if it is not one of the 64 in the fixed list.
  A row read back holding anything else is treated as English, never as an
  instruction to the stream, which is the rule record 0002 already applies to a
  stored hotkey.
- A missing row means English. A new account has no row and needs none.
- `dictation.language` holds the code the dictation was asked with, which is
  `multi` when Multilingual was chosen. It is never what Deepgram detected.
- Deleting an account deletes its `transcription_language` row along with
  everything else of theirs. Record 0002 lists three rows and a Credential
  Manager entry, record 0004 adds a fourth, record 0005 adds its terms; this is
  the sixth thing.
- Nothing about what a person said is in `transcription_language`. It holds one
  language code and a timestamp.

**Two migrations, and the second one is the careful half.** The first creates
`transcription_language`, in this feature's own schema, and is idempotent like
every other one here. The second adds `language` to `dictation`, which
`CREATE TABLE IF NOT EXISTS` cannot do for a file that already has the table.
SQLite has no `ADD COLUMN IF NOT EXISTS`, so the dictate feature's schema step
reads the table's own columns first and adds the column only when it is absent.
Existing rows take the default, `en`, and that is a true statement rather than a
convenient one: record 0002 fixed every dictation before this feature to English
in code.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The signed in account id | AC-2, AC-7, AC-8 | The Clerk session held in the Rust core, per record 0003. Never passed in from the interface, and no command here takes one. |
| The chosen language | AC-1, AC-2, AC-3 | `transcription_language.language`, defaulting to `en` when there is no row. |
| The set of languages that may be chosen | AC-1, AC-6 | This record, as a fixed list in `src-tauri/src/language/`, taken from Deepgram's own `nova-3` table read on 2026-09-04. Handed out by `get_transcription_language`, never a list the interface holds, for the same reason `get_hotkey` hands out the two hotkeys: a screen must not be able to offer something Rust would refuse. |
| What a person reads for each language | AC-1, AC-6 | `design/registry.md` and the screen, which own wording, exactly as they do for the rail's items and the two hotkey rows. Rust hands out codes. A code with no wording is not drawn at all, and a Rust test fails when Rust can hand out a code the screen has no name for. |
| The order the list is shown in | AC-1, AC-6 | This record: alphabetical by the name a person reads, which only the side holding the names can do, so the screen sorts. Multilingual falls under M. |
| What Multilingual covers | AC-5 | Deepgram's documented set for `language=multi`, read 2026-09-04: English, Spanish, French, German, Hindi, Russian, Portuguese, Japanese, Italian, Dutch. Named in the row's own wording, because it is the one row that cannot explain itself. |
| The default language | AC-1 | This record. English, `en`, which is exactly what record 0002 fixed in code, so nothing changes for anybody until they choose. |
| When the language is read for a dictation | AC-2, AC-4 | This record. Once, in Rust, at the moment the microphone opens, through one named function in this feature. The same value is used by the one reconnect attempt, so a dictation cannot change language halfway through. |
| What happens if the language cannot be read at that moment | AC-3 | This record. The dictation runs in English, which is what it did before this feature existed, and nothing on screen changes. A settings read never stops a dictation. |
| What is asked of Deepgram on the stream | AC-3, AC-5 | Record 0002, as amended by its eighteenth amendment: `nova-3`, punctuation on, interim results on, this feature's chosen language, and record 0005's terms as `keyterm`. |
| Whether punctuation is asked for in every language | AC-3 | Deepgram's punctuation documentation, read 2026-09-04, which names "all available languages" for streaming. Unchanged from record 0002, and see Still open for what is not documented. |
| The language stored against a dictation in history | AC-7 | `dictation.language`, written by record 0002's `save_dictation` from the same value the stream was opened with. Never what Deepgram detected. |
| Which way round transcribed text reads | AC-11 | The browser, from `dir="auto"` on the elements that hold text from outside: the pill's transcript line and its interim tail, and each language row. Nothing declares a direction per language, and no list of right to left languages is held anywhere. |
| How characters of any script reach the cursor | AC-3, AC-11 | Record 0002's `typing.rs`, unchanged. Each UTF-16 unit goes through `KEYEVENTF_UNICODE` in order and no key code is derived from a character, so it is already script independent. |
| Whether the custom vocabulary applies | AC-12 | Record 0005, unchanged. Deepgram documents keyterm for `nova-3` monolingual and multilingual alike, read 2026-09-04, so the terms are sent whatever the language and there is no fallback. |
| The sentence for each refusal | AC-9, AC-10 | This record, the two sentence table in The decision. Fixed wording, held in one place in Rust, never carrying anything read off the value. |
| Which refusals carry no sentence | AC-9, AC-10 | This record: nobody signed in, the core not up, and a language that is not in the list. All three unreachable from this screen; the precedent is record 0002's fourteenth amendment. |
| The caption beneath the list | AC-2, AC-4 | This record, the sentence in The decision. The screen's own static wording, in `src/language/language.js`. |
| What a person reads for the rail item | AC-1 | `design/registry.md`, which owns wording. Rust hands out the identifier `settings.languages` and nothing else. |
| Which sections the rail holds | AC-1 | Record 0004's fixed list in `src-tauri/src/shell/rail.rs`, as amended by this record to hold `settings.languages` and by record 0005 to hold `settings.vocabulary`. |

## Interface surface

Two commands, in the new feature, on the same terms as every other command in
this project: the interface asks and Rust decides, neither takes an account id
because Rust knows it from the session, and both refuse when nobody is signed
in.

- `get_transcription_language()` returns the chosen code and the complete list
  of codes it may be chosen from, so the interface renders a list rather than
  deciding what is allowed. The same shape as `get_hotkey`, deliberately, and
  for the same reason.
- `set_transcription_language(language)` takes one of those codes and refuses
  anything else outright, before anything is written. That refusal is the only
  one it has left to make.

No event, in either direction. Nothing outside this feature needs to know the
language changed: the dictate feature reads it at the start of each dictation,
which is the only moment it matters, and AC-4 says so.

Nothing in `src-tauri/capabilities/` is added or widened. The screen listens to
nothing and emits nothing, and record 0004's closed Still open established that
Tauri grants no per command permission for this application's own commands: the
`not_signed_in` refusal in Rust is what protects them, and both commands make
it.

One function this feature exposes to the rest of the Rust core, and it is not a
command:

- `language_for_dictation(&app) -> String`, read only, called by the dictate
  feature at the moment the microphone opens. It returns `en` on anything at all
  going wrong, including nobody being signed in, so a failure here can never
  stop a dictation. It is the one door between the two features and there is
  deliberately no second.

One change to a function record 0002 owns, carried by this record's amendment
to it:

- `Store::save_dictation` gains the language the dictation ran in, so the
  history row records it. Every one of its three call sites passes the value the
  stream was opened with, not a fresh read, so a language changed mid-dictation
  cannot end up on a row it did not apply to.

Errors that matter and must each read differently: the change not saving, the
choice not reading, and not signed in. The first two have the sentences in The
decision. The third carries none, for the reason given there.

## Risk

The plan row is medium weight, but this feature changes what is typed into a
person's own documents and it changes what is asked of an outside service, so
the three questions are answered rather than skipped.

**The worst thing a malicious person could do here.**

1. **Get a string of their own into the request to Deepgram.** The language ends
   up in the query of the streaming address, so a value like
   `en&redact=pci` reaching it unchecked would change what is asked, on every
   dictation, silently. Refused before it can start: the incoming value is
   matched against the fixed list of 64 codes and anything else is refused
   outright, so nothing that is not one of those 64 literals ever reaches the
   request. This is `set_hotkey`'s refusal, applied to a longer list.
2. **Make dictation stop working by choosing a language Deepgram will not
   accept.** The list is Deepgram's own, and a stored value outside it reads
   back as English rather than being sent. So there is no reachable choice that
   breaks the stream.
3. **Learn something about a person from what they store.** One language code
   per account is not much, but it does say what language somebody speaks. It
   lives in the same SQLite file as everything else, on their own machine,
   against exactly one account, and a second account on the machine has its own,
   which is AC-8. Same protection as everything else here, and no stronger:
   anybody with the machine and the file has it.

**What we are storing that we must protect.** One language code and a timestamp
per account, and one language code on each history row. Nothing about what was
said, nothing secret. The history row's language is the more sensitive of the
two, because it sits beside the words that were dictated, and it is protected
exactly as those words are: record 0002's AC-18, one account's rows are not
visible to another.

**What the system should refuse to do, even if asked nicely.**

- Send Deepgram a language that is not one of the 64 literals in the fixed list.
- Take the language for a request from the interface rather than from the store.
- Guess the language, from the machine's settings or from the audio.
- Change the language of a dictation that is already running.
- Store what Deepgram detected in place of what was asked for.

## Build plan

Three milestones, each leaving the project working.

1. **The store, the fixed list and the two commands.** The table, the 64 codes,
   the strict read for a write and the lenient read for a stored value, and the
   two commands with their two sentences. Nothing calls them yet, so nothing a
   person can see changes. This is where the list and the refusal are tested.
2. **The language reaching Deepgram, and the history column.**
   `language_for_dictation` in this feature; record 0002's `transcribe.rs`
   sending the stored language on both the first connection and the one
   reconnect; record 0002's `dictation` table gaining its column and
   `save_dictation` writing it. Proves AC-3, AC-4 and AC-7. At the end of this
   milestone the feature works with no screen: a language set by hand in the
   database is used, which is how AC-3 can be checked before there is anything
   to click.
3. **The screen and the rail item.** The Languages surface, its filter, its
   rows with the one in force marked, the Multilingual row's own wording, the
   read failure state, the caption, `dir="auto"` where text from outside is
   shown, and record 0004's rail gaining `settings.languages`.

## What this makes harder

- **Two lists now have to change together.** 64 codes in Rust and 64 names in
  the screen, and Rust must never hand out a code the screen cannot name. A test
  guards it, and it is the same shape as the guard the rail already has, but it
  is a real maintenance cost that grows every time Deepgram adds a language.
- **This project cannot prove most of what it now offers.** 64 choices, and
  `/check verify` will run a handful. The list is honest about what Deepgram
  supports and silent about what has been tested. Named in Still open, because
  the alternative is offering fewer languages than the service supports.
- **Record 0002's `save_dictation` gains a parameter**, so all three of its call
  sites change, and record 0002's own tests for it change with them. The one
  place where this feature reaches into a shipped one.
- **Record 0002's `dictation` table needs a real migration**, not another
  idempotent create. It is the first column this project has added to an existing
  table, so the guarded `ALTER TABLE` is a new pattern here and the next feature
  will copy it.
- **The pill now shows text in scripts nobody has looked at.** `dir="auto"`
  handles direction; it does not promise that a long line of Devanagari fits the
  one fixed 472x52 geometry record 0002's twelfth amendment locked. That geometry
  was measured with Latin text.
- **Punctuation is asked for in every language on documentation alone.** If one
  refuses it, dictation fails in that language and the person sees the Deepgram
  catch-all, which names no cause.
- **The rail's fixed list changes twice in one session**, once here and once in
  record 0005. Both amendments name the other, and record 0004's AC-2 now covers
  four destinations.

## Still open

- **Which languages have actually been proved.** Only the ones `/check verify`
  runs. AC-3 is one criterion and it stands for 64 choices, so the evidence file
  has to name which language it was proved in, and a second language is worth
  running for the script alone: one in a non Latin writing system, and one that
  reads right to left, are the two that could fail differently.
- **What Deepgram does with `punctuate` for a language that does not support
  it.** Its documentation says "all available languages" and does not say what
  happens otherwise, so this record does not know. If a refusal is ever seen, the
  fix is to send punctuation only where it is supported, which means a second
  fixed list and an amendment.
- **Whether keyterm holds in every language.** Shared with record 0005's Still
  open. Documented as working for `nova-3` monolingual and multilingual, not
  proved here, and AC-12 is the criterion that would catch it.
- **Whether the pill's one locked geometry holds for every script.** 472x52 was
  settled with Latin text. A script with taller glyphs, or a language whose words
  are much longer, may overflow it. Not this record's to change: the geometry is
  record 0002's, and a change reopens its AC-27 live click proof.
- **Whether regional variants are owed.** One row per language today. If a real
  person's accent comes out wrong, the answer is to offer that language's
  variants, which is an amendment and a longer list rather than a new mechanism.
- **Whether the settings error line should be a shared module.** AGENTS.md says
  something becomes shared when three features need it, and with this record
  three do: the dictate feature's settings, record 0005's vocabulary and this
  one. Each holds its own two sentences today, which is right, because the
  sentences differ and they are the part that matters. What is duplicated is the
  three field shape. Moving it means changing a shipped feature's type and its
  tests for no acceptance criterion, so it is named here for the user to settle
  rather than done quietly.
- **Whether the language names should be in each language's own words.** They
  are Deepgram's English names today, which is a real gap for exactly the person
  this feature is for: somebody who does not read English is being asked to find
  their language in an English list. It was not fixed here because 63 endonyms
  invented from memory would be wording a person reads with no named source, and
  this project refuses that. What would settle it is a named source: a locale
  library, which is a new dependency and therefore the user's decision, or a
  list the user writes. The filter softens it, because a person can type their
  language's name in their own script only if the names are in it, which they
  are not, so it softens it less than it looks. Registered in
  `design/registry.md`'s `Language list` and in that day's patrol report as the
  one finding left open.
- **Whether a person wants a different language per application.** Nobody has
  asked. It would need somewhere to say which application, which is a new kind
  of setting entirely.

## The wording of the screen

**Added 2026-09-04, after `/develop`'s gate**, on the same footing and for the
same reason as record 0005's section of this name: a word a person reads is not
a build detail, and this surface is not in the comp. Evidence in
[docs/evidence/teach-it-your-words/gate-2026-09-04.md](../evidence/teach-it-your-words/gate-2026-09-04.md),
which covers both rows. All three are **decided on the user's behalf**, and they
join the Multilingual row's line and the caption above as the screen's own
static wording in `src/language/language.js`.

| What | The words | Why these |
|---|---|---|
| The list's label | "Language" | Singular, because one is chosen, even though the section in the rail is Languages. The rail names a subject and a label names a control, and the control here holds one value. |
| The filter's label | "Find a language" | "Find" and not "Search": nothing is being searched, a list already on screen is being narrowed, and a person who types something that matches nothing has not had a search fail, they have narrowed too far. |
| The filter's match count | "{n} languages match." At one: "1 language matches." At none: "No languages match." | It is read out rather than only repainted, because a filter rearranges a list under somebody who may not be able to see it happen. The third is also the wording of the state below. |

**One state was drawn here rather than only worded**, because the gate found
nothing had drawn it: a filter can match nothing, and
`design/design-system.md`'s mandate rule 4 requires an empty state drawn rather
than assumed. When nothing matches, the list is that one sentence and no rows,
and **the filter keeps what was typed**, so a person can correct a letter rather
than start again. It is registered in `design/registry.md`'s `Language filter`.
The chosen language is untouched throughout: a language hidden by a filter is
still the one in force, which is that row's own promise.

## Decided on the user's behalf

Every one of these was `/architect`'s to ask and was decided under the user's
standing instruction for this session. Each is written where it is made, above,
with its reasoning, and each is reversible by an amendment.

1. **One chosen language, not a list with a default marker**, which is a
   correction `design/registry.md` is owed rather than a bigger feature.
2. That the person who switches languages is served by Deepgram's **`multi`
   model as one row in the list**.
3. The list is **every language in Deepgram's `nova-3` table, one row each,
   using the first code**, plus Multilingual: 64 entries.
4. **No regional variants**, except the five that Deepgram's own table names as
   distinct languages.
5. **English stays the default**, and no guess is made from the machine's own
   language.
6. That the language is **read once at the start of each dictation** rather than
   pushed on change, and the one reconnect reuses it.
7. That `dictation.language` stores **what was asked for, never what Deepgram
   detected**.
8. Both sentences a person can read, and their two codes.
9. Both pieces of static wording on the screen: the Multilingual row's
   explanation and the caption beneath the list.
10. **A filter over the list**, because 64 rows cannot be scanned.
11. **Alphabetical order by the name a person reads**, with Multilingual in the
    alphabet rather than pinned above it.
12. **`dir="auto"` on the pill's transcript line and its interim tail**, which
    touches something a person can already see and which AC-11 cannot hold
    without.
13. That the **custom vocabulary is sent in every language with no fallback** if
    a request is refused.
14. That **punctuation stays on for every language** on Deepgram's documentation
    alone.
15. That **no language detection** is used, from the machine or from the audio.
16. The three labels and sentences in "The wording of the screen" above, and the
    no-matches state the gate found undrawn, all settled before any code was
    written: the list's label, the filter's label, the filter's match count in
    all three of its forms, and what the list shows when nothing matches.

## References

- Deepgram, Models and Languages Overview, read 2026-09-04:
  `https://developers.deepgram.com/docs/models-languages-overview`. The source
  for the 63 language rows, their codes, and the ten languages `multi` covers.
- Deepgram, Punctuation, read 2026-09-04:
  `https://developers.deepgram.com/docs/punctuation`. The source for "all
  available languages" on streaming, and silent on what happens otherwise.
- Deepgram, Keyterm Prompting, read 2026-09-04:
  `https://developers.deepgram.com/docs/keyterm`. Read here for the
  monolingual and multilingual support AC-12 leans on.
