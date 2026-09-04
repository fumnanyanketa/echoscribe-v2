// The dashboard page: the rail, the account block and the white surface
// (record 0004).
//
// It asks Rust three things and draws what it is told. `get_rail()` for the
// sections that exist and which one to land on, `get_auth_state()` for the five
// things the account block shows, and `sign_out()` when a person presses Sign
// out. It sends nothing else and emits nothing, ever.
//
// Which section is showing is this page's own business and needs no command and
// no event. It is drawing, not a decision.
//
// The window itself is Rust's. It is created, sized, placed, shown and closed
// there, and nothing here knows a coordinate or could ask for one: the
// dashboard's capability grants two event permissions and nothing that touches
// a window.

import { mountRail, setActive, itemFor } from "./rail.js";
import { mountAccountBlock, setOffline } from "./account-block.js";
import { mountDictationSettings } from "../dictate/dictation-settings.js";
import { mountTranscriptionSettings } from "../dictate/transcription-settings.js";

/** Which destination opens which screen. The same shape and the same reason as
 *  rail.js's wording table: Rust hands out identifiers and this side decides
 *  what each one opens, so a destination with no entry here opens nothing
 *  rather than something nobody designed. A screen joins this table with its
 *  own feature, never ahead of it.
 *
 *  A screen belongs to its own feature's folder on both sides, the way
 *  main.js mounts the dictate feature's three dark screens from src/dictate/.
 *  This page is the dashboard's router, and routing is all it does with them. */
const SCREEN = {
  "settings.dictation": mountDictationSettings,
  "settings.transcription": mountTranscriptionSettings,
};

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const rail = document.getElementById("rail");
const surface = document.getElementById("surface");

let accountBlock = null;
// What get_rail() returned. Kept so that pressing a section can be resolved to
// the sub-section that actually holds a screen, off Rust's own structure rather
// than off a shape assumed here.
let railView = null;
// The unmount function of whatever screen the surface is holding.
let unmountScreen = null;
// Whether the app is working offline. `null` means nothing has said yet, which
// is what lets an event that arrives during startup win over the older snapshot
// `get_auth_state` returned.
let offline = null;

function applyOffline() {
  if (accountBlock && offline !== null) setOffline(accountBlock, offline);
}

async function start() {
  // The listeners go up before anything is asked of Rust, and they are awaited.
  // The session's first refresh runs at launch, so `auth:signed_in` can arrive
  // while this page is still loading; a listener registered after that has
  // already missed it, and the offline row would then sit on screen for the
  // rest of a session that is perfectly online. Proved live on 2026-09-03,
  // where exactly that happened.
  await listen("auth:offline", () => {
    offline = true;
    applyOffline();
  });
  await listen("auth:signed_in", () => {
    offline = false;
    applyOffline();
  });
  let view;
  try {
    view = await invoke("get_rail");
  } catch (err) {
    // Every command on this surface refuses when nobody is signed in, and this
    // window only exists while somebody is. So a refusal here means Rust is
    // already closing this window, and drawing anything would be drawing a
    // screen nobody designed.
    console.error("dashboard: the rail could not be read: " + err);
    return;
  }

  railView = view;
  accountBlock = mountRail(rail, view, show);
  show(view.landing);

  try {
    const state = await invoke("get_auth_state");
    if (state.state === "signed_in" || state.state === "signed_in_offline") {
      // The snapshot only fills in when neither event has spoken yet. An event
      // is always the newer truth.
      if (offline === null) offline = state.state === "signed_in_offline";
      mountAccountBlock(accountBlock, state, offline);
    }
  } catch (err) {
    // Same reasoning: the rail is up and usable, and the block that says who
    // you are is the one part that could not be filled.
    console.error("dashboard: the account could not be read: " + err);
  }
}

// Which section is showing. Drawing, not a decision, which is why it needs no
// command and no event.
//
// Every item in the rail opens a screen (record 0004 AC-2), and a section that
// holds sub-sections opens the first of them that has one: pressing Settings
// opens Dictation, and the rail then marks Dictation as the one showing with
// Settings as the section it is inside. Which sub-sections exist is Rust's
// answer, read off what get_rail() returned, so this cannot invent a
// destination.
function show(id) {
  const target = resolve(id);
  setActive(rail, target);
  const item = itemFor(rail, target);
  if (item) {
    // The surface takes its accessible name from the rail item that opened it,
    // so it is named by design/registry.md's own wording rather than by a
    // heading this build invented.
    surface.setAttribute("aria-labelledby", item.id);
  }

  // One screen at a time, and the old one goes before the new one arrives.
  if (unmountScreen) {
    unmountScreen();
    unmountScreen = null;
  }
  surface.replaceChildren();

  const mount = SCREEN[target];
  if (mount) unmountScreen = mount(surface);
}

/** The destination that actually gets drawn when `id` is pressed. `id` itself
 *  when it has a screen, otherwise its first sub-section that has one. */
function resolve(id) {
  if (SCREEN[id]) return id;
  const item = ((railView && railView.items) || []).find((one) => one.id === id);
  const child = ((item && item.children) || []).find((one) => SCREEN[one.id]);
  return child ? child.id : id;
}

// The two events this window listens to, and the only two, are registered at
// the top of `start` so that neither can be missed. Both are already emitted by
// src-tauri/src/sign_in/renewal.rs, so nothing is owed in Rust: `auth:offline`
// when a refresh cannot reach Clerk, `auth:signed_in` when a later one gets
// through. Together they are what makes record 0003's AC-14 sign arrive and go
// while this window is open.
start();
