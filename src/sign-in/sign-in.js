// The sign-in screen and the signed-in confirmation.
//
// The sign-in screen is the only thing reachable when nobody is signed in
// (record 0003 AC-1). It has one button; all credential entry happens on
// Clerk's hosted pages. Four states from design/registry.md: initial, waiting,
// failed, returned.
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

/** Mount the signed-in confirmation. A milestone-1 stand-in for the Account
 *  block, which lands in the nav rail with the dictation feature. */
export function mountSignedIn(root, state) {
  ensureStyles();
  const account = state.account;

  const column = el("section", "signed-in");
  column.append(
    withText(el("h1", "signed-in__heading"), "Signed in as " + account.display_name),
    withText(el("p", "signed-in__email"), account.email),
    withText(
      el("p", "signed-in__since"),
      "Signed in since " + formatDate(account.signed_in_since),
    ),
  );

  if (state.state === "signed_in_offline") {
    const offline = el("p", "signed-in__offline");
    offline.append(
      withText(el("span", "signin__notice-marker"), "!"),
      withText(el("span"), "Working offline"),
    );
    column.append(offline);
  }

  const row = el("div", "signin__action");
  row.append(
    makeButton("signin__btn signin__btn--secondary", "Sign out", async () => {
      try {
        await invoke("sign_out");
      } catch (_) {
        // sign-out always succeeds locally; nothing to show on failure
      }
    }),
  );
  column.append(row);

  root.replaceChildren(column);
  return function unmount() {};
}

function draw(ctx, screen) {
  const column = el("section", "signin");
  column.append(brandLockup());

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
    column.append(withText(el("h1", "signin__heading"), "Sign in to EchoScribe"));
    column.append(
      withText(
        el("p", "signin__body"),
        "Your account keeps your settings, your words and your history together, and they stay on this machine.",
      ),
    );
    column.append(actionRow(makeButton("signin__btn signin__btn--primary", "Sign in", (button) => begin(ctx, button))));
  }

  column.append(
    withText(
      el("p", "signin__caption"),
      "Sign-in opens in your browser. EchoScribe never sees your password.",
    ),
  );

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

function brandLockup() {
  const brand = el("div", "signin__brand");
  brand.append(el("span", "signin__brand-dot"), withText(el("span", "signin__brand-word"), "EchoScribe"));
  return brand;
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

function formatDate(iso) {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

function ensureStyles() {
  if (document.getElementById("signin-css")) return;
  const link = document.createElement("link");
  link.id = "signin-css";
  link.rel = "stylesheet";
  link.href = "sign-in/sign-in.css";
  document.head.appendChild(link);
}
