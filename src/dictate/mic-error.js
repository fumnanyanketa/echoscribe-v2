// The microphone error screen (record 0002 AC-28, AC-29; design/registry.md
// "Microphone error"). The first screen in the EchoScribe window that is not
// sign in. Shown when the microphone will not open: Rust brings the window to
// the front and this draws the code, the sentence and the one action.
//
// The wording comes from Rust on the event or the command error, never from
// here: the four sentences are fixed by the record and live in one place,
// microphone.rs. The code shown is the error kind itself, uppercased, so a
// person can quote it.

const { invoke } = window.__TAURI__.core;

// The four kinds, and only these four. An error of any other kind is not a
// microphone error and this screen never shows it.
export const MIC_ERROR_KINDS = [
  "microphone_blocked_by_windows",
  "microphone_in_use_by_another_app",
  "no_microphone_found",
  "microphone_unavailable",
];

/** Mount the microphone error screen into `root`.
 *  `options.kind` and `options.message` come off the `dictation:error` event.
 *  `options.onCleared` is called when the screen is done: the microphone
 *  opened on Try again, so dictation is running and the shell should show
 *  whatever is true now. Returns an unmount function. */
export function mountMicError(root, options) {
  ensureStyles();

  const ctx = {
    root,
    onCleared: options.onCleared,
    disposed: false,
  };

  draw(ctx, options.kind, options.message);

  return function unmount() {
    ctx.disposed = true;
  };
}

function draw(ctx, kind, message) {
  const column = el("section", "micerr");
  column.append(brandLockup());

  // The same three parts in the same order as the sign-in error line: a mono
  // code, one plain sentence, then the one action beneath. aria-live so a
  // failed retry that swaps the wording is read out, not just repainted.
  const line = el("div", "micerr__error");
  line.setAttribute("aria-live", "polite");
  line.setAttribute("aria-atomic", "true");
  line.append(
    withText(el("span", "micerr__error-code"), String(kind || "").toUpperCase()),
    withText(el("p", "micerr__error-text"), message || ""),
  );
  column.append(line);

  // Exactly one action, always, and never two (AC-29).
  const blocked = kind === "microphone_blocked_by_windows";
  const button = blocked
    ? makeButton("micerr__btn micerr__btn--primary", "Open Windows settings", openPrivacyPage)
    : makeButton("micerr__btn micerr__btn--primary", "Try again", (b) => retry(ctx, b));
  const row = el("div", "micerr__action");
  row.append(button);
  column.append(row);

  ctx.root.replaceChildren(column);
  // The window was brought forward for exactly this action, so hand the
  // keyboard straight to it.
  button.focus();
}

// "Open Windows settings" hands off to Windows and the screen stays as it is;
// the registry gives it no busy state.
async function openPrivacyPage() {
  try {
    await invoke("open_microphone_privacy_settings");
  } catch (_) {
    // Rust already says what happened on stderr; the screen stays, which is
    // the drawn behaviour, and the action remains available.
  }
}

// Try again opens the microphone, which takes a moment, so the button goes
// busy ("Microphone error, waiting"). Both endings are drawn ("Microphone
// error, retried"): it opens and the window leaves this screen, or it fails
// again and the screen shows whatever the failure now is.
async function retry(ctx, button) {
  setBusy(button, "Opening microphone");
  try {
    await invoke("retry_dictation");
  } catch (err) {
    if (ctx.disposed) return;
    if (err && MIC_ERROR_KINDS.includes(err.kind)) {
      draw(ctx, err.kind, err.message);
    } else {
      // Not a microphone error, so not this screen's to show. Let the shell
      // draw whatever is true now.
      ctx.onCleared();
    }
    return;
  }
  if (ctx.disposed) return;
  ctx.onCleared();
}

/** The busy state from design/registry.md "Primary button, on dark, busy".
 *  Deliberately not the `disabled` attribute: that drops focus and strands
 *  anyone using a keyboard or a screen reader mid-action. */
function setBusy(button, label) {
  button.classList.add("micerr__btn--busy");
  button.setAttribute("aria-busy", "true");
  button.setAttribute("aria-disabled", "true");
  button.textContent = label;
}

function brandLockup() {
  const brand = el("div", "micerr__brand");
  brand.append(
    el("span", "micerr__brand-dot"),
    withText(el("span", "micerr__brand-word"), "EchoScribe"),
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
  if (document.getElementById("micerr-css")) return;
  const link = document.createElement("link");
  link.id = "micerr-css";
  link.rel = "stylesheet";
  link.href = "dictate/mic-error.css";
  document.head.appendChild(link);
}
