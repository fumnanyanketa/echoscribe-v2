// EchoScribe interface shell.
//
// One window, one screen at a time. On load, after a reload, and whenever the
// auth state changes, this asks the Rust core what state the person is in and
// mounts the matching screen. Nothing else in the app is reachable except
// through here, so when nobody is signed in the only screen that exists is the
// sign-in screen (record 0003 AC-1).

import { mountSignIn, mountSignedIn } from "./sign-in/sign-in.js";
import { mountMicError, MIC_ERROR_KINDS } from "./dictate/mic-error.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const app = document.getElementById("app");

let currentUnmount = null;
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
  } else {
    currentUnmount = await mountSignIn(app, {
      startInWaiting: state.state === "signing_in",
      returnedReason: pendingNotice,
    });
    pendingNotice = null;
  }
}

function unmountCurrent() {
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
// The microphone would not open (record 0002 AC-28). Rust brings this window
// to the front; this mounts the screen with the code, the sentence and the one
// action. Only the four microphone kinds are this screen's to show. The screen
// is left when Try again succeeds, or when an auth change re-renders the shell.
listen("dictation:error", (event) => {
  const payload = event.payload || {};
  if (!MIC_ERROR_KINDS.includes(payload.kind)) return;
  unmountCurrent();
  currentUnmount = mountMicError(app, {
    kind: payload.kind,
    message: payload.message,
    onCleared: () => render(),
  });
});
// The core reached the point of knowing it cannot reach Clerk this launch. The
// screen is already showing the stored identity; this makes sure the "working
// offline" marker is on it (record 0003 AC-14).
listen("auth:offline", () => render());

render();
