// The three fact cards above the history list (design/design-system.md,
// "Night", re-locked 2026-10-02 by the user).
//
// Real values only, each from a command that already exists, and never an
// invented metric: how many dictations there are, from the same `get_history`
// read that drew the list; the hotkey, from the dictate feature's `get_hotkey`;
// and the chosen language, from the language feature's
// `get_transcription_language`. A card whose read fails keeps its label and
// shows a dash, so a fact that could not be read never looks like a fact.
//
// The language is shown as its code, in mono, the way every dictation row
// already shows it. The language names live in the language feature, and
// feature folders do not import from each other.

const { invoke } = window.__TAURI__.core;

/** What a person reads for each hotkey design/registry.md's `Hotkey choice`
 *  allows. The same division as the rail's wording and the hotkey rows on the
 *  settings surface: Rust hands out the stored value and this side decides what
 *  a person reads. A value with no wording here draws no keys at all rather
 *  than putting a machine identifier in front of somebody. */
export const HOTKEY = {
  double_tap_ctrl: { key: "Ctrl", words: "Double tap Ctrl" },
  double_tap_alt: { key: "Alt", words: "Double tap Alt" },
};

const ICON = {
  count: ["M4 6h16", "M4 12h16", "M4 18h10"],
  hotkey: ["M5 6h14a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2z", "M7 10h1M11 10h1M15 10h2M7 14h10"],
  language: ["M4 5h9M8.5 3v2M6 5c1 4 4 7 7 8M11 5c-1 4-4 7-7 8", "M13 21l4-9 4 9M14.5 18h5"],
};

/** The row of three cards. `total` is the dictation count from the read that
 *  drew the list, so the card and the list can never disagree. `isLive` says
 *  whether the screen is still showing, so a late answer paints nothing. */
export function factsRow(total, isLive) {
  const row = el("div", "history__facts");

  const count = fact(ICON.count, "DICTATIONS");
  count.value.textContent =
    typeof total === "number" ? total.toLocaleString("en-US") : "–";
  count.note.textContent = "Kept on this machine";

  const hotkey = fact(ICON.hotkey, "HOTKEY");
  hotkey.note.textContent = "Double tap to start and stop";
  fillHotkey(hotkey.value, isLive);

  const language = fact(ICON.language, "LANGUAGE");
  language.note.textContent = "Change it in Settings";
  fillLanguage(language.value, isLive);

  row.append(count.card, hotkey.card, language.card);
  return row;
}

function fact(paths, label) {
  const card = el("div", "history__fact");
  const head = el("div", "history__fact-label");
  head.append(icon(paths), document.createTextNode(label));
  const value = el("div", "history__fact-value");
  const note = el("div", "history__fact-note");
  card.append(head, value, note);
  return { card, value, note };
}

async function fillHotkey(slot, isLive) {
  slot.textContent = "–";
  let choice;
  try {
    choice = await invoke("get_hotkey");
  } catch (_) {
    return;
  }
  if (!isLive()) return;
  const wording = HOTKEY[choice && choice.chosen];
  if (!wording) return;
  slot.replaceChildren(
    withText(el("span", "history__fact-key"), wording.key),
    withText(el("span", "history__fact-times"), "× 2"),
  );
}

async function fillLanguage(slot, isLive) {
  slot.textContent = "–";
  let choice;
  try {
    choice = await invoke("get_transcription_language");
  } catch (_) {
    return;
  }
  if (!isLive() || !choice || typeof choice.chosen !== "string") return;
  slot.classList.add("history__fact-value--code");
  slot.textContent = choice.chosen;
}

function icon(paths) {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("class", "history__fact-icon");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("aria-hidden", "true");
  for (const d of paths) {
    const path = document.createElementNS(ns, "path");
    path.setAttribute("d", d);
    svg.append(path);
  }
  return svg;
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
