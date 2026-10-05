// The sign-in screen.
//
// It is the only thing reachable when nobody is signed in (record 0003 AC-1).
// It has one button; all credential entry happens on Clerk's hosted pages.
// Four states from design/registry.md: initial, waiting, failed, returned.
//
// The signed-in confirmation that used to live here is gone. It was a
// milestone-1 stand-in for the account block, and record 0004 built the real
// one at the foot of the dashboard's nav rail. The name and the "working
// offline" sign live there (src/shell/account-block.js); the mail address, the
// date and Sign out moved to the account card on the Settings, Transcription
// surface on 2026-10-02 (src/shell/account-card.js).
//
// Rust drives the actual sign-in. This module asks it to start or cancel, and
// listens for the outcome. It never sees a token or a Clerk code.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// One sentence per named failure reason (design/registry.md "Sign-in error
// line"). The mono code beside it is the reason key itself.
const FAILURE_SENTENCE = {
  browser_closed: "The browser closed before sign-in finished.",
  timed_out: "Sign-in timed out waiting for the browser.",
  security_check_failed: "A security check on the sign-in did not pass.",
  clerk_rejected: "Clerk did not accept the sign-in.",
  cannot_reach_clerk: "Could not reach Clerk. Check your connection.",
  no_loopback_port: "Could not open a local port to sign in.",
  browser_failed: "Could not open your browser.",
};

// The notice shown above the screen when a person is sent back to it
// (design/registry.md "Sign-in notice line"). A marker and words, never colour
// alone.
const NOTICE = {
  you_signed_out: { tone: "neutral", marker: "•", text: "You signed out." },
  session_ended_elsewhere: {
    tone: "warn",
    marker: "!",
    text: "Your session was ended elsewhere.",
  },
};

/** Mount the sign-in screen into `root`. Returns an unmount function. */
export async function mountSignIn(root, options = {}) {
  ensureStyles();

  const ctx = {
    root,
    returnedNotice: NOTICE[options.returnedReason] || null,
    disposed: false,
    unlisten: null,
  };

  ctx.unlisten = await listen("auth:sign_in_failed", (event) => {
    if (ctx.disposed) return;
    const code = event.payload && event.payload.code;
    draw(ctx, { view: "failed", code });
  });

  draw(ctx, { view: options.startInWaiting ? "waiting" : "initial" });

  return function unmount() {
    ctx.disposed = true;
    if (ctx.unlisten) ctx.unlisten();
  };
}

function draw(ctx, screen) {
  const column = el("section", "signin");
  column.append(watermark(""), watermark("signin__watermark--mirror"));

  if (screen.view === "initial" && ctx.returnedNotice) {
    column.append(noticeLine(ctx.returnedNotice));
  }

  if (screen.view === "failed") {
    column.append(withText(el("h1", "signin__heading"), "Sign-in did not finish"));
    column.append(errorLine(screen.code));
    column.append(actionRow(makeButton("signin__btn signin__btn--primary", "Try again", (button) => begin(ctx, button))));
  } else if (screen.view === "waiting") {
    column.append(withText(el("h1", "signin__heading"), "Signing in"));
    const waiting = el("p", "signin__waiting");
    waiting.append(el("span", "signin__pulse"), withText(el("span"), "Waiting for your browser"));
    column.append(waiting);
    column.append(
      actionRow(
        makeButton("signin__btn signin__btn--secondary", "Cancel", async () => {
          try {
            await invoke("cancel_sign_in");
          } catch (_) {
            // fall through to the initial view regardless
          }
          draw(ctx, { view: "initial" });
        }),
      ),
    );
  } else {
    // The wordmark (the waveform mark beside the app's name) is the heading,
    // then the one action. Since the 2026-10-05 trim there is no "Sign in to"
    // prefix and no paragraph: the button alone says what pressing it does.
    const name = el("h1", "signin__heading signin__wordmark");
    name.append(mark(), withText(el("span"), "EchoScribe"));
    column.append(name);
    column.append(actionRow(makeButton("signin__btn signin__btn--primary", "Sign in", (button) => begin(ctx, button))));
  }

  // The waiting view already says it is waiting for the browser, so the
  // caption would only repeat it there.
  if (screen.view !== "waiting") {
    column.append(withText(el("p", "signin__caption"), "Clicking the button opens your browser to sign in."));
  }

  ctx.root.replaceChildren(column);
}

// Pressing the button does not leave this view at once. Rust checks that Clerk
// answers before it binds a port or opens anything, which takes a couple of
// seconds on a good connection and up to five on a bad one. So the button goes
// busy and the screen stays put; it moves to waiting only once a browser is
// genuinely open, and to failed if Clerk never answered
// (record 0003 AC-15, design/registry.md "Sign-in action").
async function begin(ctx, button) {
  setBusy(button, "Checking connection");
  try {
    await invoke("start_sign_in");
  } catch (err) {
    if (ctx.disposed) return;
    draw(ctx, { view: "failed", code: typeof err === "string" ? err : "clerk_rejected" });
    return;
  }
  if (ctx.disposed) return;
  draw(ctx, { view: "waiting" });
}

/** Put a button into the busy state from design/registry.md "Primary button, on
 *  dark, busy". Deliberately not the `disabled` attribute: that drops focus and
 *  strands anyone using a keyboard or a screen reader mid-action. `aria-disabled`
 *  says the same thing while the button keeps its place and its focus ring, and
 *  `makeButton` ignores the click. */
function setBusy(button, label) {
  button.classList.add("signin__btn--busy");
  button.setAttribute("aria-busy", "true");
  button.setAttribute("aria-disabled", "true");
  button.textContent = label;
}


function noticeLine(notice) {
  const line = el("p", "signin__notice signin__notice--" + notice.tone);
  line.append(
    withText(el("span", "signin__notice-marker"), notice.marker),
    withText(el("span"), notice.text),
  );
  return line;
}

function errorLine(code) {
  const box = el("div", "signin__error");
  box.append(
    withText(el("span", "signin__error-code"), code || "clerk_rejected"),
    withText(el("p", "signin__error-text"), FAILURE_SENTENCE[code] || FAILURE_SENTENCE.clerk_rejected),
  );
  return box;
}

/** The app icon's twelve bars, as fractions of its tallest one, measured off
 *  design/icon-1024.png. The same list as the dashboard rail's watermark
 *  (src/shell/rail.js) and the key setup screen's (src/dictate/key-setup.js):
 *  feature folders do not import from each other, so each holds its own copy,
 *  and a Rust guard in src-tauri/src/sign_in/mod.rs fails the build when the
 *  three copies differ, because the icon's bars are one fact. */
const MARK_BARS = [0.23, 0.47, 0.82, 0.58, 1, 0.74, 0.42, 0.89, 0.63, 0.32, 0.55, 0.25];

// Decoration, hidden from a screen reader: the rail's faded logo, bleeding in
// from the window's left edge and, mirrored, from its right. Replaced the grey
// tile rows on 2026-10-05 at the user's request, so the first-run screens carry
// the same mark the dashboard does.
function watermark(modifier) {
  const node = el("div", modifier ? "signin__watermark " + modifier : "signin__watermark");
  node.setAttribute("aria-hidden", "true");
  for (const height of MARK_BARS) {
    const bar = el("i");
    bar.style.height = Math.round(height * 100) + "%";
    node.append(bar);
  }
  return node;
}

// The waveform mark in teal: the same six bars as the key setup screen's mark
// (src/dictate/key-setup.js), so the two first screens share one motif.
const MARK_HEIGHTS = [12, 24, 34, 20, 28, 14];

function mark() {
  const node = el("div", "signin__mark");
  node.setAttribute("aria-hidden", "true");
  for (const height of MARK_HEIGHTS) {
    const bar = el("i");
    bar.style.height = height + "px";
    node.append(bar);
  }
  return node;
}

function actionRow(button) {
  const row = el("div", "signin__action");
  row.append(button);
  return row;
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

function makeButton(className, label, onClick) {
  const button = el("button", className);
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", () => {
    if (button.getAttribute("aria-disabled") === "true") return;
    onClick(button);
  });
  return button;
}

function ensureStyles() {
  if (document.getElementById("signin-css")) return;
  const link = document.createElement("link");
  link.id = "signin-css";
  link.rel = "stylesheet";
  link.href = "sign-in/sign-in.css";
  document.head.appendChild(link);
}
