// EchoScribe interface shell.
//
// One window, one screen at a time. On load, after a reload, and whenever the
// auth state changes, this asks the Rust core what state the person is in and
// mounts the matching screen. Nothing else in the app is reachable except
// through here, so when nobody is signed in the only screen that exists is the
// sign-in screen (record 0003 AC-1).

import { mountSignIn, mountSignedIn } from "./sign-in/sign-in.js";
import { mountMicError, MIC_ERROR_KINDS } from "./dictate/mic-error.js";
import { mountKeySetup } from "./dictate/key-setup.js";
import {
  mountDeepgramError,
  isDeepgramErrorKind,
} from "./dictate/deepgram-error.js";

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
    currentUnmount = mountSignedIn(app, state);
    currentScreen = "signed-in";
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
// and the one action. Three routes, each drawn in design/registry.md:
//
//   * a microphone kind, whether the microphone would not open or died mid
//     dictation, is the microphone error screen;
//   * a saved key that stopped being accepted mid dictation is the key setup
//     screen, where a new key can actually be pasted (AC-13's Replace key);
//   * every other mid dictation Deepgram ending is the Deepgram error screen.
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
  if (MIC_ERROR_KINDS.includes(payload.kind)) {
    mountMicErrorScreen(payload.kind, payload.message);
    return;
  }
  if (payload.kind === "deepgram_key_rejected") {
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
  }
  if (isDeepgramErrorKind(payload.kind)) {
    unmountCurrent();
    currentUnmount = mountDeepgramError(app, {
      code: payload.code,
      message: payload.message,
      action: payload.action,
      onCleared: () => render(),
      onMicError: (err) => mountMicErrorScreen(err.kind, err.message),
    });
    currentScreen = "deepgram-error";
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
// Only the four microphone kinds clear on this signal, which is why it checks
// the mounted screen rather than clearing whatever is there. An open
// microphone is no proof that a Deepgram allowance is back, so the spent
// allowance error clears on the first finalised words instead, in milestone 4.
// Its kind is deliberately not named anywhere in this file: a source guard in
// src-tauri/src/dictate/mod.rs keeps it out, so nobody can quietly widen this
// listener to cover it.
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
