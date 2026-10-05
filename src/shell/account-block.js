// The account block, pinned to the foot of the nav rail
// (design/registry.md "Account block", "Offline row").
//
// One row that is also a button: the initials disc and the display name.
// Pressing it opens the account menu above it, holding the mail address, the
// signed-in date and Sign out. The user asked for this on 2026-10-05, after
// finding that the name did nothing when clicked and that Sign out was hard
// to find on the Settings, Transcription surface, where the comp had drawn it
// (account-card.js, deleted in the same change). Everything shown comes back
// on `get_auth_state()`'s AccountView, and it never sees a token. Sign out
// calls `sign_out()` and nothing else.
//
// It holds the app's one "working offline" sign, which moved here from the
// signed-in stub in src/sign-in/sign-in.js when record 0004 deleted it. That
// sign is record 0003's AC-14 and stays record 0003's criterion; this is only
// where it lives now. The rail is the one surface that is on screen whether or
// not a person visits a section, which is what "the app shows it is working
// offline" needs.

const { invoke } = window.__TAURI__.core;

// Fixed at design time, both of them (design/registry.md "Offline row"). The
// sentence never claims the machine has no internet, because the app has
// exactly one notion of being online, Clerk answered, and a machine behind a
// hotel portal is connected and still cannot sign in. It never mentions
// dictation either: Deepgram is a different host.
const OFFLINE_BADGE = "OFFLINE";
const OFFLINE_SENTENCE =
  "Your sign-in could not be checked, so this is your account from last time.";

/** Draw the account block into `root` from an auth state.
 *
 *  `offline` decides whether the offline row shows, and is passed in rather
 *  than read off `state` here: the state is a snapshot from the moment it was
 *  asked for, and by the time this draws, one of the two events may already
 *  have overtaken it. The page holds that one answer and hands it down.
 *
 *  The offline row goes first in the reading order, above the initials and the
 *  name. The block is pinned to the rail foot, so the row's arrival grows the
 *  block upward and nothing already on screen moves. */
export function mountAccountBlock(root, state, offline) {
  const account = state.account;

  const offlineRow = el("div", "account__offline");
  offlineRow.setAttribute("role", "status");
  offlineRow.hidden = true;
  offlineRow.append(
    withText(el("span", "account__offline-badge"), OFFLINE_BADGE),
    withText(el("p", "account__offline-text"), OFFLINE_SENTENCE),
  );

  const menu = accountMenu(account);

  // The row is the menu's one trigger. aria-expanded carries the open state
  // for a screen reader; the chevron carries it for the eye.
  const identity = el("button", "account__identity");
  identity.type = "button";
  identity.setAttribute("aria-haspopup", "menu");
  identity.setAttribute("aria-expanded", "false");
  identity.setAttribute("aria-controls", menu.id);
  identity.append(
    withText(el("span", "account__initials"), account.initials),
    withText(el("span", "account__name"), account.display_name),
    chevron(),
  );

  const setOpen = (open) => {
    menu.hidden = !open;
    identity.setAttribute("aria-expanded", open ? "true" : "false");
  };
  identity.addEventListener("click", () => setOpen(menu.hidden));

  // Escape and a click anywhere else close it, as every menu does. Listeners
  // on the document are added once per mount; the block is mounted once per
  // window, so they never pile up.
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !menu.hidden) {
      setOpen(false);
      identity.focus();
    }
  });
  document.addEventListener("pointerdown", (event) => {
    if (!menu.hidden && !root.contains(event.target)) setOpen(false);
  });

  root.replaceChildren(offlineRow, menu, identity);
  setOffline(root, offline);
}

/** Show or hide the offline row. Called at open from the auth state, and again
 *  on the two events Rust already emits: `auth:offline` when a refresh cannot
 *  reach Clerk, and `auth:signed_in` when a later one gets through. The row
 *  goes the moment a refresh succeeds. */
export function setOffline(root, offline) {
  const row = root.querySelector(".account__offline");
  if (row) row.hidden = !offline;
}

/** The menu above the row: the address, the signed-in date, Sign out. Hidden
 *  until the row is pressed. */
function accountMenu(account) {
  const menu = el("div", "account__menu");
  menu.id = "account-menu";
  menu.setAttribute("role", "menu");
  menu.hidden = true;

  const email = withText(el("p", "account__email selectable"), account.email);
  const since = withText(
    el("p", "account__since"),
    "Signed in since " + formatDate(account.signed_in_since),
  );

  const signOut = el("button", "account__signout");
  signOut.type = "button";
  signOut.setAttribute("role", "menuitem");
  signOut.textContent = "Sign out";
  signOut.addEventListener("click", async () => {
    try {
      await invoke("sign_out");
    } catch (_) {
      // Signing out always succeeds locally: it empties the session and deletes
      // the credential entry before it tells Clerk anything, so there is
      // nothing to show a person here. Rust closes this window either way.
    }
  });

  menu.append(email, since, signOut);
  return menu;
}

/** A small up-pointing chevron, the rail's stroke style. Decoration: the
 *  button's aria-expanded is what says the state. */
function chevron() {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("class", "account__chevron");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("aria-hidden", "true");
  const path = document.createElementNS(ns, "path");
  path.setAttribute("d", "M6 15l6-6 6 6");
  svg.append(path);
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

/** The signed-in date in the machine's own locale. transcription-settings.js
 *  keeps its own copy on purpose: AGENTS.md makes something shared when a
 *  third feature needs it, and two is a coincidence. */
function formatDate(iso) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}
