// The history screen: what you have said before (record 0007;
// design/registry.md "Search field", "Result count", "Dictation row",
// "Language tag", "Older dictations action", "History empty state",
// "History, could not be read", "Setting error line").
//
// It is the white reading surface of the dashboard, at History, and it is where
// the dashboard lands. Not a dark interruption: nothing here has gone wrong.
//
// One command and nothing else, `get_history`, plus the dictate feature's
// existing `get_hotkey` for the empty state's keys alone. It emits nothing and
// listens to nothing.
//
// **Every string from a dictation reaches this page as textContent and never as
// markup.** A transcription is text from outside the program (AGENTS.md rule 7)
// and this is the first screen in EchoScribe that renders a whole one. Set as
// HTML, a dictation reading `<img src=x onerror=...>` would run script inside
// the dashboard's web view, which can invoke every command this app has. A Rust
// guard in src-tauri/src/history/mod.rs reads this file and fails the build if
// any of the four markup-setting properties appears here, which is why this
// comment describes them rather than naming one: the guard is a plain substring
// search, and a comment explaining the rule would otherwise break it.
//
// **No sentence a person reads on a failure lives here.** The code and the
// sentence arrive on the command's error, from the one place in Rust that holds
// them. The same guard keeps this file from growing a copy.
//
// **Nothing on this screen counts.** How many match and whether there are older
// ones are Rust's answers, from the same read that produced the rows, so the
// number beside the search and the list beneath it can never come from two
// different moments.

const { invoke } = window.__TAURI__.core;

/** The empty state's paragraph, which is the second and third of the four
 *  things design/design-system.md's empty state rule asks for: where the words
 *  will go, and that they stay on this machine. Record 0007's wording. The
 *  second sentence claims only what this app controls, the same care record
 *  0005's caption takes: nothing about a dictation is stored anywhere but here,
 *  and nothing is claimed about the service that transcribed it. */
const EMPTY_PARAGRAPH =
  "Press the hotkey anywhere on this machine, speak, and the words appear " +
  "where your cursor already is. What you say is saved here, on this machine, " +
  "and nowhere else.";

/** What a person reads for each hotkey design/registry.md's `Hotkey choice`
 *  allows. The same division as the rail's wording and the hotkey rows on the
 *  settings surface: Rust hands out the stored value and this side decides what
 *  a person reads. A value with no wording here draws no keys at all rather
 *  than putting a machine identifier in front of somebody. */
const HOTKEY = {
  double_tap_ctrl: { key: "Ctrl", words: "Double tap Ctrl" },
  double_tap_alt: { key: "Alt", words: "Double tap Alt" },
};

/** Mount the history screen into `root`. `go` moves the rail, and the empty
 *  state's one action is the only thing that uses it. Returns an unmount
 *  function. The read runs after this returns, so a second mount arriving first
 *  cannot be painted over by a first one finishing late. */
export function mountHistory(root, options) {
  const ctx = {
    root,
    go: (options && options.go) || null,
    disposed: false,
    // The query the newest page on screen was asked with. A new search throws
    // away every page already loaded and starts again (record 0007, answered at
    // the gate): a search is a new question, and appending to the answer to a
    // different one would put rows on screen that no single query produced.
    query: "",
    // Rises with every search so a slow answer to an old question cannot paint
    // over a fast answer to a new one.
    generation: 0,
    view: null,
    rows: [],
  };
  load(ctx);
  return function unmount() {
    ctx.disposed = true;
  };
}

/* ---- Reading --------------------------------------------------------- */

/** The first read, which is the only one that builds the screen. Everything
 *  after it repaints the list and the count in place, because rebuilding the
 *  screen would replace the search field on every keystroke and take a person's
 *  focus and their cursor with it. */
async function load(ctx) {
  const generation = ++ctx.generation;
  let view;
  try {
    view = await ask(ctx, null);
  } catch (err) {
    if (ctx.disposed || generation !== ctx.generation) return;
    drawReadFailure(ctx, err);
    return;
  }
  if (ctx.disposed || generation !== ctx.generation) return;
  ctx.view = view;
  ctx.rows = view.rows || [];
  draw(ctx);
}

/** One read, for the current query, from the start or from a cursor. The query
 *  and the cursor always travel together, so pressing for older rows cannot
 *  widen the search. */
function ask(ctx, cursor) {
  return invoke("get_history", {
    query: ctx.query === "" ? null : ctx.query,
    beforeStartedAt: cursor ? cursor.started_at : null,
    beforeId: cursor ? cursor.id : null,
  });
}

/** design/registry.md "History, could not be read": one error line and no
 *  control at all. Not the list, not the search field, not the count. A search
 *  field over a list that could not be read invites a person to search a
 *  history the app cannot see, and the empty answer would be indistinguishable
 *  from an empty history. */
function drawReadFailure(ctx, err) {
  const line = errorLine();
  if (!showError(line, err)) {
    // The refusal has no line, which means nobody is signed in or the core is
    // not up. Neither can happen on a screen only reachable while signed in,
    // and if one does then Rust is already closing this window. Say so where a
    // developer sees it and draw nothing. Same reasoning as dashboard.js.
    console.error("history: the history was refused: " + reasonOf(err));
    ctx.root.replaceChildren();
    return;
  }
  const screen = el("section", "history");
  screen.append(line);
  ctx.root.replaceChildren(screen);
}

function draw(ctx) {
  const state = { ctx, line: errorLine() };
  const screen = el("section", "history");

  // A history with nothing in it draws no search field at all (record 0007,
  // answered at the gate). It is not the no-matches state: an empty history is
  // a person who has not dictated, and a search matching nothing is a person
  // whose dictations are all still there. The way out of an empty history is
  // the hotkey, which is what the empty state shows, so a search field here
  // would be a control whose every answer is already known.
  const neverDictated = ctx.query === "" && ctx.rows.length === 0;

  if (!neverDictated) {
    state.search = searchGroup(state);
    screen.append(state.search.group);
  }
  screen.append(state.line);

  state.list = el("div", "history__list-group");
  screen.append(state.list);

  ctx.root.replaceChildren(screen);
  paintList(state);

  if (neverDictated) mountEmptyState(state);
}

/* ---- Search field, and the result count ------------------------------ */

/** design/registry.md "Search field" and "Result count". A real search over
 *  stored text, unlike the language filter, which narrows rows already on
 *  screen. It sits at the top of the surface and stays there while the list
 *  scrolls beneath it. */
function searchGroup(state) {
  const group = el("div", "history__search-group");

  const label = withText(el("label", "history__label"), "Search your history");
  label.id = "history-search-label";
  label.htmlFor = "history-search-field";

  const field = el("input", "history__field");
  field.id = "history-search-field";
  field.type = "search";
  field.autocomplete = "off";
  field.spellcheck = false;
  // The browser lays the field out from what is in it, so a person searching in
  // their own script sees it the right way round as they type. Same mechanism
  // as the pill's transcript line and a dictation row: nothing declares a
  // direction and no list of right to left languages exists anywhere.
  field.dir = "auto";
  // It has a real label directly above it, so a placeholder repeating that
  // label would be noise that vanishes exactly when a person still wants it.
  field.value = state.ctx.query;
  field.addEventListener("input", () => search(state, field.value));

  // The count is read out and not only repainted, because a search rearranges
  // the list under somebody who may not be able to see it happen, and here the
  // rows that arrive were not on screen a moment ago.
  const count = el("p", "history__count");
  count.setAttribute("aria-live", "polite");
  count.setAttribute("aria-atomic", "true");

  group.append(label, field, count);
  state.count = count;
  state.field = field;
  paintCount(state);
  return { group, field, count };
}

/** Three forms, which are the language filter's three unchanged in shape so one
 *  pattern means one thing across the app. Shown only while something is typed:
 *  with an empty search the number would be a count of everything a person has
 *  ever dictated, which is not a result. */
function paintCount(state) {
  // No count element means the empty state, which draws no search field, so
  // there is no query for a count to be about.
  if (!state.count) return;
  const view = state.ctx.view;
  if (state.ctx.query === "" || !view) {
    state.count.textContent = "";
    state.count.hidden = true;
    return;
  }
  const n = view.total;
  state.count.hidden = false;
  state.count.textContent =
    n === 0
      ? "No dictations match."
      : n === 1
        ? "1 dictation matches."
        : n + " dictations match.";
}

/** A new search. It throws away every page already loaded and starts again at
 *  the newest one (record 0007, answered at the gate): a search is a new
 *  question, and appending to the answer to a different one would put rows on
 *  screen that no single query produced.
 *
 *  **The screen is not rebuilt**, only the list and the count. The search field
 *  is the thing a person is typing into, and replacing it would take their
 *  focus and their cursor on every keystroke. Nothing else about the screen can
 *  change from here either: the field exists only when the account has at least
 *  one dictation, and a search cannot change that. */
async function search(state, value) {
  const ctx = state.ctx;
  // A query of nothing but spaces is no query at all (record 0007, answered at
  // the gate). Anything else is used exactly as it was typed, because a person
  // searching for "the " may well mean the trailing space.
  ctx.query = value.trim() === "" ? "" : value;
  clearError(state.line);

  // Rises with every search, so a slow answer to an old question cannot paint
  // over a fast answer to a new one. Typing is faster than a read, so this is
  // ordinary rather than a corner case.
  const generation = ++ctx.generation;
  let view;
  try {
    view = await ask(ctx, null);
  } catch (err) {
    if (ctx.disposed || generation !== ctx.generation) return;
    // The rows already on screen do not move (design/registry.md "Setting error
    // line"), and what was typed stays in the field. This is not the
    // "could not be read" state: that one is for a surface that never opened,
    // and here a person is looking at rows that are still true.
    if (!showError(state.line, err)) {
      console.error("history: a search was refused: " + reasonOf(err));
    }
    return;
  }
  if (ctx.disposed || generation !== ctx.generation) return;

  ctx.view = view;
  ctx.rows = view.rows || [];
  paintList(state);
  paintCount(state);
}

/* ---- The list -------------------------------------------------------- */

/** design/registry.md "Dictation row" and "Older dictations action". Newest
 *  first, which is the only order record 0002 indexed for. */
function paintList(state) {
  const ctx = state.ctx;
  const group = state.list;
  group.replaceChildren();

  if (ctx.rows.length === 0) {
    if (ctx.query !== "") {
      // design/registry.md "Search field", its no-matches state: one sentence
      // and no rows, and the field keeps what was typed so a person corrects a
      // letter rather than starting again. The same wording as the count at
      // zero, deliberately, so a person hears one thing and not two.
      group.append(withText(el("p", "history__none"), "No dictations match."));
    }
    return;
  }

  const list = el("ul", "history__list");
  if (state.field) {
    list.setAttribute("aria-labelledby", "history-search-label");
  }
  for (const row of ctx.rows) list.append(dictationRow(row));
  group.append(list);

  if (ctx.view && ctx.view.has_older) group.append(olderAction(state));
}

function dictationRow(row) {
  const item = el("li", "history__row");

  // The whole transcript, wrapped, never truncated and never collapsed:
  // truncating hides the one thing this screen exists to show, and an expand
  // control is a component nothing draws. It is selectable, which is not
  // decoration: it is the whole of how a person gets a transcript out of
  // EchoScribe today (record 0007 AC-13).
  const text = withText(el("p", "history__text"), row.text);
  // The dictation reads the way it was said, whatever the script.
  text.dir = "auto";

  const meta = el("p", "history__meta");
  meta.append(
    withText(el("span", "history__when"), whenOf(row.started_at)),
    withText(el("span", "history__how-long"), howLongOf(row.duration_ms)),
    // design/registry.md "Language tag": mono, a literal code, on every row
    // including English ones. The code the dictation was asked with, never
    // what Deepgram detected.
    withText(el("span", "history__language"), row.language),
  );

  item.append(text, meta);
  return item;
}

/** design/registry.md "Older dictations action": the Secondary button
 *  primitive, drawn only when Rust says there are older ones. It adds rows
 *  below the ones already there and keeps the search and the place. */
function olderAction(state) {
  const button = withText(
    el("button", "history__older"),
    "Show older dictations",
  );
  button.type = "button";
  button.addEventListener("click", () => older(state, button));
  return button;
}

async function older(state, button) {
  const ctx = state.ctx;
  const last = ctx.rows[ctx.rows.length - 1];
  if (!last) return;
  clearError(state.line);
  // The button is the one thing that must not be pressed twice while an answer
  // is in flight: two answers would append the same page twice. There is no
  // busy state to draw, per design/registry.md, so it simply stops answering.
  button.disabled = true;
  const generation = ctx.generation;

  let view;
  try {
    view = await ask(ctx, last);
  } catch (err) {
    if (ctx.disposed || generation !== ctx.generation) return;
    button.disabled = false;
    // The rows already on screen do not move (design/registry.md "Setting
    // error line"). The button is its own retry.
    if (!showError(state.line, err)) {
      console.error("history: older dictations were refused: " + reasonOf(err));
    }
    return;
  }
  if (ctx.disposed || generation !== ctx.generation) return;

  ctx.view = view;
  ctx.rows = ctx.rows.concat(view.rows || []);
  paintList(state);
  paintCount(state);
}

/* ---- History empty state --------------------------------------------- */

/** design/registry.md "History empty state", and the one state
 *  design/design-system.md's empty state rule was written for. Four parts in
 *  order: what has not happened, the hotkey as keys, one paragraph, and exactly
 *  one action.
 *
 *  The keys arrive from the dictate feature's own `get_hotkey`, and only here,
 *  because this is the only moment they are drawn. If that call fails the keys
 *  are simply absent and the rest stands: keys are not a control, so their
 *  absence is an omission rather than a guessed value. */
function mountEmptyState(state) {
  const ctx = state.ctx;
  const empty = el("div", "history__empty");

  empty.append(withText(el("p", "history__empty-line"), "Nothing dictated yet."));
  const keys = el("p", "history__empty-keys");
  empty.append(keys);
  empty.append(withText(el("p", "history__empty-text"), EMPTY_PARAGRAPH));

  // The rule's "one thing to do next" is pressing the hotkey, which the keys
  // above already are and which cannot be a button. So the one action is the
  // other half of the rule, the way to change the setup.
  const change = withText(el("button", "history__empty-action"), "Change the hotkey");
  change.type = "button";
  change.addEventListener("click", () => {
    if (ctx.go) ctx.go("settings.dictation");
  });
  empty.append(change);

  state.list.replaceChildren(empty);
  fillKeys(ctx, keys);
}

async function fillKeys(ctx, keys) {
  let choice;
  try {
    choice = await invoke("get_hotkey");
  } catch (err) {
    console.error("history: the hotkey could not be read: " + reasonOf(err));
    return;
  }
  if (ctx.disposed) return;
  const wording = HOTKEY[choice && choice.chosen];
  if (!wording) {
    console.warn("history: no wording for the hotkey " + (choice && choice.chosen));
    return;
  }
  // design/registry.md "Hotkey choice": the Keycap primitive for the one
  // modifier, then the plain wording, so one hotkey looks like one thing
  // wherever it is shown.
  keys.replaceChildren(
    withText(el("span", "history__keycap"), wording.key),
    withText(el("span", "history__keys-words"), wording.words),
  );
}

/* ---- What a person reads for a time and a duration ------------------- */

/** The machine's own conventions, from the browser, in its locale and time
 *  zone. Only this side knows either, and a format invented in a record would
 *  be one more piece of wording with no source. The stored value is UTC.
 *
 *  A stored value the browser cannot read is shown as it was stored rather than
 *  as a guess, on the same rule the rest of this app applies to a value it does
 *  not understand: never an instruction, and never invented. */
function whenOf(startedAt) {
  const at = new Date(startedAt);
  if (Number.isNaN(at.getTime())) return startedAt;
  return at.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Whole seconds below a minute, minutes and seconds above it (record 0007).
 *  Rounded, never a millisecond figure: nobody reads a dictation's length to
 *  three decimal places. */
function howLongOf(durationMs) {
  const seconds = Math.max(0, Math.round(durationMs / 1000));
  if (seconds < 60) return seconds + "s";
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  return minutes + "m " + String(rest).padStart(2, "0") + "s";
}

/* ---- Setting error line ---------------------------------------------- */

/** design/registry.md "Setting error line": a mono code, then one plain
 *  sentence of cause, and no action button, because the control is its own
 *  retry. Both parts come from Rust; this builds the shape and nothing else.
 *  One line for the whole screen, which is that row's rule. */
function errorLine() {
  const line = el("div", "history__error");
  line.hidden = true;
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(el("span", "history__error-code"), el("p", "history__error-text"));
  return line;
}

/** Fill and show the line from a command's error. Returns false when that error
 *  carries no line, which is the two refusals that cannot happen here. */
function showError(line, err) {
  const code = err && err.code;
  const message = err && err.message;
  if (!code || !message) return false;
  line.querySelector(".history__error-code").textContent = code;
  line.querySelector(".history__error-text").textContent = message;
  line.hidden = false;
  return true;
}

function clearError(line) {
  line.hidden = true;
  line.querySelector(".history__error-code").textContent = "";
  line.querySelector(".history__error-text").textContent = "";
}

/** The machine cause, for a log line only. Never shown to a person, and never
 *  carrying a transcript or a search. */
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
