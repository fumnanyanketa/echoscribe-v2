// The mid dictation Deepgram error screen (record 0002 AC-13, AC-14, AC-30;
// design/registry.md "Mid dictation Deepgram errors"). Comes forward after the
// pill closes on a mid dictation Deepgram ending, because there is something
// the person can do: the pill carries no action, so every action lives here.
//
// The wording comes from Rust on the event, never from here: the codes and
// sentences are fixed by the record and live in one place, deepgram_key.rs.
// The event also names the one action for each code, so this screen renders
// the pairing rather than deciding it.
//
// The screen is a claim that dictation cannot happen, so it does not wait to
// be dismissed: it clears the moment finalised words come back from Deepgram,
// which is the record's own proof that the thing it complained about works
// (the clearing table; the shell listens for `dictation:text`).

const { invoke } = window.__TAURI__.core;

/** Whether this kind is this screen's to show. Every mid dictation Deepgram
 *  kind lands here except the rejected key, which the registry sends to the
 *  key setup screen, where a new key can actually be pasted. */
export function isDeepgramErrorKind(kind) {
  return (
    typeof kind === "string" &&
    kind.startsWith("deepgram_") &&
    kind !== "deepgram_key_rejected"
  );
}

/** Mount the Deepgram error screen into `root`.
 *  `options.code`, `options.message` and `options.action` come off the
 *  `dictation:error` event. `options.onCleared` is called when Try again has
 *  started dictation, so the shell can show whatever is true now.
 *  `options.onMicError` is called when Try again failed on the microphone
 *  instead, with the same shaped error, so the shell can mount the microphone
 *  error screen. Returns an unmount function. */
export function mountDeepgramError(root, options) {
  ensureStyles();

  const ctx = {
    root,
    onCleared: options.onCleared,
    onMicError: options.onMicError,
    disposed: false,
  };

  draw(ctx, options.code, options.message, options.action);

  return function unmount() {
    ctx.disposed = true;
  };
}

function draw(ctx, code, message, action) {
  const column = el("section", "dgerr");
  column.append(brandLockup());

  // The same three parts in the same order as the sign-in, microphone and key
  // setup error lines: a mono code, one plain sentence, then the one action
  // beneath (design/registry.md "Deepgram error line").
  const line = el("div", "dgerr__error");
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(
    withText(el("span", "dgerr__error-code"), String(code || "")),
    withText(el("p", "dgerr__error-text"), message || ""),
  );
  column.append(line);

  // Exactly one action per code, never two, fixed by the record's tables and
  // named on the event (design/registry.md "Deepgram error action").
  const console = action === "open_deepgram_console";
  const button = console
    ? makeButton("dgerr__btn dgerr__btn--primary", "Open Deepgram console", openConsole)
    : makeButton("dgerr__btn dgerr__btn--primary", "Try again", (b) => retry(ctx, b));
  const row = el("div", "dgerr__action");
  row.append(button);
  column.append(row);

  ctx.root.replaceChildren(column);
  // The window was brought forward for exactly this action, so hand the
  // keyboard straight to it.
  button.focus();
}

// "Open Deepgram console" hands off to the browser and the screen stays as it
// is, exactly as "Open Windows settings" does on the microphone screen; the
// registry gives it no busy state.
async function openConsole() {
  try {
    await invoke("open_deepgram_console");
  } catch (_) {
    // Rust already says what happened on stderr; the screen stays, which is
    // the drawn behaviour, and the action remains available.
  }
}

// Try again goes through try_start like every other way into dictation
// (record 0002 AC-14). It waits on the microphone and the connection, so the
// button goes busy (design/registry.md "Deepgram error, trying again").
async function retry(ctx, button) {
  setBusy(button, "Starting dictation");
  try {
    await invoke("retry_dictation");
  } catch (err) {
    if (ctx.disposed) return;
    if (err && err.kind === "no_deepgram_key") {
      // The key was cleared in the meantime. Rust has already sent
      // `dictation:needs_key` and the shell is mounting the setup screen, so
      // doing anything here would race it.
      return;
    }
    if (err && err.kind) {
      // The microphone would not open this time. That is the microphone error
      // screen's to show, and the shell owns which screen is mounted.
      ctx.onMicError(err);
      return;
    }
    ctx.onCleared();
    return;
  }
  if (ctx.disposed) return;
  ctx.onCleared();
}

/** The busy state from design/registry.md "Primary button, on dark, busy".
 *  Deliberately not the `disabled` attribute: that drops focus and strands
 *  anyone using a keyboard or a screen reader mid-action. */
function setBusy(button, label) {
  button.classList.add("dgerr__btn--busy");
  button.setAttribute("aria-busy", "true");
  button.setAttribute("aria-disabled", "true");
  button.textContent = label;
}

function brandLockup() {
  const brand = el("div", "dgerr__brand");
  brand.append(
    el("span", "dgerr__brand-dot"),
    withText(el("span", "dgerr__brand-word"), "EchoScribe"),
  );
  return brand;
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
  if (document.getElementById("dgerr-css")) return;
  const link = document.createElement("link");
  link.id = "dgerr-css";
  link.rel = "stylesheet";
  link.href = "dictate/deepgram-error.css";
  document.head.appendChild(link);
}
