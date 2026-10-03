// The nav rail: the nav items, and the sub-nav beneath them
// (design/registry.md "Nav rail", "Nav item", "Section sub-nav").
//
// Rust hands out identifiers and this file decides what a person reads, the
// same division as `get_hotkey` and the two hotkey rows. So an identifier with
// no wording here is not rendered at all: showing a person a machine
// identifier would be worse than showing them one fewer destination, and Rust
// and this file are meant to change together.
//
// Arranged by the 2026-10-02 re-lock, "Night" (design/design-system.md): no
// brand row, since the window title already names the app; one stroke icon per
// top level item; History at the top, and a section that holds sub-sections
// at the foot with the account block beneath it, which lives next door in
// account-block.js.

/// What a person reads for each identifier `get_rail()` can return.
/// design/registry.md owns this wording. Adding a row here without its screen
/// would put a nav item on screen that opens nothing, which record 0004's AC-2
/// forbids, so a new row arrives with its feature and not before.
const LABEL = {
  history: "History",
  settings: "Settings",
  "settings.dictation": "Dictation",
  "settings.languages": "Languages",
  "settings.vocabulary": "Vocabulary",
  "settings.transcription": "Transcription",
};

/** One stroke icon per top level destination, as SVG path data on a 24 unit
 *  grid. A destination with no icon here simply has none; its label still says
 *  where it goes. Drawn with createElementNS, never as markup. */
const ICON = {
  history: ["M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18z", "M12 7v5l3 2"],
  settings: [
    "M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6z",
    "M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z",
  ],
};

/** The app icon's twelve bars, as fractions of its tallest one, measured off
 *  design/icon-1024.png. The rail's watermark draws exactly these, so it reads
 *  as the logo rather than as a new shape. */
const MARK_BARS = [0.23, 0.47, 0.82, 0.58, 1, 0.74, 0.42, 0.89, 0.63, 0.32, 0.55, 0.25];

/** The faded logo bleeding in from the rail's left edge, in the empty middle
 *  of the rail (design/design-system.md, "Night", asked for by the user on
 *  2026-10-02). Decoration only: hidden from a screen reader, never under a
 *  click, and drawn beneath every item. */
function watermark() {
  const node = el("div", "rail__watermark");
  node.setAttribute("aria-hidden", "true");
  for (const height of MARK_BARS) {
    const bar = el("i");
    bar.style.height = Math.round(height * 100) + "%";
    node.append(bar);
  }
  return node;
}

/** Draw the rail into `root` from what `get_rail()` returned.
 *  `onSelect` is called with the identifier of whatever was clicked.
 *  Returns the account block's own element, so the caller can fill it. */
export function mountRail(root, view, onSelect) {
  const nav = el("div", "rail__nav");
  const foot = el("div", "rail__foot");
  for (const item of view.items || []) {
    const label = LABEL[item.id];
    if (!label) {
      // Rust offers a destination this build has no wording for. Say so where a
      // developer will see it and draw nothing, rather than put an identifier
      // in front of a person.
      console.warn("rail: no wording for the destination " + item.id);
      continue;
    }
    const children = (item.children || []).filter((child) => LABEL[child.id]);
    // A section with sub-sections sits at the foot, above the account; every
    // other destination sits at the top. Order within each group is Rust's.
    const group = children.length === 0 ? nav : foot;
    group.append(navItem(item.id, label, "rail__item", onSelect, ICON[item.id]));

    if (children.length === 0) continue;
    const sub = el("div", "rail__sub");
    // Which section owns this sub-nav, so setActive can show it only while
    // that section is the one showing: the comp draws no sub-nav under a
    // resting Settings and an open one under an active Settings
    // (design/registry.md "Section sub-nav", corrected 2026-10-02). It starts
    // hidden; setActive runs before a person sees the rail.
    sub.dataset.parent = item.id;
    sub.hidden = true;
    for (const child of children) {
      sub.append(navItem(child.id, LABEL[child.id], "rail__item rail__item--sub", onSelect));
    }
    group.append(sub);
  }

  const account = el("div", "account");
  foot.append(account);
  root.replaceChildren(watermark(), nav, foot);
  return account;
}

/** Mark one destination as the one showing, and nothing else.
 *
 *  Two signals carry it, per design/design-system.md's mandate: the raised rail
 *  fill, which is a lightness change and not a hue, and `aria-current`, which
 *  is what a screen reader reads out. The active parent of an active child is
 *  marked too, so a person can see which section they are inside. */
export function setActive(root, id) {
  const items = root.querySelectorAll("[data-destination]");
  for (const item of items) {
    const own = item.dataset.destination;
    const active = own === id;
    // "settings" is the section holding "settings.dictation".
    const holds = id.startsWith(own + ".");
    item.classList.toggle("rail__item--active", active);
    item.classList.toggle("rail__item--within", !active && holds);
    if (active) {
      item.setAttribute("aria-current", "page");
    } else {
      item.removeAttribute("aria-current");
    }
  }

  // The sub-nav of a section is on screen only while that section is the one
  // showing, which is what the comp's two artboards draw (design/registry.md
  // "Section sub-nav", corrected 2026-10-02). The parent item says so in words
  // too, through aria-expanded.
  for (const sub of root.querySelectorAll(".rail__sub")) {
    const parent = sub.dataset.parent;
    const open = id === parent || id.startsWith(parent + ".");
    sub.hidden = !open;
    const item = itemFor(root, parent);
    if (item) item.setAttribute("aria-expanded", open ? "true" : "false");
  }
}

/** The element for one destination, so the surface can borrow its wording as
 *  its own accessible name rather than this build inventing a heading. */
export function itemFor(root, id) {
  return root.querySelector('[data-destination="' + id + '"]');
}

/** Fold or unfold one section's sub-nav, for a press on the section a person
 *  is already inside (design/registry.md "Section sub-nav", amended
 *  2026-10-02 by the user). The screen is untouched: this moves nothing but
 *  the list. Any navigation afterwards goes through setActive, which puts the
 *  sub-nav back to its rule, so entering a section afresh always arrives
 *  unfolded. */
export function toggleSub(root, parentId) {
  const sub = root.querySelector('.rail__sub[data-parent="' + parentId + '"]');
  if (!sub) return;
  sub.hidden = !sub.hidden;
  const item = itemFor(root, parentId);
  if (item) item.setAttribute("aria-expanded", sub.hidden ? "false" : "true");
}

/** Put the comp's faint count at the right edge of one nav item, or clear it
 *  by passing null. The number is always real and always redundant to the
 *  label (design/registry.md "Nav count"): the caller hands in a count it got
 *  from the feature that owns the data, never an estimate. */
export function setCount(root, id, count) {
  const item = itemFor(root, id);
  if (!item) return;
  let slot = item.querySelector(".rail__item-count");
  if (!slot) {
    slot = el("span", "rail__item-count");
    item.append(slot);
  }
  slot.textContent =
    typeof count === "number" ? count.toLocaleString("en-US") : "";
}

function navItem(id, label, className, onSelect, iconPaths) {
  const button = el("button", className);
  button.type = "button";
  button.dataset.destination = id;
  button.id = "rail-" + id.replace(/\./g, "-");
  if (iconPaths) button.append(icon(iconPaths));
  button.append(withText(el("span", "rail__item-label"), label));
  button.addEventListener("click", () => onSelect(id));
  return button;
}

/** A stroke icon in the item's own ink, so the active state's teal reaches it
 *  through CSS. Decoration beside a label, so a screen reader skips it. */
function icon(paths) {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("class", "rail__icon");
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
