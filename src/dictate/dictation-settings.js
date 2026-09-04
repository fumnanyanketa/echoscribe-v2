// The dictation settings screen: the hotkey choice and the sound switch
// (record 0002 AC-19, AC-21, AC-22; design/registry.md "Hotkey choice",
// "Sound switch", "Setting error line").
//
// It is the white reading surface of the dashboard, at Settings, Dictation,
// which is where the rail lands at rest (record 0004 AC-1). It is not a dark
// interruption: nothing here has gone wrong and there are several things to do,
// which is the opposite shape to every dark screen in this app.
//
// Four commands and nothing else: `get_hotkey`, `set_hotkey`,
// `get_dictation_sounds`, `set_dictation_sounds`. It emits nothing and listens
// to nothing.
//
// **The two hotkey rows come from what `get_hotkey` returned and never from a
// list held here.** That is AC-22 as a rule on this side as well as in Rust:
// there is no third row, no box to type a key into, no way to record one and no
// Rebind. What is held here is the wording for each value Rust can hand out,
// the same division as the rail's, so an identifier this file has no wording
// for is not drawn at all rather than shown to a person as a machine value.
//
// **Neither sentence a person reads on a failure lives here.** The code and the
// sentence arrive on the command's error, from the one place in Rust that holds
// them, `src-tauri/src/dictate/settings.rs`. A guard in that file keeps this
// one from growing a second copy.

const { invoke } = window.__TAURI__.core;

/** What a person reads for each hotkey `get_hotkey` can hand out.
 *  design/registry.md "Hotkey choice" owns this wording: the `Keycap`
 *  primitive carries the one modifier, the words carry the double tap.
 *  Adding a row here would not add a hotkey, because Rust refuses anything
 *  that is not one of its own two, but it would put a row on screen that
 *  cannot be chosen. A hotkey arrives here with an amendment or not at all. */
const HOTKEY_WORDING = {
  double_tap_ctrl: { keycap: "Ctrl", label: "Double tap Ctrl" },
  double_tap_alt: { keycap: "Alt", label: "Double tap Alt" },
};

/** The one caption beneath both hotkey rows, once and not per row
 *  (design/registry.md "Hotkey choice"). It carries the whole of AC-19's
 *  interaction: how dictation starts, how it stops, and that either side of
 *  the keyboard counts. Settled by the user on 2026-09-04; see
 *  docs/evidence/dictate-with-a-hotkey/milestone-5-screen-decisions-owed.md. */
const HOTKEY_CAPTION =
  "Double tap to start dictating, double tap again to stop. " +
  "The left and right keys both count.";

/** The one caption beneath the sound switch (design/registry.md "Sound
 *  switch"). It says the one thing AC-21 promises and AGENTS.md makes a hard
 *  limit, and the one thing a person would otherwise reasonably fear: the pill
 *  still appears, because it is the only visible sign that the microphone is
 *  open and there is no silent listening in this app. Settled by the user on
 *  2026-09-04, same evidence file. */
const SOUND_CAPTION =
  "Turning sounds off silences the start and stop sounds only. " +
  "The pill still appears whenever the microphone is open.";

/** Mount the dictation settings screen into `root`. Returns an unmount
 *  function. The two reads run after this returns, so a second mount arriving
 *  first cannot be painted over by a first one finishing late. */
export function mountDictationSettings(root) {
  const ctx = { root, disposed: false };
  load(ctx);
  return function unmount() {
    ctx.disposed = true;
  };
}

async function load(ctx) {
  // Both settings are read before anything is drawn. If either read fails the
  // screen is one error line and no control at all: a control drawn without a
  // chosen value would be showing a setting the app cannot read, and a person
  // could leave believing a hotkey is in force that is not (record 0002,
  // fourteenth amendment).
  let hotkey;
  let sounds;
  try {
    hotkey = await invoke("get_hotkey");
    sounds = await invoke("get_dictation_sounds");
  } catch (err) {
    if (ctx.disposed) return;
    drawReadFailure(ctx, err);
    return;
  }
  if (ctx.disposed) return;
  draw(ctx, hotkey, sounds);
}

/** The read failure state: the one line the record fixed, and nothing else. */
function drawReadFailure(ctx, err) {
  const line = errorLine();
  if (!showError(line, err)) {
    // The refusal has no line, which means nobody is signed in or the core is
    // not up. Neither can happen on a screen only reachable while signed in,
    // and if one does then Rust is already closing this window. Say so where a
    // developer sees it and draw nothing: the alternative is a sentence for a
    // state nobody designed. Same reasoning as dashboard.js on `get_rail`.
    console.error("dictation settings: the settings were refused: " + reasonOf(err));
    ctx.root.replaceChildren();
    return;
  }
  const screen = el("section", "dset");
  screen.append(line);
  ctx.root.replaceChildren(screen);
}

function draw(ctx, hotkey, sounds) {
  const screen = el("section", "dset");
  screen.append(hotkeyGroup(ctx, hotkey), soundGroup(ctx, sounds));
  ctx.root.replaceChildren(screen);
}

/* ---- Hotkey choice --------------------------------------------------- */

/** design/registry.md "Hotkey choice": exactly two rows, one per allowed
 *  hotkey, rendered from what `get_hotkey` returned. One radio group, arrow
 *  keys move between them, one caption beneath both. */
function hotkeyGroup(ctx, hotkey) {
  const group = el("div", "dset__group");

  // The setting's label, in the shape design/registry.md's "Setting row" uses
  // for every setting: a label, then the control. The word is the comp's own,
  // on the Settings artboard above this control, and the user confirmed it on
  // 2026-09-04 rather than leaving a radio group with no name.
  const label = withText(el("span", "dset__label"), "Hotkey");
  label.id = "dset-hotkey-label";

  const caption = withText(el("p", "dset__caption"), HOTKEY_CAPTION);
  caption.id = "dset-hotkey-caption";

  const rows = el("div", "dset__rows");
  // A custom radio group, because two rows carrying a keycap, wording and a
  // badge are not a native radio. Named by the label and described by the
  // caption, so a screen reader gets the setting and then its interaction.
  rows.setAttribute("role", "radiogroup");
  rows.setAttribute("aria-labelledby", label.id);
  rows.setAttribute("aria-describedby", caption.id);

  const line = errorLine();
  const state = { rows: [], chosen: hotkey.chosen, line, ctx };

  for (const value of hotkey.choices || []) {
    const wording = HOTKEY_WORDING[value];
    if (!wording) {
      // Rust offers a hotkey this build has no wording for. Rust and this file
      // are meant to change together; showing a person a machine value would
      // be worse than showing them one fewer row.
      console.warn("dictation settings: no wording for the hotkey " + value);
      continue;
    }
    const row = hotkeyRow(value, wording, state);
    state.rows.push(row);
    rows.append(row);
  }

  paintHotkey(state);
  group.append(label, rows, caption, line);
  return group;
}

function hotkeyRow(value, wording, state) {
  const row = el("button", "dset__row");
  row.type = "button";
  row.setAttribute("role", "radio");
  row.dataset.hotkey = value;
  row.append(
    withText(el("span", "dset__keycap"), wording.keycap),
    withText(el("span", "dset__row-label"), wording.label),
    // The second signal, so the choice never rests on the fill alone
    // (design/registry.md "Hotkey choice"). Present only on the chosen row, so
    // it is removed rather than hidden: an empty badge is a shape with no
    // meaning. `aria-hidden`, because `aria-checked` already says this to a
    // screen reader and saying it twice is noise.
    badge(),
  );
  row.addEventListener("click", () => choose(state, value));
  row.addEventListener("keydown", (event) => onArrow(event, state));
  return row;
}

function badge() {
  const node = withText(el("span", "dset__badge"), "CHOSEN");
  node.setAttribute("aria-hidden", "true");
  return node;
}

/** Arrow keys move between the two rows (design/registry.md "Hotkey choice").
 *  They move focus and do not choose: a choice is a write that can be refused,
 *  and arrowing past a row should not try to store it. Space and Enter choose,
 *  which a `<button>` already does for both. */
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

async function choose(state, value) {
  if (value === state.chosen) return;
  clearError(state.line);
  try {
    await invoke("set_hotkey", { binding: value });
  } catch (err) {
    if (state.ctx.disposed) return;
    // The shown choice does not move (design/registry.md "Setting error line"),
    // so the screen never displays a setting as changed that is not. The
    // control is its own retry: a person presses the other row again, and no
    // action button follows the line.
    if (!showError(state.line, err)) {
      console.error("dictation settings: the hotkey was refused: " + reasonOf(err));
    }
    return;
  }
  if (state.ctx.disposed) return;
  // Written and, in Rust, the keyboard hook re-armed with it, which is AC-19's
  // "works immediately, with no restart". Only now does the shown choice move.
  state.chosen = value;
  paintHotkey(state);
}

function paintHotkey(state) {
  for (const row of state.rows) {
    const chosen = row.dataset.hotkey === state.chosen;
    row.classList.toggle("dset__row--chosen", chosen);
    row.setAttribute("aria-checked", chosen ? "true" : "false");
    // One tab stop for the group, on the chosen row, which is the radio group
    // convention. Arrow keys reach the other one.
    row.tabIndex = chosen ? 0 : -1;
    const existing = row.querySelector(".dset__badge");
    if (chosen && !existing) row.append(badge());
    if (!chosen && existing) existing.remove();
  }
  if (!state.rows.some((row) => row.dataset.hotkey === state.chosen)) {
    // No row matches what Rust says is chosen, so every row is now unchecked
    // and the group has no tab stop. Give it one, on the first row, rather than
    // leave a keyboard user unable to reach the control at all.
    console.warn("dictation settings: the chosen hotkey has no row: " + state.chosen);
    if (state.rows[0]) state.rows[0].tabIndex = 0;
  }
}

/* ---- Sound switch ---------------------------------------------------- */

/** design/registry.md "Sound switch": the label, the switch, and the state in
 *  words beside it. Two signals, the knob position and a word, so the meaning
 *  never rests on a fill colour. Never violet in either state, because violet
 *  only ever means an open microphone. */
function soundGroup(ctx, enabled) {
  const group = el("div", "dset__group");

  const label = withText(el("span", "dset__label dset__label--inline"), "Dictation sounds");
  label.id = "dset-sound-label";

  const track = el("button", "dset__track");
  track.type = "button";
  track.setAttribute("role", "switch");
  // The switch is named by its label alone. The state word beside it is not
  // part of the name: `aria-checked` already carries the state, and folding the
  // word in would have a screen reader say it twice.
  track.setAttribute("aria-labelledby", label.id);
  track.append(el("span", "dset__knob"));

  const word = el("span", "dset__switch-state");
  const line = errorLine();
  const state = { enabled, track, word, line, ctx };

  track.addEventListener("click", () => toggleSound(state));

  const row = el("div", "dset__switch-row");
  row.append(label, track, word);
  paintSound(state);

  group.append(row, withText(el("p", "dset__caption"), SOUND_CAPTION), line);
  return group;
}

async function toggleSound(state) {
  const wanted = !state.enabled;
  clearError(state.line);
  try {
    await invoke("set_dictation_sounds", { enabled: wanted });
  } catch (err) {
    if (state.ctx.disposed) return;
    // The switch does not move. Same reasoning as the hotkey rows: the screen
    // never shows a setting as changed that is not.
    if (!showError(state.line, err)) {
      console.error("dictation settings: the sound setting was refused: " + reasonOf(err));
    }
    return;
  }
  if (state.ctx.disposed) return;
  state.enabled = wanted;
  paintSound(state);
}

function paintSound(state) {
  state.track.classList.toggle("dset__track--on", state.enabled);
  state.track.setAttribute("aria-checked", state.enabled ? "true" : "false");
  state.word.textContent = state.enabled ? "On" : "Off";
}

/* ---- Setting error line ---------------------------------------------- */

/** design/registry.md "Setting error line": a mono code, then one plain
 *  sentence of cause, and no action button, because the control is its own
 *  retry. Both parts come from Rust; this builds the shape and nothing else. */
function errorLine() {
  const line = el("div", "dset__error");
  line.hidden = true;
  // A write is refused without the person having asked a question, so the line
  // has to be read out rather than only repainted.
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(el("span", "dset__error-code"), el("p", "dset__error-text"));
  return line;
}

/** Fill and show the line from a command's error. Returns false when that
 *  error carries no line, which is the two refusals that cannot happen on a
 *  screen only reachable while signed in. */
function showError(line, err) {
  const code = err && err.code;
  const message = err && err.message;
  if (!code || !message) return false;
  line.querySelector(".dset__error-code").textContent = code;
  line.querySelector(".dset__error-text").textContent = message;
  line.hidden = false;
  return true;
}

function clearError(line) {
  line.hidden = true;
  line.querySelector(".dset__error-code").textContent = "";
  line.querySelector(".dset__error-text").textContent = "";
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
