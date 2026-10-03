// The account card on the Settings, Transcription surface (design/registry.md
// "Account card"; design/patrols/2026-10-02-rail-account.md).
//
// The comp always drew the mail address, the signed-in line and Sign out here:
// a bordered card under a mono ACCOUNT eyebrow, beneath the Deepgram key
// section (comp lines 550 to 566). They sat in the rail foot until 2026-10-02,
// when the user chose the comp over decision 4 of the 2026-10-01 patrol. The
// rail keeps only the disc and the name, drawn by account-block.js beside this
// file; this lives in the shell too because it is the same account, shown by
// the same window, from the same one read.
//
// Everything shown comes from `get_auth_state()`'s AccountView, fetched once by
// dashboard.js and handed in. Sign out calls `sign_out()` and nothing else,
// exactly as it did in the rail.

const { invoke } = window.__TAURI__.core;

/** The comp's own static copy beneath the card, ratified with the comp as the
 *  pixel-for-pixel authority on 2026-10-01 (design/registry.md "Account card").
 *  If it is ever to say something else, that is wording and goes to
 *  /architect. */
const CAPTION =
  "That is the whole Account section. Nothing to bill, nothing to upgrade.";

/** Draw the account card into `root` from an auth state. */
export function mountAccountCard(root, state) {
  const account = state.account;

  const eyebrow = withText(el("div", "acard__eyebrow"), "ACCOUNT");
  eyebrow.id = "acard-eyebrow";

  const who = el("div", "acard__who");
  who.append(
    withText(el("span", "acard__name"), account.display_name),
    withText(el("p", "acard__email selectable"), account.email),
  );

  const identity = el("div", "acard__identity");
  identity.append(withText(el("span", "acard__initials"), account.initials), who);

  const since = withText(
    el("p", "acard__since"),
    "Signed in since " + formatDate(account.signed_in_since),
  );

  const signOut = el("button", "acard__signout");
  signOut.type = "button";
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

  const card = el("div", "acard__card");
  card.append(identity, since, signOut);

  const section = el("section", "acard");
  section.setAttribute("aria-labelledby", eyebrow.id);
  section.append(eyebrow, card, withText(el("p", "acard__caption"), CAPTION));

  root.replaceChildren(section);
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

/** The signed-in date in the machine's own locale. Moved here from
 *  account-block.js with the line that uses it; the rail no longer shows a
 *  date. transcription-settings.js keeps its own copy on purpose: AGENTS.md
 *  makes something shared when a third feature needs it, and two is a
 *  coincidence. */
function formatDate(iso) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}
