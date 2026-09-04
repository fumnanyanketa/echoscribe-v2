// The vocabulary screen: the words Deepgram keeps getting wrong (record 0005;
// design/registry.md "Term add field", "Term row", "Term room left",
// "Vocabulary empty state", "Vocabulary, could not be read",
// "Setting error line").
//
// It is the white reading surface of the dashboard, at Settings, Vocabulary.
// Not a dark interruption: nothing here has gone wrong and there are several
// things to do, which is the opposite shape to every dark screen in this app.
//
// Three commands and nothing else: `get_vocabulary`, `add_vocabulary_term`,
// `remove_vocabulary_term`. It emits nothing and listens to nothing.
//
// **Both limits come from Rust and neither is held here.** The longest a word
// may be and how much room is left both arrive on every answer, for the same
// reason `get_hotkey` hands out the two hotkeys: what is allowed is Rust's
// answer, so this screen cannot show a person a limit that is not the limit. A
// guard in `src-tauri/src/vocabulary/mod.rs` fails the build if either number
// appears in this file.
//
// **No sentence a person reads on a failure lives here.** The code and the
// sentence arrive on the command's error, from the one place in Rust that holds
// all seven. The same guard keeps this file from growing a copy.
//
// **Every add and every remove redraws from what Rust returned**, never from
// what this file thought the list was. Each command answers with the whole
// list and the new room left, so the words on screen and the room beside them
// always come from the same moment.

const { invoke } = window.__TAURI__.core;

/** The one caption on this screen, and it says the thing a person cannot see
 *  for themselves (record 0005). Their words leave the machine with every
 *  dictation, and AGENTS.md makes that knowledge something a person is owed.
 *  The second sentence is the true and reassuring half: the list is stored here
 *  and nowhere else. It claims nothing about what Deepgram does with a request,
 *  because this project does not control that. */
const CAPTION =
  "Each word here is sent to Deepgram with your voice, so it knows to listen " +
  "for it. Nothing is stored outside this machine.";

/** The empty state, which is the first thing every person sees, because nobody
 *  starts with a vocabulary (design/registry.md "Vocabulary empty state").
 *  Three sentences doing three jobs: what has not happened, what the list is
 *  for with examples rather than an abstraction, and where to go next, which is
 *  the field already on screen. Settled by record 0005. */
const EMPTY =
  "Nothing added yet. The words to add here are the ones Deepgram keeps " +
  "getting wrong: names, places, and the jargon of your work. Add one above " +
  "and your next dictation will listen for it.";

/** Mount the vocabulary screen into `root`. Returns an unmount function. The
 *  read runs after this returns, so a second mount arriving first cannot be
 *  painted over by a first one finishing late. */
export function mountVocabulary(root) {
  const ctx = { root, disposed: false };
  load(ctx);
  return function unmount() {
    ctx.disposed = true;
  };
}

async function load(ctx) {
  let view;
  try {
    view = await invoke("get_vocabulary");
  } catch (err) {
    if (ctx.disposed) return;
    drawReadFailure(ctx, err);
    return;
  }
  if (ctx.disposed) return;
  draw(ctx, view);
}

/** design/registry.md "Vocabulary, could not be read": one error line and no
 *  control at all. Not the add field, not the list, nothing holding a guessed
 *  value. An empty list is a person with no words; an unreadable one is a
 *  person whose words may all still be there, and drawing the first for the
 *  second would be a lie. */
function drawReadFailure(ctx, err) {
  const line = errorLine();
  if (!showError(line, err)) {
    // The refusal has no line, which means nobody is signed in or the core is
    // not up. Neither can happen on a screen only reachable while signed in,
    // and if one does then Rust is already closing this window. Say so where a
    // developer sees it and draw nothing. Same reasoning as dashboard.js.
    console.error("vocabulary: the list was refused: " + reasonOf(err));
    ctx.root.replaceChildren();
    return;
  }
  const screen = el("section", "vocab");
  screen.append(line);
  ctx.root.replaceChildren(screen);
}

function draw(ctx, view) {
  const line = errorLine();
  const state = { ctx, view, line };

  const screen = el("section", "vocab");
  state.addGroup = addGroup(state);
  state.listGroup = el("div", "vocab__group");
  screen.append(state.addGroup, line, state.listGroup);
  paintList(state);
  ctx.root.replaceChildren(screen);
  state.screen = screen;
}

/* ---- Term add field -------------------------------------------------- */

/** design/registry.md "Term add field": the only way a word ever enters the
 *  list. A one line field, one action beside it, the room left beneath, and the
 *  caption. */
function addGroup(state) {
  const group = el("div", "vocab__group");

  const label = withText(el("label", "vocab__label"), "Add a word or phrase");
  label.id = "vocab-add-label";
  label.htmlFor = "vocab-add-field";

  const field = el("input", "vocab__field");
  field.id = "vocab-add-field";
  field.type = "text";
  field.autocomplete = "off";
  field.spellcheck = false;
  // The browser lays the field out from what is in it, so a person typing in
  // their own script sees it the right way round as they type. Same mechanism
  // as the pill's transcript line and a term row: nothing declares a direction
  // and no list of right to left languages exists anywhere.
  field.dir = "auto";
  // The longest a word may be is Rust's answer, arriving on every read, so the
  // field cannot let a person type past a limit this file does not know. It is
  // a courtesy and not the rule: `rules.rs` refuses a long word whatever this
  // attribute says, and a paste can still exceed it in some browsers.
  field.maxLength = state.view.max_term_characters;
  // No placeholder. It has a real label directly above it, and a placeholder
  // repeating that label is noise that vanishes the moment somebody types,
  // which is exactly when a person still wants the label (record 0005).
  field.setAttribute("aria-describedby", "vocab-room vocab-caption");

  const add = withText(el("button", "vocab__add"), "Add");
  add.type = "button";

  const room = el("p", "vocab__room");
  room.id = "vocab-room";
  // The room left changes without the person having asked a question, so it is
  // read out rather than only repainted.
  room.setAttribute("aria-live", "polite");

  const caption = withText(el("p", "vocab__caption"), CAPTION);
  caption.id = "vocab-caption";

  state.field = field;
  state.add = add;
  state.room = room;

  add.addEventListener("click", () => submit(state));
  // Enter does what Add does. A person typing a list of words should not have
  // to reach for the mouse between each one (design/registry.md).
  field.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    submit(state);
  });
  // The Add control is unusable while there is nothing to add, which is what
  // makes record 0005's empty term unreachable and why that refusal carries no
  // sentence. Recomputed on every input, so pasting spaces over a real word
  // disables it again.
  field.addEventListener("input", () => paintAdd(state));

  const row = el("div", "vocab__add-row");
  row.append(field, add);
  group.append(label, row, room, caption);
  paintAdd(state);
  return group;
}

/** Whether Add can be pressed, and the room left in words. Both from the same
 *  state, so a full list and an empty field cannot disagree about why nothing
 *  can be added. */
function paintAdd(state) {
  const typed = state.field.value.trim();
  const full = state.view.room_for_characters === 0;
  const disabled = typed === "" || full;
  state.add.disabled = disabled;
  // Two signals for a full list, and one of them is words: this line's own
  // wording changes and the action becomes unusable (design/registry.md
  // "Term room left").
  state.room.textContent = full
    ? "No room left. Remove a word to make space."
    : "Room for " + state.view.room_for_characters + " more characters.";
  state.room.classList.toggle("vocab__room--full", full);
}

async function submit(state) {
  const term = state.field.value.trim();
  if (term === "") return;
  clearError(state.line);
  let view;
  try {
    view = await invoke("add_vocabulary_term", { term });
  } catch (err) {
    if (state.ctx.disposed) return;
    // The list on screen does not move (design/registry.md "Setting error
    // line"), and what was typed stays in the field, so one refusal does not
    // cost a person their typing. The control is its own retry.
    if (!showError(state.line, err)) {
      console.error("vocabulary: a word was refused: " + reasonOf(err));
    }
    return;
  }
  if (state.ctx.disposed) return;
  // Written. Only now does the field empty, and it keeps focus so the next
  // word can be typed straight away.
  state.view = view;
  state.field.value = "";
  paintAdd(state);
  paintList(state);
  state.field.focus();
}

/* ---- The list -------------------------------------------------------- */

/** design/registry.md "Term row" and "Vocabulary empty state". Newest first,
 *  which is record 0005's choice, so an add can be seen to have worked. */
function paintList(state) {
  const group = state.listGroup;
  const terms = state.view.terms || [];

  if (terms.length === 0) {
    // The empty state replaces the list and nothing else: the field, its label
    // and the room left all stay where they are.
    group.replaceChildren(withText(el("p", "vocab__empty"), EMPTY));
    return;
  }

  const label = withText(el("span", "vocab__label"), "Your words");
  label.id = "vocab-list-label";

  const list = el("ul", "vocab__list");
  list.setAttribute("aria-labelledby", label.id);
  for (const term of terms) {
    list.append(termRow(state, term));
  }
  group.replaceChildren(label, list);
}

function termRow(state, term) {
  const row = el("li", "vocab__row");

  const word = withText(el("span", "vocab__term"), term.term);
  // The word reads the way its owner typed it, whatever the script. Rust
  // refuses any character that could reorder it, so this is direction alone.
  word.dir = "auto";

  const remove = withText(el("button", "vocab__remove"), "Remove");
  remove.type = "button";
  // The label says "Remove" for every row, so a screen reader user needs the
  // word it removes as well. The visible label stays short.
  remove.setAttribute("aria-label", "Remove " + term.term);
  // Remove asks nothing first, and that is the deliberate opposite of the
  // saved Deepgram key: that one can never be shown again, and a word can be
  // retyped in seconds, so a question here would be ceremony
  // (design/registry.md "Term row").
  remove.addEventListener("click", () => drop(state, term.id));

  row.append(word, remove);
  return row;
}

async function drop(state, id) {
  clearError(state.line);
  let view;
  try {
    view = await invoke("remove_vocabulary_term", { id });
  } catch (err) {
    if (state.ctx.disposed) return;
    // The list does not move. Same reasoning as a refused add.
    if (!showError(state.line, err)) {
      console.error("vocabulary: a word could not be removed: " + reasonOf(err));
    }
    return;
  }
  if (state.ctx.disposed) return;
  state.view = view;
  paintAdd(state);
  paintList(state);
}

/* ---- Setting error line ---------------------------------------------- */

/** design/registry.md "Setting error line": a mono code, then one plain
 *  sentence of cause, and no action button, because the control is its own
 *  retry. Both parts come from Rust; this builds the shape and nothing else.
 *  One line for the whole screen, which is that row's rule: one line per
 *  surface, every cause routed through it. */
function errorLine() {
  const line = el("div", "vocab__error");
  line.hidden = true;
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(el("span", "vocab__error-code"), el("p", "vocab__error-text"));
  return line;
}

/** Fill and show the line from a command's error. Returns false when that
 *  error carries no line, which is the three refusals that cannot happen on
 *  this screen. */
function showError(line, err) {
  const code = err && err.code;
  const message = err && err.message;
  if (!code || !message) return false;
  line.querySelector(".vocab__error-code").textContent = code;
  line.querySelector(".vocab__error-text").textContent = message;
  line.hidden = false;
  return true;
}

function clearError(line) {
  line.hidden = true;
  line.querySelector(".vocab__error-code").textContent = "";
  line.querySelector(".vocab__error-text").textContent = "";
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
