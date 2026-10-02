// The transcription settings screen: the saved Deepgram key (record 0002 AC-12,
// AC-34, AC-35, settled by that record's sixteenth amendment;
// design/registry.md "Secret field", "Secret field, replacing", "Secret field,
// removing", "Secret field, empty", "Setting error line").
//
// It is the white reading surface of the dashboard, at Settings, Transcription.
// It is not a dark interruption: nothing here has gone wrong and there are two
// things a person might do, which is the opposite shape to every dark screen in
// this app.
//
// Three commands and nothing else: `get_deepgram_key_info`,
// `save_deepgram_key`, `clear_deepgram_key`. It emits nothing and listens to
// nothing. All three have existed since milestone 3.
//
// **The key is never here.** Rust hands back its last four characters and
// nothing else, and the mask beside them is a fixed run of bullets that says
// only that there is a secret. It is not the key's length: the length is not
// stored and must never be, because a length narrows a secret. A key being
// pasted is a secret from the first keystroke, in a password field, never
// rendered in plain text and never read back for anything but the one check.
//
// **No sentence a person reads on a failure lives here.** Every code and
// sentence arrives on the command's error, from the one place in Rust that
// holds it: `settings.rs` for a refused write and a failed read, and
// `deepgram_key.rs` for every way a pasted key can be turned down.

const { invoke } = window.__TAURI__.core;

/** The label on this setting, in the shape design/registry.md's "Setting row"
 *  uses for every setting: a label, then the control. The comp's own words. */
const LABEL = "Deepgram API key";

/** design/registry.md "Secret field": the `Mono badge` primitive beside the
 *  label, saying both of the things this row is. The comp's own words. */
const BADGE = "SECRET · STORED LOCALLY";

/** The mask, and **it carries no information**. The key's length is not stored
 *  and must never be, so this is a fixed run whatever the key was and says only
 *  "there is a secret here" (record 0002, sixteenth amendment). Twenty, which
 *  is what the comp draws. */
const MASK = "•".repeat(20);

/** The one caption beneath the row. The comp's own words, fixed by record
 *  0002's sixteenth amendment. The first sentence is dropped when there is no
 *  verified date rather than a date being invented for it. */
const NEVER_LEAVES = "Never leaves this machine.";

/** The sentence that stands where `Secret field, empty`'s blocker sentence
 *  would be on the first-run path. That one, "Dictation is off until this is
 *  set", is false here: a working key is still saved and dictation still works.
 *  Settled 2026-09-04 under `/develop`'s gate option 3, carried into record
 *  0002 by its seventeenth amendment. */
const STILL_IN_FORCE = "Your saved key stays in force until Deepgram accepts a new one.";

/** The question Remove asks before anything is deleted (record 0002 AC-35, and
 *  its sixteenth amendment, which fixed these words). It says the two things a
 *  person cannot see for themselves: what stops, and that there is no copy to
 *  take first. It claims nothing about Deepgram's own console, which this
 *  project does not control. */
const REMOVE_QUESTION =
  "Dictation stops until another key is saved. EchoScribe cannot show you this one first.";

/** The wording on every control here. All four settled 2026-09-04 under
 *  `/develop`'s gate option 3 and carried into record 0002 by its seventeenth
 *  amendment. `Verify` and `Get a free key from Deepgram` are the key setup
 *  screen's own words, so one action has one name across the app, and `Cancel`
 *  is one word backing out of either state, so one word means one thing here. */
const REPLACE = "Replace";
const REMOVE = "Remove";
const CANCEL = "Cancel";
const VERIFY = "Verify";
const CHECKING = "Checking key";
const GET_A_KEY = "Get a free key from Deepgram";
const OPEN_CONSOLE = "Open Deepgram console";

/** Mount the transcription settings screen into `root`. Returns an unmount
 *  function. The read runs after this returns, so a second mount arriving first
 *  cannot be painted over by a first one finishing late. */
export function mountTranscriptionSettings(root) {
  const ctx = {
    root,
    disposed: false,
    // "saved", "replacing" or "removing". design/registry.md draws all three.
    mode: "saved",
    info: null,
    // Kept across redraws so a rejected key stays in the field: one typo must
    // not cost a fresh paste (design/registry.md "Key setup, rejected").
    pasted: "",
    error: null,
  };
  load(ctx);
  return function unmount() {
    ctx.disposed = true;
  };
}

async function load(ctx) {
  let info;
  try {
    info = await invoke("get_deepgram_key_info");
  } catch (err) {
    if (ctx.disposed) return;
    drawReadFailure(ctx, err);
    return;
  }
  if (ctx.disposed) return;
  if (!info) {
    // No key saved. This surface cannot ordinarily be reached in that state,
    // because the dashboard exists only while a key is saved, so arriving here
    // means the key went a moment ago and Rust is already closing this window.
    // Say so where a developer sees it and draw nothing, rather than a state
    // nobody designed. Same reasoning as the refusals below.
    console.error("transcription settings: no key is saved, so nothing is drawn");
    ctx.root.replaceChildren();
    return;
  }
  ctx.info = info;
  draw(ctx);
}

/** The read failure state: the one line record 0002 fixed, and no control at
 *  all (design/registry.md "Settings, could not be read"). A control drawn
 *  without a value would be showing a setting the app cannot read. */
function drawReadFailure(ctx, err) {
  const line = errorLine();
  if (!fillError(line, err)) {
    // The refusal carries no line, which means nobody is signed in or the core
    // is not up. Neither can happen on a screen only reachable while signed in,
    // and if one does then Rust is already closing this window.
    console.error("transcription settings: the key was refused: " + reasonOf(err));
    ctx.root.replaceChildren();
    return;
  }
  const screen = el("section", "tset");
  screen.append(line);
  ctx.root.replaceChildren(screen);
}

/** Draw whichever of the three states the row is in. The whole section is
 *  rebuilt rather than patched, the same shape key-setup.js uses, because each
 *  state hands the keyboard somewhere different and a rebuild makes that one
 *  decision instead of three. */
function draw(ctx) {
  const screen = el("section", "tset");
  const group = el("div", "tset__group");

  const label = withText(el("span", "tset__label"), LABEL);
  label.id = "tset-key-label";
  const head = el("div", "tset__head");
  head.append(label, withText(el("span", "tset__badge"), BADGE));

  const line = errorLine();
  if (ctx.error) fillError(line, ctx.error);

  group.append(head);
  if (ctx.mode === "replacing") {
    group.append(replacingWell(ctx, label), line, actionFor(ctx.error), note(STILL_IN_FORCE));
  } else if (ctx.mode === "removing") {
    group.append(removingWell(ctx), line, caption(ctx));
  } else {
    group.append(savedWell(ctx), line, caption(ctx));
  }

  screen.append(group);
  ctx.root.replaceChildren(screen);
  handKeyboardOver(ctx, screen);
}

/* ---- Secret field, the saved state ----------------------------------- */

/** design/registry.md "Secret field": the mask, the trailing fragment, then
 *  the two actions. `Setting row` puts one action on the right and this row
 *  carries two, which is what makes it that row's specialisation. */
function savedWell(ctx) {
  const well = el("div", "tset__well");
  well.append(mask(), fragment(ctx));

  const actions = el("div", "tset__actions");
  actions.append(
    button("tset__btn", REPLACE, () => {
      ctx.mode = "replacing";
      ctx.pasted = "";
      ctx.error = null;
      draw(ctx);
    }),
    button("tset__btn tset__btn--danger", REMOVE, () => {
      ctx.mode = "removing";
      ctx.error = null;
      draw(ctx);
    }),
  );
  well.append(actions);
  return well;
}

/** The mask and the fragment together are one readable claim, so the bullets
 *  are hidden from a screen reader, which would otherwise read twenty of them,
 *  and the fragment carries the whole sentence in words. */
function mask() {
  const node = withText(el("span", "tset__mask"), MASK);
  node.setAttribute("aria-hidden", "true");
  return node;
}

function fragment(ctx) {
  const node = el("span", "tset__fragment");
  node.append(
    withText(el("span", "tset__hidden"), "Saved key ending "),
    withText(el("span", "tset__fragment-text"), "·" + ctx.info.last_four),
  );
  return node;
}

/** The caption beneath the row, in record 0002's words. The verified date is
 *  `last_validated_at`, the moment Deepgram last accepted the key. When it is
 *  missing the first sentence is dropped: the date is never taken from
 *  `saved_at` instead, because saved is not verified. */
function caption(ctx) {
  const verified = formatDate(ctx.info.last_validated_at);
  const words = verified ? "Verified " + verified + ". " + NEVER_LEAVES : NEVER_LEAVES;
  return withText(el("p", "tset__caption"), words);
}

/* ---- Secret field, replacing ----------------------------------------- */

/** design/registry.md "Secret field, replacing": `Secret field, empty`'s paste
 *  field, in place on this surface. Replace never hands a person to the small
 *  dark window: that window is for a single way forward and this is not. */
function replacingWell(ctx, labelNode) {
  const form = el("form", "tset__well tset__well--paste");
  form.noValidate = true;

  // The field's accessible name is the setting's own label, which is right: it
  // is the same setting, being given a new value. The placeholder is not a
  // label and is not used as one.
  const input = el("input", "tset__input");
  input.type = "password";
  input.id = "tset-key";
  input.name = "tset-key";
  input.placeholder = "Paste your key";
  input.autocomplete = "off";
  input.spellcheck = false;
  input.value = ctx.pasted;
  input.setAttribute("aria-labelledby", labelNode.id);

  const verify = el("button", "tset__btn tset__btn--primary");
  verify.type = "submit";
  verify.textContent = VERIFY;
  // Nothing to check yet. Genuinely disabled rather than aria-disabled: there
  // is no action to announce and no busy state to hold focus for.
  verify.disabled = ctx.pasted.trim() === "";

  input.addEventListener("input", () => {
    ctx.pasted = input.value;
    verify.disabled = input.value.trim() === "";
  });

  const cancel = button("tset__btn", CANCEL, () => {
    ctx.mode = "saved";
    ctx.pasted = "";
    ctx.error = null;
    draw(ctx);
  });

  const actions = el("div", "tset__actions");
  actions.append(verify, cancel);
  form.append(input, actions);
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (verify.disabled || verify.getAttribute("aria-disabled") === "true") return;
    check(ctx, input, verify);
  });
  return form;
}

/** Check the pasted key against Deepgram through the same command that admits a
 *  first one, so a replacement is admitted on exactly the terms a first key is
 *  (record 0002 AC-34). Nothing is written on any failing path, so the old key
 *  stays in force and dictation keeps working. */
async function check(ctx, input, verify) {
  // design/registry.md "Key setup, checking", on the light primary button: the
  // label swaps to what is being waited on and the field is not editable.
  // Deliberately not the `disabled` attribute, which drops focus and strands
  // anyone using a keyboard mid action.
  verify.classList.add("tset__btn--busy");
  verify.setAttribute("aria-busy", "true");
  verify.setAttribute("aria-disabled", "true");
  verify.textContent = CHECKING;
  input.readOnly = true;

  try {
    await invoke("save_deepgram_key", { key: ctx.pasted });
  } catch (err) {
    if (ctx.disposed) return;
    // Every failing path leaves the old key exactly where it was, which is what
    // the sentence from Rust says. The pasted text stays in the field.
    ctx.error = err;
    draw(ctx);
    return;
  }
  if (ctx.disposed) return;

  // Accepted and stored. The row goes back to the saved state, and the last
  // four and the verified date are re-read rather than patched from what the
  // command returned, so what is on screen is what is in the database.
  ctx.pasted = "";
  ctx.error = null;
  ctx.mode = "saved";
  load(ctx);
}

/** The one action a key error can carry that this surface cannot answer with
 *  the verify button itself. Three of the causes this check produces are Try
 *  again, and the verify button is that; a spent allowance cannot arrive here
 *  at all but is mapped in Rust anyway, so it is drawn rather than left without
 *  a way forward, exactly as the key setup screen draws it. */
function actionFor(error) {
  const row = el("div", "tset__action");
  if (error && error.action === "open_deepgram_console") {
    row.append(
      button("tset__btn", OPEN_CONSOLE, () => openFixedPage("open_deepgram_console")),
    );
    return row;
  }
  // AC-9's link out, in the key setup screen's own words. A button and not an
  // anchor: Rust holds the one address it is allowed to open, and navigating
  // this window would take the dashboard somewhere it cannot come back from.
  row.append(button("tset__link", GET_A_KEY, () => openFixedPage("open_deepgram_signup")));
  return row;
}

/* ---- Secret field, removing ------------------------------------------ */

/** design/registry.md "Secret field, removing": the question replaces the row's
 *  two actions in place. **Never a modal**, and the mask stays on screen, so a
 *  person can see the thing they are about to remove. */
function removingWell(ctx) {
  const well = el("div", "tset__well tset__well--asking");
  well.append(mask(), fragment(ctx));

  const question = withText(el("p", "tset__question"), REMOVE_QUESTION);
  question.id = "tset-remove-question";
  // A question a person did not ask for has appeared, so it is read out rather
  // than only painted.
  question.setAttribute("role", "alert");

  const confirm = button("tset__btn tset__btn--danger", REMOVE, () => remove(ctx, confirm));
  confirm.setAttribute("aria-describedby", question.id);
  const cancel = button("tset__btn", CANCEL, () => {
    ctx.mode = "saved";
    draw(ctx);
  });
  // Named for the one line below that has to find it again. Focus lands here
  // and not on Remove, so a stray second press backs out rather than destroys.
  cancel.dataset.focusFirst = "true";

  const actions = el("div", "tset__actions");
  actions.append(confirm, cancel);
  well.append(actions);

  const asking = el("div", "tset__asking");
  asking.append(well, question);
  return asking;
}

/** Remove the key (record 0002 AC-35). Rust removes the row first and emits
 *  `dictation:key_cleared` the moment it is gone, so on success this surface is
 *  on its way out with the dashboard and there is nothing left to draw here. */
async function remove(ctx, confirm) {
  confirm.setAttribute("aria-busy", "true");
  confirm.setAttribute("aria-disabled", "true");

  try {
    await invoke("clear_deepgram_key");
  } catch (err) {
    if (ctx.disposed) return;
    // Nothing was removed: Rust returns this only when the row would not
    // delete, which is why "so it is unchanged" is true. Back to the saved
    // state, with the line above the caption.
    ctx.error = err;
    ctx.mode = "saved";
    draw(ctx);
    return;
  }
  // Removed. Rust is closing this window; drawing anything now would be drawing
  // over a surface that is going.
}

/* ---- Setting error line ---------------------------------------------- */

/** design/registry.md "Setting error line": a mono code, then one plain
 *  sentence of cause. Both parts come from Rust; this builds the shape and
 *  nothing else. */
function errorLine() {
  const line = el("div", "tset__error");
  line.hidden = true;
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(el("span", "tset__error-code"), el("p", "tset__error-text"));
  return line;
}

/** Fill and show the line from a command's error. Returns false when that error
 *  carries no line, which is the two refusals that cannot happen on a screen
 *  only reachable while signed in. */
function fillError(line, err) {
  const code = err && err.code;
  const message = err && err.message;
  if (!code || !message) return false;
  line.querySelector(".tset__error-code").textContent = code;
  line.querySelector(".tset__error-text").textContent = message;
  line.hidden = false;
  return true;
}

/** The machine cause, for a log line only. Never shown to a person. */
function reasonOf(err) {
  return (err && err.reason) || String(err);
}

/* ---- Small helpers --------------------------------------------------- */

/** Where the keyboard goes after each redraw. Replacing hands it the field,
 *  because that state exists to be typed into. Removing hands it **Cancel and
 *  not Remove**, so a second stray press backs out rather than destroys
 *  (design/registry.md "Secret field, removing"). The saved state takes it
 *  nowhere: nothing was asked for, and moving focus on a first paint would
 *  steal it from the rail. */
function handKeyboardOver(ctx, screen) {
  if (ctx.mode === "replacing") {
    const input = screen.querySelector(".tset__input");
    if (!input) return;
    input.focus();
    const end = input.value.length;
    try {
      // The caret goes to the end rather than selecting, so a stray keystroke
      // cannot wipe what was pasted.
      input.setSelectionRange(end, end);
    } catch (_) {
      // Some browsers refuse setSelectionRange on a password field. The caret
      // lands at the end there anyway.
    }
    return;
  }
  if (ctx.mode === "removing") {
    const cancel = screen.querySelector("[data-focus-first]");
    if (cancel) cancel.focus();
  }
}

async function openFixedPage(command) {
  try {
    await invoke(command);
  } catch (_) {
    // Rust already says what happened on stderr. The surface stays exactly as
    // it is, which is the drawn behaviour, and the action remains available.
  }
}

/** The verified date in the machine's own locale, or null when there is none.
 *  A second copy of `src/shell/account-card.js`'s formatter on purpose:
 *  AGENTS.md makes something shared when a third feature needs it, and two is
 *  a coincidence. */
function formatDate(iso) {
  if (!iso) return null;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function note(words) {
  return withText(el("p", "tset__note"), words);
}

function el(tag, className) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

function withText(node, str) {
  node.textContent = str;
  return node;
}

function button(className, label, onClick) {
  const node = el("button", className);
  node.type = "button";
  node.textContent = label;
  node.addEventListener("click", () => {
    if (node.getAttribute("aria-disabled") === "true") return;
    onClick();
  });
  return node;
}
