// The language screen: which language Deepgram is told to expect (record 0006;
// design/registry.md "Language list", "Multilingual row", "Language filter",
// "Default marker", "Setting error line").
//
// It is the white reading surface of the dashboard, at Settings, Languages.
//
// Two commands and nothing else: `get_transcription_language` and
// `set_transcription_language`. It emits nothing and listens to nothing.
//
// **The rows come from what `get_transcription_language` returned and never
// from a list held here.** What is held here is the wording for each code Rust
// can hand out, which is this project's division everywhere: Rust hands out
// identifiers and this side decides what a person reads. So a code with no
// wording here is not drawn at all, and two guards in
// `src-tauri/src/language/mod.rs` fail the build if the two lists drift apart
// in either direction.
//
// **Neither sentence a person reads on a failure lives here.** Both arrive on
// the command's error, from the one place in Rust that holds them.
//
// **The filter is this screen's own business entirely.** It asks Rust nothing
// and changes nothing: a language hidden by a filter is still the one in force.

const { invoke } = window.__TAURI__.core;

/** What a person reads for each language `get_transcription_language` can hand
 *  out. design/registry.md's `Language list` owns this wording, and it is
 *  Deepgram's own English naming from the table record 0006 read on 2026-09-04.
 *
 *  These are English names and not each language's own, which is a real gap for
 *  exactly the person this feature is for. It was not closed by inventing 63
 *  endonyms from memory, because that would be wording a person reads with no
 *  named source; record 0006's Still open holds it for a real source.
 *
 *  Adding a row here does not add a language: Rust refuses anything that is not
 *  one of its own 64. Removing one would hide a language a person can choose.
 *  Both directions are guarded by tests in Rust. */
const NAME = {
  "multi": "Multilingual",
  "af": "Afrikaans",
  "ar": "Arabic",
  "hy": "Armenian",
  "as": "Assamese",
  "be": "Belarusian",
  "bn": "Bengali",
  "bs": "Bosnian",
  "bg": "Bulgarian",
  "ca": "Catalan",
  "zh-HK": "Chinese (Cantonese, Traditional)",
  "zh": "Chinese (Mandarin, Simplified)",
  "zh-TW": "Chinese (Mandarin, Traditional)",
  "hr": "Croatian",
  "cs": "Czech",
  "da": "Danish",
  "nl": "Dutch",
  "en": "English",
  "et": "Estonian",
  "fi": "Finnish",
  "nl-BE": "Flemish",
  "fr": "French",
  "ka": "Georgian",
  "de": "German",
  "de-CH": "German (Switzerland)",
  "el": "Greek",
  "gu": "Gujarati",
  "he": "Hebrew",
  "hi": "Hindi",
  "hu": "Hungarian",
  "id": "Indonesian",
  "it": "Italian",
  "ja": "Japanese",
  "kn": "Kannada",
  "kk": "Kazakh",
  "ko": "Korean",
  "lv": "Latvian",
  "lt": "Lithuanian",
  "mk": "Macedonian",
  "ms": "Malay",
  "mr": "Marathi",
  "mn": "Mongolian",
  "ne": "Nepali",
  "no": "Norwegian",
  "ps": "Pashto",
  "fa": "Persian",
  "pl": "Polish",
  "pt": "Portuguese",
  "pa": "Punjabi",
  "ro": "Romanian",
  "ru": "Russian",
  "sr": "Serbian",
  "sk": "Slovak",
  "sl": "Slovenian",
  "es": "Spanish",
  "sv": "Swedish",
  "tl": "Tagalog",
  "ta": "Tamil",
  "te": "Telugu",
  "th": "Thai",
  "tr": "Turkish",
  "uk": "Ukrainian",
  "ur": "Urdu",
  "vi": "Vietnamese",
};

/** The one row that cannot explain itself, so it gets a second line
 *  (design/registry.md "Multilingual row"). The ten are Deepgram's own
 *  documented set for its `multi` model, read 2026-09-04, and the wording is
 *  record 0006's. */
const MULTILINGUAL_NOTE =
  "Switches between English, Spanish, French, German, Hindi, Russian, " +
  "Portuguese, Japanese, Italian and Dutch inside one dictation.";

/** The caption beneath the list, in record 0006's words. It says the two things
 *  a person cannot see for themselves: that the choice travels with each
 *  dictation, and that a change waits for the next one. */
const CAPTION =
  "This is the language Deepgram is told to expect. A change applies to your " +
  "next dictation, not to one already running.";

/** Mount the language screen into `root`. Returns an unmount function. */
export function mountLanguageSettings(root) {
  const ctx = { root, disposed: false };
  load(ctx);
  return function unmount() {
    ctx.disposed = true;
  };
}

async function load(ctx) {
  let choice;
  try {
    choice = await invoke("get_transcription_language");
  } catch (err) {
    if (ctx.disposed) return;
    drawReadFailure(ctx, err);
    return;
  }
  if (ctx.disposed) return;
  draw(ctx, choice);
}

/** design/registry.md "Settings, could not be read": one error line and no
 *  control at all. No list, no filter, nothing holding a guessed value. A list
 *  drawn without a chosen language would let a person leave believing a
 *  language is in force that is not. */
function drawReadFailure(ctx, err) {
  const line = errorLine();
  if (!showError(line, err)) {
    console.error("language: the language was refused: " + reasonOf(err));
    ctx.root.replaceChildren();
    return;
  }
  const screen = el("section", "lang");
  screen.append(line);
  ctx.root.replaceChildren(screen);
}

function draw(ctx, choice) {
  const line = errorLine();
  const state = { ctx, line, chosen: choice.chosen, rows: [], filter: "" };

  // Alphabetical by the name a person reads, which only this side can do,
  // because Rust hands out codes and this file holds the names. Multilingual
  // takes its place under M rather than being pinned above a heading: its own
  // row explains what it is wherever it sits, and pinning it would need a
  // grouping component nothing draws. localeCompare so accented names sort
  // where a person expects rather than after Z.
  state.offered = (choice.choices || [])
    .filter((code) => {
      if (NAME[code]) return true;
      // Rust offers a language this build has no wording for. Rust and this
      // file are meant to change together, and a test in Rust fails when they
      // do not, so this is the belt after the braces: showing a person a
      // machine code would be worse than showing them one fewer row.
      console.warn("language: no wording for the language " + code);
      return false;
    })
    .sort((a, b) => NAME[a].localeCompare(NAME[b]));

  const screen = el("section", "lang");
  screen.append(filterGroup(state), listGroup(state), line);
  paintList(state);
  ctx.root.replaceChildren(screen);
}

/* ---- Language filter ------------------------------------------------- */

/** design/registry.md "Language filter": the History `Search field` primitive
 *  on a light settings surface, because 64 rows cannot be scanned. It asks Rust
 *  nothing. */
function filterGroup(state) {
  const group = el("div", "lang__group");

  const label = withText(el("label", "lang__label"), "Find a language");
  label.id = "lang-filter-label";
  label.htmlFor = "lang-filter";

  const field = el("input", "lang__field");
  field.id = "lang-filter";
  field.type = "text";
  field.autocomplete = "off";
  field.spellcheck = false;
  field.setAttribute("aria-describedby", "lang-count");

  // Filtering rearranges a list under somebody who may not be able to see it
  // happen, so how many match is read out rather than only repainted.
  const count = el("p", "lang__count");
  count.id = "lang-count";
  count.setAttribute("aria-live", "polite");

  state.count = count;
  field.addEventListener("input", () => {
    state.filter = field.value.trim().toLowerCase();
    paintList(state);
  });

  group.append(label, field, count);
  return group;
}

/* ---- Language list --------------------------------------------------- */

/** design/registry.md "Language list": one row per language, one radio group,
 *  exactly one chosen. Never a dropdown: dropdown menus are on that list's
 *  not-in-the-registry list, which is why the filter exists instead. */
function listGroup(state) {
  const group = el("div", "lang__group");

  const label = withText(el("span", "lang__label"), "Language");
  label.id = "lang-list-label";

  const caption = withText(el("p", "lang__caption"), CAPTION);
  caption.id = "lang-caption";

  const rows = el("div", "lang__rows");
  rows.setAttribute("role", "radiogroup");
  rows.setAttribute("aria-labelledby", label.id);
  rows.setAttribute("aria-describedby", caption.id);

  state.rowsHost = rows;
  group.append(label, rows, caption);
  return group;
}

/** Draw the rows the filter leaves, and say how many that is. */
function paintList(state) {
  const showing = state.offered.filter((code) => matches(code, state.filter));

  state.count.textContent = countWords(showing.length);

  if (showing.length === 0) {
    // The no-matches state (design/registry.md "Language filter"). One
    // sentence and no rows, and the filter keeps what was typed so a person
    // corrects a letter rather than starting again. The chosen language is
    // untouched: it is still in force and its row returns the moment it
    // matches. The same wording the count uses at zero, deliberately, so a
    // person hears one thing and not two.
    state.rows = [];
    state.rowsHost.replaceChildren(
      withText(el("p", "lang__empty"), countWords(0)),
    );
    return;
  }

  state.rows = showing.map((code) => languageRow(state, code));
  state.rowsHost.replaceChildren(...state.rows);
  paintChosen(state);
}

/** How many languages the filter leaves, in words (record 0006). */
function countWords(n) {
  if (n === 0) return "No languages match.";
  if (n === 1) return "1 language matches.";
  return n + " languages match.";
}

/** Matched on the name a person can see, which is the only string on screen.
 *  The code is deliberately not matched: a person typing "no" means Norwegian,
 *  not every language whose tag happens to hold those letters. */
function matches(code, filter) {
  if (filter === "") return true;
  return NAME[code].toLowerCase().includes(filter);
}

function languageRow(state, code) {
  const row = el("button", "lang__row");
  row.type = "button";
  row.setAttribute("role", "radio");
  row.dataset.language = code;

  const words = el("span", "lang__row-words");
  words.append(withText(el("span", "lang__name"), NAME[code]));
  if (code === "multi") {
    // The one row with a second line, because "Multilingual" names no language
    // a person could check (design/registry.md "Multilingual row").
    words.append(withText(el("span", "lang__note"), MULTILINGUAL_NOTE));
  }

  row.append(words, badge());
  row.addEventListener("click", () => choose(state, code));
  row.addEventListener("keydown", (event) => onArrow(event, state));
  return row;
}

/** The second signal, so the choice never rests on the fill alone
 *  (design/registry.md "Default marker"): the `Mono badge` primitive reading
 *  CHOSEN, the same word and the same badge the hotkey rows use, so one marker
 *  means one thing across the Settings section. Present only on the chosen row,
 *  so it is removed rather than hidden: an empty badge is a shape with no
 *  meaning. `aria-hidden`, because `aria-checked` already says this to a screen
 *  reader and saying it twice is noise. */
function badge() {
  const node = withText(el("span", "lang__badge"), "CHOSEN");
  node.setAttribute("aria-hidden", "true");
  return node;
}

/** Arrow keys move between the rows that are showing. They move focus and do
 *  not choose: a choice is a write that can be refused, and arrowing past a row
 *  should not try to store it. Space and Enter choose, which a `<button>`
 *  already does for both. */
function onArrow(event, state) {
  const keys = ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"];
  if (!keys.includes(event.key)) return;
  event.preventDefault();
  const here = state.rows.indexOf(event.currentTarget);
  if (here < 0 || state.rows.length === 0) return;
  const step = event.key === "ArrowDown" || event.key === "ArrowRight" ? 1 : -1;
  const next = state.rows[(here + step + state.rows.length) % state.rows.length];
  // The roving tab stop follows focus, so tabbing away and back returns to the
  // row the person left rather than jumping to the chosen one.
  for (const row of state.rows) row.tabIndex = row === next ? 0 : -1;
  next.focus();
}

async function choose(state, code) {
  if (code === state.chosen) return;
  clearError(state.line);
  try {
    await invoke("set_transcription_language", { language: code });
  } catch (err) {
    if (state.ctx.disposed) return;
    // The marked language does not move (design/registry.md "Setting error
    // line"), so the screen never shows a setting as changed that is not. The
    // control is its own retry: a person presses another row again.
    if (!showError(state.line, err)) {
      console.error("language: the language was refused: " + reasonOf(err));
    }
    return;
  }
  if (state.ctx.disposed) return;
  // Written, and in force from the next dictation, which needs nothing
  // re-armed: Rust reads the language at the moment the microphone opens. Only
  // now does the marked row move.
  state.chosen = code;
  paintChosen(state);
}

function paintChosen(state) {
  for (const row of state.rows) {
    const chosen = row.dataset.language === state.chosen;
    row.classList.toggle("lang__row--chosen", chosen);
    row.setAttribute("aria-checked", chosen ? "true" : "false");
    // One tab stop for the group, on the chosen row, which is the radio group
    // convention. Arrow keys reach the others.
    row.tabIndex = chosen ? 0 : -1;
    const existing = row.querySelector(".lang__badge");
    if (chosen && !existing) row.append(badge());
    if (!chosen && existing) existing.remove();
  }
  if (!state.rows.some((row) => row.dataset.language === state.chosen)) {
    // The chosen language is filtered out, so every row showing is unchecked
    // and the group has no tab stop. Give it one, on the first row, rather than
    // leave a keyboard user unable to reach the control at all. The chosen
    // language is unaffected: it is still in force and its row returns the
    // moment it matches again.
    if (state.rows[0]) state.rows[0].tabIndex = 0;
  }
}

/* ---- Setting error line ---------------------------------------------- */

/** design/registry.md "Setting error line": a mono code, then one plain
 *  sentence, and no action, because the control is its own retry. Both parts
 *  come from Rust; this builds the shape and nothing else. */
function errorLine() {
  const line = el("div", "lang__error");
  line.hidden = true;
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(el("span", "lang__error-code"), el("p", "lang__error-text"));
  return line;
}

function showError(line, err) {
  const code = err && err.code;
  const message = err && err.message;
  if (!code || !message) return false;
  line.querySelector(".lang__error-code").textContent = code;
  line.querySelector(".lang__error-text").textContent = message;
  line.hidden = false;
  return true;
}

function clearError(line) {
  line.hidden = true;
  line.querySelector(".lang__error-code").textContent = "";
  line.querySelector(".lang__error-text").textContent = "";
}

/** The machine cause, for a log line only. Never shown to a person. */
function reasonOf(err) {
  return (err && err.reason) || String(err);
}

/* ---- Small helpers --------------------------------------------------- */

function el(tag, className) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

function withText(node, str) {
  node.textContent = str;
  return node;
}
