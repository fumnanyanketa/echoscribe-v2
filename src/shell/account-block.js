// The account block, pinned to the foot of the nav rail
// (design/registry.md "Account block", "Offline row").
//
// The comp's one row and nothing else: the initials disc and the display name
// (comp line 253). Both come back on `get_auth_state()`'s AccountView, and it
// never sees a token. The mail address, the signed-in date and Sign out moved
// to the account card on the Settings, Transcription surface on 2026-10-02,
// when the user chose the comp over decision 4 of the 2026-10-01 patrol
// (design/patrols/2026-10-02-rail-account.md); account-card.js beside this
// file draws them now.
//
// It holds the app's one "working offline" sign, which moved here from the
// signed-in stub in src/sign-in/sign-in.js when record 0004 deleted it. That
// sign is record 0003's AC-14 and stays record 0003's criterion; this is only
// where it lives now. The rail is the one surface that is on screen whether or
// not a person visits a section, which is what "the app shows it is working
// offline" needs. That reasoning never depended on Sign out being here, so the
// sign did not move with it.

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

  const identity = el("div", "account__identity");
  identity.append(
    withText(el("span", "account__initials"), account.initials),
    withText(el("span", "account__name"), account.display_name),
  );

  root.replaceChildren(offlineRow, identity);
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

function el(tag, className) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

function withText(node, str) {
  node.textContent = str;
  return node;
}
