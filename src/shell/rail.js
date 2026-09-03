// The nav rail: the brand lockup, the nav items, and the sub-nav beneath them
// (design/registry.md "Nav rail", "Nav item", "Section sub-nav").
//
// Rust hands out identifiers and this file decides what a person reads, the
// same division as `get_hotkey` and the two hotkey rows. So an identifier with
// no wording here is not rendered at all: showing a person a machine
// identifier would be worse than showing them one fewer destination, and Rust
// and this file are meant to change together.
//
// The account block is pinned to the foot and lives next door in
// account-block.js.

/// What a person reads for each identifier `get_rail()` can return.
/// design/registry.md owns this wording. Adding a row here without its screen
/// would put a nav item on screen that opens nothing, which record 0004's AC-2
/// forbids, so a new row arrives with its feature and not before.
const LABEL = {
  settings: "Settings",
  "settings.dictation": "Dictation",
  "settings.transcription": "Transcription",
};

/** Draw the rail into `root` from what `get_rail()` returned.
 *  `onSelect` is called with the identifier of whatever was clicked.
 *  Returns the account block's own element, so the caller can fill it. */
export function mountRail(root, view, onSelect) {
  const brand = el("div", "rail__brand");
  brand.append(
    el("span", "rail__brand-dot"),
    withText(el("span", "rail__brand-word"), "EchoScribe"),
  );

  const nav = el("div", "rail__nav");
  for (const item of view.items || []) {
    const label = LABEL[item.id];
    if (!label) {
      // Rust offers a destination this build has no wording for. Say so where a
      // developer will see it and draw nothing, rather than put an identifier
      // in front of a person.
      console.warn("rail: no wording for the destination " + item.id);
      continue;
    }
    nav.append(navItem(item.id, label, "rail__item", onSelect));

    const children = (item.children || []).filter((child) => LABEL[child.id]);
    if (children.length === 0) continue;
    const sub = el("div", "rail__sub");
    for (const child of children) {
      sub.append(navItem(child.id, LABEL[child.id], "rail__item rail__item--sub", onSelect));
    }
    nav.append(sub);
  }

  const account = el("div", "account");
  root.replaceChildren(brand, nav, account);
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
}

/** The element for one destination, so the surface can borrow its wording as
 *  its own accessible name rather than this build inventing a heading. */
export function itemFor(root, id) {
  return root.querySelector('[data-destination="' + id + '"]');
}

function navItem(id, label, className, onSelect) {
  const button = el("button", className);
  button.type = "button";
  button.dataset.destination = id;
  button.id = "rail-" + id.replace(/\./g, "-");
  button.append(withText(el("span", "rail__item-label"), label));
  button.addEventListener("click", () => onSelect(id));
  return button;
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
