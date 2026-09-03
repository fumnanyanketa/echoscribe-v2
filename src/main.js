// The small dark window's router.
//
// One screen at a time. On load, after a reload, and whenever the auth state
// changes, this asks the Rust core what state the person is in and mounts the
// matching screen. Nothing else in the app is reachable except through here, so
// when nobody is signed in the only screen that exists is the sign-in screen
// (record 0003 AC-1).
//
// This is the 760x540 window, and record 0004 gave it one job: everything with
// exactly one way forward. Sign in, the Deepgram key setup screen, the
// microphone error screen and the mid-dictation Deepgram error screen. The
// dashboard is a second window with its own page (src/shell/dashboard.js), so
// when a person is signed in with a key saved there is nothing for this window
// to hold and Rust hides it. Rust decides that, not this file.

import { mountSignIn } from "./sign-in/sign-in.js";
import { mountMicError } from "./dictate/mic-error.js";
import { mountKeySetup } from "./dictate/key-setup.js";
import { mountDeepgramError } from "./dictate/deepgram-error.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const app = document.getElementById("app");

let currentUnmount = null;
// Which screen is mounted. A signal that clears one screen must never clear
// another, so every mount below names itself here and the clearing listener
// checks the name before it acts (record 0002 AC-32).
let currentScreen = null;
// Set by the auth:signed_out event so the next sign-in screen can say why the
// person is back on it.
let pendingNotice = null;

async function render() {
  let state;
  try {
    state = await invoke("get_auth_state");
  } catch (err) {
    unmountCurrent();
    app.replaceChildren(fatal("The core did not answer: " + err));
    return;
  }

  unmountCurrent();

  if (state.state === "signed_in" || state.state === "signed_in_offline") {
    // Signed in. Which leaves one pre-shell step that can still be outstanding:
    // a Deepgram key, without which dictation cannot start at all (record 0002
    // AC-9). With a key saved the dashboard is this person's screen, in its own
    // window, and this one has nothing to hold.
    let savedKey;
    try {
      savedKey = await invoke("get_deepgram_key_info");
    } catch (err) {
      app.replaceChildren(fatal("The core did not answer: " + err));
      return;
    }
    if (savedKey) {
      app.replaceChildren();
      currentScreen = "at-rest";
      return;
    }
    currentUnmount = mountKeySetup(app, { onSaved: () => render() });
    currentScreen = "key-setup";
  } else {
    currentUnmount = await mountSignIn(app, {
      startInWaiting: state.state === "signing_in",
      returnedReason: pendingNotice,
    });
    currentScreen = "sign-in";
    pendingNotice = null;
  }
}

function unmountCurrent() {
  currentScreen = null;
  if (currentUnmount) {
    currentUnmount();
    currentUnmount = null;
  }
}

function fatal(message) {
  const section = document.createElement("section");
  section.style.margin = "auto";
  section.style.maxWidth = "24rem";
  section.style.padding = "2rem";
  section.style.textAlign = "center";
  section.style.color = "#f4f4f6";
  const heading = document.createElement("h1");
  heading.textContent = "Could not start";
  const paragraph = document.createElement("p");
  paragraph.textContent = message;
  section.append(heading, paragraph);
  return section;
}

listen("auth:signed_in", () => render());
listen("auth:signed_out", (event) => {
  pendingNotice = (event.payload && event.payload.reason) || null;
  render();
});
// Something ended dictation, or stopped it starting, and this window is where
// it is read (record 0002 AC-13, AC-14, AC-28, AC-30). Rust brings the window
// to the front; this mounts the matching screen with the code, the sentence
// and the one action. Three screens, each drawn in design/registry.md, and the
// event names which one.
//
// Rust worked that out before the event left it, once, in the dictate feature
// where the error kinds are minted (record 0004's second amendment of
// 2026-09-03, and record 0002's fifteenth). This file no longer inspects a
// kind: the clearing table has one copy, and it is Rust's. What is left here is
// each family's pairing with its own proof, in the clearing listeners below,
// because this side clears the screen it is itself holding.
//
// The password refusal never lands here: its event goes to the pill alone and
// the window deliberately stays where it is (AC-20).
function mountMicErrorScreen(kind, message) {
  unmountCurrent();
  currentUnmount = mountMicError(app, {
    kind,
    message,
    onCleared: () => render(),
  });
  currentScreen = "mic-error";
}

listen("dictation:error", (event) => {
  const payload = event.payload || {};
  switch (payload.screen) {
    case "mic-error":
      mountMicErrorScreen(payload.kind, payload.message);
      return;
    case "key-setup":
      unmountCurrent();
      currentUnmount = mountKeySetup(app, {
        onSaved: () => render(),
        initialError: {
          code: payload.code,
          message: payload.message,
          action: payload.action,
        },
      });
      currentScreen = "key-setup";
      return;
    case "deepgram-error":
      unmountCurrent();
      currentUnmount = mountDeepgramError(app, {
        code: payload.code,
        message: payload.message,
        action: payload.action,
        onCleared: () => render(),
        onMicError: (err) => mountMicErrorScreen(err.kind, err.message),
      });
      currentScreen = "deepgram-error";
      return;
    default:
      // No screen named. Either nothing on this window is meant to hold it,
      // which is what the password refusal is, or Rust could not place the
      // kind, and in that case it has already said so on stderr. Nothing is
      // drawn either way: this file cannot pick a screen, which is the price
      // of the clearing table having one copy (record 0002, fifteenth
      // amendment, "What this makes harder").
      return;
  }
});
// The microphone opened (record 0002 AC-32, and the clearing table in the
// record's "The decision"). A microphone error still on screen is a claim that
// dictation cannot start, and this is that claim's own proof that it can, so
// the screen clears itself and the window shows whatever is ordinarily true.
// That is the same ending a successful Try again already has, drawn as
// "Microphone error, retried" in design/registry.md; the button is no longer
// the only door into it.
//
// It clears quietly. Nothing here shows, hides, focuses or moves the window:
// the person is dictating into another app, and this record brings a window
// forward for one reason only, a hotkey press that produced nothing.
//
// Only the microphone screen clears on this signal, which is why it checks the
// mounted screen rather than clearing whatever is there. An open microphone is
// no proof that a Deepgram allowance is back, so the spent allowance error
// clears on the first finalised words instead. No error kind is named anywhere
// in this file, and two source guards in src-tauri/src/dictate/ keep it that
// way: one holds each family to its own proof, the other keeps every kind out
// of here, so nobody can quietly widen this listener to cover another screen.
listen("dictation:opened", () => {
  if (currentScreen !== "mic-error") return;
  // Claimed here, before the await inside render, so a second opening cannot
  // start a second render of the same screen.
  currentScreen = null;
  render();
});
// Finalised words came back from Deepgram (record 0002 AC-32, and the clearing
// table). They are the one proof that clears a Deepgram error screen: an open
// microphone proves nothing about an allowance, a key's permissions or the
// connection, but words that arrived prove the whole stream works. Quiet, like
// every clearing: nothing shows, hides, focuses or moves the window. The key
// setup screen deliberately does not clear on this signal; the clearing table
// gives its errors exactly one proof, a key being accepted and saved.
listen("dictation:text", () => {
  if (currentScreen !== "deepgram-error") return;
  currentScreen = null;
  render();
});
// The hotkey was pressed with no Deepgram key saved (record 0002 AC-9). The
// microphone did not open, no pill appeared and neither sound played; Rust
// brings this window to the front and this mounts the guided setup screen.
// Once Deepgram accepts a key the screen closes and the shell shows whatever
// is true now, because the person was mid task and the hotkey works from here.
listen("dictation:needs_key", () => {
  unmountCurrent();
  currentUnmount = mountKeySetup(app, {
    onSaved: () => render(),
  });
  currentScreen = "key-setup";
});
// The core reached the point of knowing it cannot reach Clerk this launch. The
// screen is already showing the stored identity; this makes sure the "working
// offline" marker is on it (record 0003 AC-14).
listen("auth:offline", () => render());

render();
