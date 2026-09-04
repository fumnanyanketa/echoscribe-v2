# 0005. Teach it your words

**Status:** In progress
**Date:** 2026-09-04
**Weight:** medium
**Plan row:** 3
**Supersedes:** nothing

**Decided without the user, on their own instruction.** The user was away for
this session and authorised `/architect`, `/canvas`, `/develop` and `/test` to
run end to end on plan rows 3 and 4, with a standing instruction: where a value
has no named source, decide it, take the most reversible option, and write the
reasoning into the record so it can be ratified or reversed later. That is
`/develop`'s gate option 3, and this project has used it three times before.
Every such decision in this record is marked **decided on the user's behalf**
where it is made, and all of them are listed together at the end, under
"Decided on the user's behalf", so none of them has to be hunted for.

## In one line

A Vocabulary section on the settings surface holds the words and names
Deepgram keeps getting wrong, up to a budget this app can prove is inside
Deepgram's own, and every dictation sends that list along with the voice as
Deepgram's `keyterm` so those words are recognised. It buys the one thing
AGENTS.md says the app must do well, accuracy, for the words a general model
cannot know. It costs a list of a person's own words travelling to an outside
service with every dictation, which the screen has to say out loud.

## What this is for

A general speech model has never heard your colleague's name, your company's
product, or the jargon of your trade, so it guesses something close and you fix
it by hand every time. This is where you tell it. You type the word once, and
from the next dictation on Deepgram is told to listen for it.

It is plan row 3's `Done when:` exactly: add a word or name it mis-transcribes,
and the next time you say it, it comes out right.

## Acceptance criteria

- **AC-1**: Settings, Vocabulary shows the words and phrases I have added,
  newest first, and says how much room is left.
- **AC-2**: I type a word or a short phrase, add it, and it appears at the top
  of the list straight away and is still there after closing and reopening the
  app.
- **AC-3**: The next dictation after I add a word uses it. A dictation already
  running when I added it is not changed.
- **AC-4**: Removing a word takes it off the list straight away, the next
  dictation no longer uses it, and it is still gone after a restart.
- **AC-5**: A word or phrase can be up to 30 characters, and the list holds up
  to 400 characters in total. When there is no room for what I typed, the
  screen says so and refuses it. Nothing is ever silently shortened or dropped.
- **AC-6**: Spaces at either end of what I type are trimmed off. I cannot add
  an empty entry.
- **AC-7**: Adding a word that is already in my list, whatever its capital
  letters, is refused and says so. The list never holds the same word twice.
- **AC-8**: Pasting several lines at once is refused with one sentence saying
  it must be one word or phrase. Nothing is added.
- **AC-9**: My list is mine. A second account signing in on the same machine
  sees none of my words, and its dictations do not use them.
- **AC-10**: With nothing added, the screen says the list is empty and what it
  is for, and dictation works exactly as it did before this feature existed.
- **AC-11**: If my list cannot be read when the screen opens, the screen is one
  error line and draws no control at all: no list, no add field, nothing
  holding a guessed value.
- **AC-12**: An add or a remove that cannot be saved leaves the list exactly as
  it was on screen, and says so on the same one error line.
- **AC-13**: The words I add appear on this screen and nowhere else in
  EchoScribe. They are never shown in the pill, never in history, and never in
  a message about something going wrong.
- **AC-14**: A word containing characters that mean something to a web address,
  such as `&` or `=`, changes nothing about what is asked of Deepgram beyond
  adding that word.

## The decision

**Deepgram's `keyterm`, sent on the live stream, and nothing cleverer.** The
model is `nova-3`, fixed by record 0002, and Deepgram's own documentation points
`nova-3` at Keyterm Prompting: up to 100 terms, up to 500 tokens per request,
plain terms with no weights, and it works on streaming as well as on recorded
audio. Checked live on 2026-09-04 at
`https://developers.deepgram.com/docs/keyterm`. The Rust SDK already has it, as
`OptionsBuilder::keyterms`, which serialises to `keyterm=one&keyterm=two` with
proper encoding. Nothing is installed and no new outside address is called: it
is one more parameter on the socket record 0002 already opens.

**The older `keywords` parameter is deliberately not used.** It is the
pre-`nova-3` mechanism, it takes intensifiers, and Deepgram documents it as
superseded by keyterm for this model. Using it would mean either sending an
intensifier this record would have to invent a value for, or sending a
parameter the chosen model does not act on.

**A new feature folder on both sides.** `src-tauri/src/vocabulary/` holds the
table, the rules and the three commands, and `src/vocabulary/` holds the
screen. AGENTS.md forbids adding a feature to a folder that seems related, and
record 0004 set the precedent when the shell opened its own connection to the
same SQLite file beside sign in's and the dictate feature's. This does the same.

**The dictate feature reads the list, and the vocabulary feature never pushes
it.** `dictate` asks `vocabulary` for the terms through one named function, once,
at the moment a dictation starts. Read only, in Rust, and in one direction only.
That is the direction record 0004 already established when the shell read the
dictate feature's error classifier through one named function, and it is why
AC-3 reads the way it does: the list is fixed for the whole of one dictation, so
the one reconnect attempt AC-14 of record 0002 allows sends exactly the terms
the dictation started with, and no event or listener is needed anywhere.

**The budget, and why it is counted in characters.** This is the part of the
decision that took the most care, and it is **decided on the user's behalf**.
Deepgram's limit is 500 tokens per request and it returns an error above that.
An error there would stop dictation, which is the whole product, so this app has
to be certain it can never send too much. It cannot count tokens: the tokeniser
is Deepgram's and this project does not have it. So it counts the one thing that
is a proven upper bound on tokens. **No tokeniser ever produces more tokens than
there are characters**, because a token is one or more characters. Counting each
character of each term as one token, plus one per term for the separator, and
capping that at 400 therefore guarantees the request is inside Deepgram's 500
whatever the script, with 100 tokens of headroom for the model's own overhead.

A character budget is the only rule here that is script independent. The obvious
alternative, a count of words, is not: 40 English names average about 12
characters and are roughly 120 tokens, while 40 Chinese phrases of 30 characters
each are closer to 1200 and would be refused by Deepgram. A count would have
shipped a feature that works for English and breaks dictation for a Mandarin
speaker, which is precisely the person plan row 4 exists for.

Three numbers, and only one of them is ever met by a person:

| Rule | Value | Where it comes from |
|---|---|---|
| The longest one word or phrase | 30 characters | This record, a judgement, **decided on the user's behalf**. Long enough for "Fumnanya Nketa" or a drug name, short enough that one entry cannot eat the budget. Raising it later takes nothing away from anybody, which is why it is set low rather than high. |
| The whole list | 400 characters, each term's characters plus one | Derived from Deepgram's documented 500 tokens, with headroom, by the upper bound argued above. The only rule a person meets in practice. |
| The most terms | 100 | Deepgram's own documented ceiling. Practically unreachable, because 100 terms inside 400 characters means an average of three characters each. It is here so that the app never sends more than Deepgram accepts even if the budget alone would allow it. |

**What a person sees of the budget is one number: how much room is left.** The
screen states it beside the add field and it is Rust's answer, not the screen's
arithmetic. `design/registry.md` owns what that looks like; this record only
requires that AC-1's "says how much room is left" be on screen without a person
having to try an add to find out.

**Nothing is ever silently dropped, and that is why the cap is at the add.** The
alternative, accepting everything and sending as much as fits, was considered
and refused: a person would be looking at a list of words believing they were in
force when some of them were not, and no screen could honestly show which. So an
add that does not fit is refused with a sentence, and the list on screen is
always exactly the list that is sent.

**One place where the app does drop the whole list rather than fail.** If the
stored list somehow exceeds the budget, which the rules above make unreachable
from the screen and leaves only a hand edited database, the dictation runs with
**no** keyterm at all rather than with a shortened list or a failure. Dictation
working with ordinary accuracy beats dictation not working, and a shortened list
would be a silent lie about which words are in force. This is not an acceptance
criterion because a person cannot reach the state to check it; it is a rule in
Rust with a test.

**Every word is treated as hostile input** (AGENTS.md rule 7). A term is text a
person typed or pasted, and it ends up in a web address as a query parameter, so
the two things that matter are that it cannot add a parameter of its own and
that it cannot carry something that is not text. Three guards, in this order:
the term is trimmed and refused if it is empty; it is refused if it holds a line
break, a tab or any other control character; and it reaches Deepgram only
through the SDK's own serialiser, which percent encodes it, proven by that
crate's own tests. AC-14 is the criterion for it and there is a test that a term
reading `&language=de` changes nothing but the term list.

**No term is ever logged.** Not on a failure, not on a refusal, not in a
diagnostic. `set_hotkey` in record 0002 already set this rule for a value
arriving from outside: what it was is not information worth a log line, and a
log is a file. A refusal logs its machine reason and never the text.

**The seven sentences a person can read live in Rust, in this feature, and
nowhere else.** That is record 0002's fourteenth amendment applied to a second
feature: the screen draws the code and the sentence it is handed and does not
know either one, so a wording change reaches a person without a second file
having to agree, and a Rust test reads the screen's source and fails if a copy
appears there. Each sentence is **decided on the user's behalf**.

| State | Code | Sentence |
|---|---|---|
| No room for this word | `VOCABULARY_FULL` | "There is no room for this, so it was not added. Remove a word to make space." |
| The word is too long | `TERM_TOO_LONG` | "A word or phrase can be up to 30 characters." |
| Already in the list | `TERM_ALREADY_ADDED` | "This is already in your list." |
| More than one line | `TERM_NOT_ALLOWED` | "This must be one word or phrase on a single line." |
| The add would not save | `TERM_NOT_SAVED` | "This could not be saved, so your list is unchanged." |
| The remove would not save | `TERM_NOT_REMOVED` | "This could not be removed, so your list is unchanged." |
| The list would not read | `VOCABULARY_NOT_READ` | "Your list could not be read, so none of it is shown." |

**Two refusals carry no sentence at all**, exactly as record 0002's settings
commands do: nobody signed in, and the core not being up. Neither can happen on
a screen that only exists while somebody is signed in, and both only happen
while Rust is already closing the window the screen lives in. A screen with
nothing to draw draws nothing rather than inventing a sentence for a state
nobody designed. An empty term is a third: the add control is not usable while
the field is empty or holds only spaces, so Rust's own refusal of an empty term
is a second guard on an unreachable state and carries no sentence either.

**One caption says the thing a person cannot see for themselves**, and it is
**decided on the user's behalf**: "Each word here is sent to Deepgram with your
voice, so it knows to listen for it. Nothing is stored outside this machine."
AGENTS.md makes the microphone's visibility a hard limit for the same reason
this caption exists: a person is owed the knowledge that their own words leave
the machine. The second sentence is the true and reassuring half, and it is
careful: the list is stored locally and nowhere else, and each term travels as
part of a request rather than being kept by anybody. It claims nothing about
what Deepgram does with a request, because this project does not control that
and a sentence here could not keep the promise. It is the screen's own static
wording, in `src/vocabulary/vocabulary.js`, and not Rust's, the same division
record 0002's two captions are on: nothing in Rust produces it and no second
screen shows it.

**The rail gains a third destination, `settings.vocabulary`, and record 0004 is
amended for it.** That record says in as many words that each of plan rows 3, 4
and 5 amends it to add its own item as it lands, so this is the amendment, not a
new decision. Its place in the rail is the comp's order, Dictation, Languages,
Vocabulary, Transcription, which `design/registry.md`'s `Section sub-nav` row
already names. Nothing else about record 0004 changes: no new command, no new
event, and `dashboard.json` is untouched.

**Nothing on the out of scope list is built, and four near misses are refused
by name.** No pronunciation or phonetic spelling: Deepgram's keyterm takes plain
terms and nothing else. No "when I say X, write Y" replacement rule: that is
voice driven editing of a transcript, which AGENTS.md puts out of scope, and it
would change words a person actually said. No importing a list from a file:
nothing asks for it, and it is the one route by which a list larger than the
budget could arrive. No editing a word in place: removing and adding does it,
with one control fewer to design and one write fewer to get wrong.

**Adding a word changes nothing that has already happened.** History holds what
was typed, and a term added today does not rewrite a dictation from yesterday.
Named here because a person could reasonably expect otherwise.

## What else was considered

| Option | Why not |
|---|---|
| The older `keywords` parameter with an intensifier | It is the pre-`nova-3` mechanism and Deepgram documents keyterm as the one for this model. It also takes a boost number this record would have had to invent, with no way to tell whether the value chosen was helping. |
| A count of words as the cap, say 40, with a length limit each | Simple to say and simple to show, and it was the first answer this record reached. It is not script independent: 40 entries of 30 characters is about 400 tokens in English and about 1200 in Chinese, so it would work for English and break dictation for a Mandarin speaker. The character budget costs one slightly odd number on screen and is provably safe for every language Deepgram supports. |
| No cap, sending as much of the list as fits | The person is then looking at words they believe are in force when some are not, and there is no honest way to show which. It also makes the fix for "it still gets my name wrong" invisible: the name might simply not have been sent. |
| Storing the terms in record 0002's `dictation_setting` table | It is one row per account and this is many rows per account, so it does not fit the table's shape. It would also put a new feature's data inside another feature's schema, which AGENTS.md's folder rule exists to prevent, and record 0004 already set the opposite precedent with its own table and its own connection. |
| The vocabulary feature pushing its list to the dictate feature on change | Needs an event, a listener and a copy of the list held somewhere between dictations, and it buys nothing: the list is only ever wanted at the instant a dictation starts, which is exactly when a read is cheapest and freshest. Reading at start is also what makes AC-3 a simple promise rather than a race. |
| Sending the terms from the interface with each dictation | Refused outright. It would mean the interface holding and transmitting a list Rust is responsible for, and AGENTS.md keeps decisions in Rust and the interface thin. The interface never touches the socket. |
| A search field over the list | Not drawn for this surface in `design/registry.md`, and a list bounded at 400 characters is short enough to read at a glance. Named in Still open so that it is a decision rather than an omission. |

## Data model

One new table, in the same SQLite file as everything else, because AGENTS.md
holds all persistent data in one file on the person's own machine. The `account`
table is the sign in feature's, plan row 1, and is referenced here.

| Table | Key | Fields | Relationship |
|---|---|---|---|
| `vocabulary_term` | `id`, auto | `account_id` text, required. `term` text, required, 1 to 30 characters. `added_at` text, required, UTC. | One account has many terms |

Rules that must always hold:

- `account_id` is never empty. Nothing is written until we know whose it is.
  This applies to custom words exactly as AGENTS.md's data rules say it does.
- `term` is stored exactly as the person typed it, capital letters included,
  because Deepgram's own guidance is to preserve capitalisation for proper nouns
  and lower case for common ones. It is stored after trimming the ends and
  never otherwise altered.
- No two rows for one account hold the same `term` ignoring case. Enforced by a
  unique index using SQLite's `NOCASE` collation as well as by the check in
  Rust, so the schema is the second guard and not the first.
- `term` never holds a line break, a tab or any other control character. Checked
  in Rust before the write; a row read back holding one is dropped from the list
  handed to Deepgram rather than sent, on the same "not an instruction" rule
  record 0002 applies to a stored hotkey it cannot understand.
- An account's terms total no more than 400 characters, counting each term's
  characters plus one per term, and number no more than 100. Both are checked in
  Rust before an insert, because SQLite cannot express a per account budget.
- `vocabulary_term` is indexed on account and added time, newest first, because
  that is the only order the screen reads it in.
- A term is kept until the person removes it. Nothing expires on its own.
- Deleting an account deletes all of its terms. Record 0002 lists three rows and
  a Credential Manager entry and record 0004 adds a fourth row; this is a fifth
  thing, and it is many rows rather than one.
- No audio, no transcript and no secret of any kind is ever in this table. It
  holds words a person typed.

One migration, creating one table and its two indexes.

## Value sourcing

| Value | Needed by | Comes from |
|---|---|---|
| The signed in account id | AC-2, AC-4, AC-9 | The Clerk session held in the Rust core, per record 0003. Never passed in from the interface, and no command here takes one. |
| The list of words | AC-1, AC-3 | `vocabulary_term` rows for the account, newest first. |
| The order the list is shown in | AC-1 | This record, **decided on the user's behalf**: newest first, from `added_at`. The word you just added is the one you are looking for, so it is at the top where an add can be seen to have worked. Alphabetical was the alternative and is better for answering "did I already add this", which AC-7's refusal answers anyway, out loud, at the moment it matters. |
| The longest one term may be | AC-5, AC-6 | This record. 30 characters, after trimming. Fixed, not a setting. |
| How much room the whole list has | AC-1, AC-5 | This record. 400 characters, counting each term's characters plus one per term. Derived from Deepgram's documented 500 token limit by the upper bound in The decision. Fixed, not a setting. |
| The most terms there may be | AC-5 | Deepgram's own documented ceiling of 100, checked 2026-09-04. Practically unreachable inside the character budget. |
| How much room is left, as shown on screen | AC-1 | Rust, on `get_vocabulary`, computed from the same one place the budget is enforced. The screen does no arithmetic of its own, so what it shows and what an add will accept cannot disagree. |
| Whether a word is already in the list | AC-7 | The stored terms for that account, compared ignoring case. Rust decides, and the unique index refuses it a second time. |
| Which characters a term may not hold | AC-8, AC-14 | This record: no line break, no tab, no other control character. Everything else is allowed, including letters of any script and punctuation, because a name can hold an apostrophe or a hyphen and refusing those would be refusing the feature's own purpose. |
| How a term reaches Deepgram safely | AC-14 | The Deepgram SDK's own query serialiser, which percent encodes each term, proven by that crate's own tests. Nothing in this project builds a web address by joining strings. |
| What is asked of Deepgram on the stream | AC-3 | Record 0002, as amended by its seventeenth amendment: `nova-3`, punctuation on, interim results on, the chosen language, and this feature's terms as `keyterm`. |
| When the terms are read for a dictation | AC-3 | This record. Once, in Rust, at the moment the microphone opens, through one named function in this feature. The same terms are used by the one reconnect attempt, so a dictation cannot change its list halfway through. |
| What happens if the terms cannot be read at that moment | AC-10 | This record. The dictation runs with no keyterm at all and nothing on screen changes. Accuracy falls back to what it was before this feature existed, which is exactly AC-10's promise, and dictation never fails because of a settings read. |
| What happens if the stored list exceeds the budget | AC-5 | This record. The dictation runs with no keyterm at all, never a shortened list. Unreachable from the screen; it exists for a hand edited database. |
| The sentence for each refusal | AC-5, AC-7, AC-8, AC-11, AC-12 | This record, the seven sentence table in The decision. Fixed wording, held in one place in Rust, and never carrying anything read off the term. |
| Which refusals carry no sentence | AC-11, AC-12 | This record: nobody signed in, the core not up, and an empty term. All three are unreachable from this screen, and the precedent is record 0002's fourteenth amendment. |
| The caption saying the words leave the machine | AC-13 | This record, the sentence in The decision. The screen's own static wording, in `src/vocabulary/vocabulary.js`. |
| What a person reads for the rail item | AC-1 | `design/registry.md`, which owns wording, exactly as it does for the three rail items that exist. Rust hands out the identifier `settings.vocabulary` and nothing else. |
| Which sections the rail holds | AC-1 | Record 0004's fixed list in `src-tauri/src/shell/rail.rs`, as amended by this record to hold `settings.vocabulary`. |

## Interface surface

Three commands, all in the new feature, all on the same terms as every other
command in this project: the interface asks and Rust decides, none takes an
account id because Rust knows it from the session, and all refuse when nobody
is signed in.

- `get_vocabulary()` returns the terms for the account, newest first, each with
  the identifier the remove needs, together with how much room is left and the
  longest a term may be. It hands out the limits rather than letting the screen
  hold copies, for the same reason `get_hotkey` hands out the two hotkeys: what
  is allowed is Rust's answer, so the screen cannot offer something Rust would
  refuse, and the room shown can never disagree with the room there is.
- `add_vocabulary_term(term)` trims it, refuses it for one of the four named
  reasons or writes it, and returns the whole list again in the same shape
  `get_vocabulary` returns. Returning the list rather than the one new term is
  deliberate: the room left has changed, and one answer keeps the screen from
  drawing a list and a budget that came from two different moments.
- `remove_vocabulary_term(id)` removes that one term for that account and
  returns the list in the same shape, for the same reason. An identifier that is
  not this account's row removes nothing and is not an error, because there is
  nothing to tell a person and nothing went wrong.

No event, in either direction. Nothing outside this feature needs to know that
the list changed: the dictate feature reads it at the start of each dictation,
which is the only moment it matters.

Nothing in `src-tauri/capabilities/` is added or widened. The screen listens to
nothing and emits nothing, and record 0004's closed Still open established that
Tauri grants no per command permission for this application's own commands: what
protects them is the `not_signed_in` refusal in Rust, which all three make.

One function this feature exposes to the rest of the Rust core, and it is not a
command:

- `terms_for_dictation(&app) -> Vec<String>`, read only, called by the dictate
  feature at the moment the microphone opens. Empty on anything at all going
  wrong, including nobody being signed in, so a failure here can never stop a
  dictation. It is the one door between the two features and there is deliberately
  no second.

Errors that matter and must each read differently: no room, too long, already
there, more than one line, the add not saving, the remove not saving, the list
not reading, and not signed in. The first seven have the sentences in The
decision. The last carries none, for the reason given there.

## Risk

The plan row is medium weight, but this feature stores words a person typed and
sends them to an outside service on every dictation, so the three questions are
answered rather than skipped. AGENTS.md's decision rules put both of those in
the ask-first column, and this record is the answer.

**The worst thing a malicious person could do here.** Three things, and all
three are treated as real.

1. **Turn a term into an extra Deepgram parameter.** A term goes into the query
   of the streaming address. A term reading `&language=de` or
   `&redact=pci` that reached the address unencoded would change what is asked
   of Deepgram, silently, on every dictation. Refused three ways: nothing here
   builds an address by joining strings, the SDK percent encodes each term, and
   AC-14 has a test that names this exact string.
2. **Use the list as a place to keep something else.** A term is 30 characters
   and there is room for 400, so the list is not a useful hiding place, but it is
   still storage that travels. The character rules refuse anything that is not a
   single line of text, the terms are only ever read by this feature and the one
   named function, and they are never logged. Nothing else in the app can read
   them.
3. **Learn what somebody dictates about by reading their list.** The list is on
   the person's own machine, in the same SQLite file as everything else, tied to
   their account, and a second account on the machine cannot see it, which is
   AC-9. That is the same protection record 0002's AC-18 gives history, and it
   is not stronger: anybody with the machine and the file has the words. Said
   plainly rather than dressed up, because the honest answer is that this is
   local storage and not a vault.

**What we are storing that we must protect.** Words a person chose to type,
which is more personal than it looks: a list of custom vocabulary can hold
colleagues' names, a client list, a diagnosis, or the internal name of an
unannounced product. It is personal data. It lives in one SQLite file on the
person's own machine, against exactly one account, and nothing about it is
written anywhere else. It is never in a log, never in an error message, never in
the pill, and never in history.

**What the system should refuse to do, even if asked nicely.**

- Send a term anywhere but the Deepgram streaming request. There is no second
  destination and no command that returns the list to anything but this screen.
- Log a term, on any path, including a failure.
- Accept a list from anywhere but a person typing into this one screen. No file
  import, no command that takes more than one term at a time, and no way for the
  interface to hand the list to the socket itself.
- Accept more than the budget, or send more than Deepgram's documented limits.
- Show a person a list that is not the list being sent.

## Build plan

Three milestones, each leaving the project working.

1. **The store and the rules, with no screen.** The table, its two indexes, the
   trim, the character check, the case insensitive duplicate check, the budget
   arithmetic and the three commands, with their seven sentences. Nothing calls
   them yet, so nothing a person can see changes. Testable in full on its own,
   which is where most of this feature's correctness lives.
2. **The terms reaching Deepgram.** `terms_for_dictation` in this feature, and
   record 0002's `transcribe.rs` sending them as `keyterm` on both the first
   connection and the one reconnect. Proves AC-3, AC-10 and AC-14. At the end of
   this milestone the feature works and has no screen: a term added by hand to
   the database is used, which is how it can be checked before there is anything
   to click.
3. **The screen, and the rail item.** The Vocabulary surface, its add field, its
   list with a remove on each row, its empty state, its read failure state, its
   room left, its caption, and record 0004's rail gaining `settings.vocabulary`.

## What this makes harder

- **Deepgram's token limit is now this project's to own.** The 400 character
  budget is only safe while Deepgram's limit is 500 tokens. If that number
  changes, or if a future model counts differently, this record's arithmetic has
  to be redone rather than nudged. Named so that the next person does not read
  400 as arbitrary and raise it.
- **Raising the caps later is cheap; lowering them is not.** A lower cap takes
  words away from somebody who is relying on them, and this app has no way to
  tell them which ones went. That asymmetry is why every number here is set at
  the cautious end.
- **The rail's fixed list changes for the first time.** Record 0004's AC-2, that
  every item in the rail opens a screen, now covers a third destination, and the
  same is true again the moment plan row 4 lands. Two records amending one list
  in one session is exactly the kind of thing that goes wrong, so both
  amendments name each other.
- **Every dictation's request is slightly larger.** At most 400 characters more,
  once, on the handshake. Negligible, and named rather than unmeasured.
- **A person may reasonably expect this to fix the past.** It does not. History
  holds the words that were typed, and there is nowhere honest to apply a term
  retroactively.
- **The list is one more thing tied to an account.** A person with two accounts
  on one machine keeps two lists and will have to teach both, which AC-9 makes
  correct rather than convenient.

## Still open

- **Whether keyterm actually fixes the words it is given, and how often.**
  Deepgram claims it and this project has not measured it. It needs a live
  sitting: a name the model reliably gets wrong, said and mis transcribed, added
  to the list, and said again. That is plan row 3's own `Done when:` line and it
  belongs to `/check verify`. Nothing in this record depends on the answer, but
  the feature's whole value does, so it is the first thing to prove.
- **Whether 30 characters and 400 characters are the right numbers.** Both are
  judgements at the cautious end of a bound, not measurements. What would settle
  them is a person using the feature for a week and saying whether they ran out.
- **Whether the list needs a search field.** Not drawn for this surface, and a
  400 character list is short. If it grows past what a person can scan, that is
  a `/canvas` row and a `/architect` amendment, not a quick addition.
- **Whether a term should be editable in place.** Remove and add covers it
  today. If a person hits a typo in a long term often enough to complain, this
  is where the answer goes.
- **How this behaves in the multilingual case.** Deepgram documents keyterm as
  working for `nova-3` both monolingual and multilingual, checked 2026-09-04,
  and record 0006 makes the language a choice. A term in one script with the
  language set to another is a state neither record has proved. It cannot break
  dictation, because the request is inside every documented limit either way, so
  the worst case is a term that does not help.

## The wording of the screen

**Added 2026-09-04, after `/develop`'s gate.** The gate found seven labels and
sentences on this screen with no named source, evidence in
[docs/evidence/teach-it-your-words/gate-2026-09-04.md](../evidence/teach-it-your-words/gate-2026-09-04.md),
and they are here rather than left to the build for the reason
`design/registry.md`'s `Setting label` row already gives: a word a person reads
is not a build detail, and this surface is not in the comp, so there was no comp
to take one from. All seven are **decided on the user's behalf**. They are the
screen's own static wording, in `src/vocabulary/vocabulary.js`, and not Rust's,
the same division the caption above is on and the opposite of the seven refusal
sentences, which are Rust's.

| What | The words | Why these |
|---|---|---|
| The add field's label | "Add a word or phrase" | It says both halves of what may go in, a single word or a short phrase, which is the one thing about this field a person cannot guess from looking at it. |
| Its action | "Add" | The verb, alone. Nothing is being saved, submitted or confirmed, and a longer label would claim more than one row in a list is worth. |
| Whether it has a placeholder | **None.** | It has a real label directly above it. A placeholder repeating that label is noise, and it vanishes the moment somebody types, which is exactly when a person still wants the label. |
| The list's label | "Your words" | It says whose they are, which is the whole of AC-9 said quietly, and it separates the list from the field above it without a heading, which this surface deliberately does not have. |
| Remove, on each row | "Remove" | The same verb `Secret field` uses for the same act, so one word means one thing across the Settings section. It is not "Delete": nothing is being destroyed that cannot be typed again in seconds, and `Term row` records that difference. |
| The room left line | "Room for {n} more characters." At nothing left: "No room left. Remove a word to make space." | The number is the only fact a person can act on. The second sentence arrives only when it is true and says the one thing to do about it, which is why the line changing is one of `Term room left`'s two signals. |
| The empty state's paragraph | "Nothing added yet. The words to add here are the ones Deepgram keeps getting wrong: names, places, and the jargon of your work. Add one above and your next dictation will listen for it." | Three sentences doing three jobs: what has not happened, what the list is for with examples rather than an abstraction, and where to go next, which is the field already on screen. It offers no action of its own, which `Vocabulary empty state` registers as a deliberate deviation from the design system's empty state rule. |

## Decided on the user's behalf

Every one of these was `/architect`'s to ask and was decided under the user's
standing instruction for this session. Each is written where it is made, above,
with its reasoning. Each is reversible by an amendment and none of them is load
bearing for another decision.

1. The longest one word or phrase may be: **30 characters**.
2. The whole list's budget: **400 characters**, counting each term's characters
   plus one per term, derived from Deepgram's 500 tokens.
3. Counting the budget in **characters rather than words**, so that it is safe
   for every script Deepgram supports.
4. The order the list is shown in: **newest first**.
5. All seven sentences a person can read, and their seven codes.
6. The caption saying the words are sent to Deepgram with each dictation.
7. That an add which does not fit is **refused**, rather than the list being
   sent shortened.
8. That a stored list somehow over budget sends **no** terms rather than some.
9. That there is no in place edit of a term, no file import, no pronunciation or
   phonetic field, and no replacement rule.
10. That the terms are read **once at the start of each dictation** rather than
    pushed on change, and that the one reconnect reuses them.
11. That a term is stored with its capital letters as typed, while a duplicate
    is judged ignoring case.
12. All seven labels and sentences in "The wording of the screen" above, found
    by `/develop`'s gate on the same day and settled before any code was
    written: the add field's label, its action, that it has no placeholder, the
    list's label, Remove's label, the room left line in both its states, and the
    empty state's paragraph.

## References

- Deepgram, Keyterm Prompting, read 2026-09-04:
  `https://developers.deepgram.com/docs/keyterm`. The source for up to 100
  terms, 500 tokens per request, plain terms with no intensifiers, streaming
  support, and the `keyterm=one&keyterm=two` form.
- Deepgram, Models and Languages Overview, read 2026-09-04:
  `https://developers.deepgram.com/docs/models-languages-overview`. Read here
  for `nova-3`'s keyterm support; record 0006 reads it for the language list.
