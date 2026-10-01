// The Deepgram key setup screen (record 0002 AC-9 to AC-13;
// design/registry.md "Deepgram key setup").
//
// Shown when the hotkey is pressed with no key saved. The microphone did not
// open, no pill appeared and neither sound played: this screen is the whole of
// what happened, and it is the same shape as the microphone error screen, a
// hotkey press that could not start dictation and a window saying why.
//
// The pasted key is a secret from the first keystroke. It is masked as it is
// typed, it is never read back out of the field for any purpose but the one
// check, it is never logged, and the only thing that ever comes back from Rust
// is its last four characters. Nothing here ever puts it on the page.
//
// Every wording for a failed check comes from Rust, never from here. The four
// codes and sentences live in one place, deepgram_key.rs, so no screen can
// invent its own.

const { invoke } = window.__TAURI__.core;

/** Mount the key setup screen into `root`.
 *  `options.onSaved` is called once Deepgram has accepted a key and it has
 *  been stored, so the shell can leave this screen.
 *  `options.initialError` puts an error line on the screen from the first
 *  paint: it is how a saved key that stopped being accepted mid dictation
 *  lands here, code and sentence straight off the `dictation:error` event
 *  (record 0002 AC-13; design/registry.md "Key setup, rejected").
 *  Returns an unmount function. */
export function mountKeySetup(root, options) {
  ensureStyles();

  const ctx = {
    root,
    onSaved: options.onSaved,
    disposed: false,
    // Kept across redraws so a rejected key stays in the field: one typo must
    // not cost a fresh paste (design/registry.md "Key setup, rejected").
    pasted: "",
    error: options.initialError || null,
  };

  draw(ctx);

  return function unmount() {
    ctx.disposed = true;
  };
}

function draw(ctx, focusTarget) {
  const column = el("section", "keysetup");
  column.append(stepRow());

  column.append(
    withText(el("h1", "keysetup__heading"), "Add your Deepgram key"),
    withText(
      el("p", "keysetup__body"),
      "EchoScribe sends what you say to Deepgram, which turns it into text. " +
        "A Deepgram key is how Deepgram knows that usage is yours, so you need " +
        "your own before dictation can start. Your audio goes from this machine " +
        "to your own Deepgram account and nowhere else.",
    ),
  );

  // The error line sits above the action, the same order as the sign-in and
  // microphone error lines: a mono code, one plain sentence, the one action
  // beneath. aria-live so a second failure that swaps the wording is read out
  // and not only repainted.
  const errorLine = el("div", "keysetup__error");
  errorLine.setAttribute("aria-live", "polite");
  errorLine.setAttribute("aria-atomic", "true");
  if (ctx.error) {
    errorLine.append(
      withText(el("span", "keysetup__error-code"), ctx.error.code),
      withText(el("p", "keysetup__error-text"), ctx.error.message),
    );
  }
  column.append(errorLine);

  // A form, so Enter in the field verifies, which is what anyone who has just
  // pasted a key will press.
  const form = el("form", "keysetup__form");
  form.noValidate = true;

  // The field's accessible name. Visually hidden rather than absent: the drawn
  // field carries a placeholder and a SECRET badge, and a placeholder is not a
  // label. Nothing about the look changes.
  const label = withText(el("label", "keysetup__label"), "Deepgram key");
  label.htmlFor = "deepgram-key";

  const field = el("div", "keysetup__field");

  const badge = withText(el("span", "keysetup__badge"), "SECRET");
  // Decoration for a screen reader: the field is already named by its label and
  // announced as a password field, so this would only add noise.
  badge.setAttribute("aria-hidden", "true");

  // type=password is the masking. The key is never rendered in plain text on
  // this screen, at any point, in any state.
  const input = el("input", "keysetup__input");
  input.type = "password";
  input.id = "deepgram-key";
  input.name = "deepgram-key";
  input.placeholder = "Paste your key";
  input.autocomplete = "off";
  input.spellcheck = false;
  input.value = ctx.pasted;
  input.addEventListener("input", () => {
    ctx.pasted = input.value;
    verify.disabled = input.value.trim() === "";
  });

  const verify = el("button", "keysetup__btn keysetup__btn--primary");
  verify.type = "submit";
  verify.textContent = "Verify";
  // Nothing to check yet. The button is genuinely disabled here rather than
  // aria-disabled, because there is no action to announce and no busy state to
  // hold focus for.
  verify.disabled = ctx.pasted.trim() === "";

  // The comp's order inside the field: the violet dot, the key itself, the
  // SECRET badge, then the action at the right edge.
  const dot = el("span", "keysetup__field-dot");
  dot.setAttribute("aria-hidden", "true");
  field.append(dot, input, badge, verify);
  form.append(label, field);
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (verify.getAttribute("aria-disabled") === "true" || verify.disabled) return;
    check(ctx, input, verify);
  });
  column.append(form);

  // AC-13's allowance action, when it has one. It cannot arrive at this screen:
  // Deepgram only returns 402 for a transcription request, so the check this
  // screen makes can never produce it. It is drawn here because the state
  // exists and milestone 4 raises it; leaving it undrawn would mean a screen
  // with an error and no way forward.
  if (ctx.error && ctx.error.action === "open_deepgram_console") {
    const console = makeButton(
      "keysetup__btn keysetup__btn--secondary",
      "Open Deepgram console",
      () => openFixedPage("open_deepgram_console"),
    );
    const row = el("div", "keysetup__action");
    row.append(console);
    column.append(row);
  }

  // AC-9's link out to get a key. A button, not an anchor: it opens the system
  // browser through Rust, which holds the one address it is allowed to open.
  // Navigating this window would take the app somewhere it cannot come back
  // from.
  const link = makeButton(
    "keysetup__link",
    "Get a free key from Deepgram",
    () => openFixedPage("open_deepgram_signup"),
  );
  column.append(link);

  column.append(
    withText(
      el("p", "keysetup__caption"),
      "Your key is kept on this machine, in Windows Credential Manager. " +
        "EchoScribe never shows it in full again once it is saved.",
    ),
  );

  ctx.root.replaceChildren(column);

  // The window was brought forward for exactly this, so hand the keyboard to
  // the field. After a failed check focus goes back to the field too, with the
  // pasted text still in it, so a typo can be corrected without reaching for
  // the mouse.
  if (focusTarget === "field" || !ctx.error) {
    input.focus();
    // Put the caret at the end rather than selecting, so a stray keystroke
    // cannot wipe what was pasted.
    const end = input.value.length;
    try {
      input.setSelectionRange(end, end);
    } catch (_) {
      // Some browsers refuse setSelectionRange on a password field. The caret
      // lands at the end by default there anyway.
    }
  }
}

// Check the pasted key against Deepgram. Verifying is the only way a key is
// ever saved: a key that fails the check is never stored (AC-10, AC-11).
async function check(ctx, input, button) {
  // design/registry.md "Key setup, checking": the button takes the busy
  // primitive and the field is not editable while the check runs.
  setBusy(button, "Checking key");
  input.readOnly = true;

  let lastFour;
  try {
    lastFour = await invoke("save_deepgram_key", { key: ctx.pasted });
  } catch (err) {
    if (ctx.disposed) return;
    // Every failing path leaves nothing saved, which is what the sentence from
    // Rust says. The pasted text stays in the field.
    ctx.error = {
      code: (err && err.code) || "DEEPGRAM_CHECK_FAILED",
      message:
        (err && err.message) ||
        "The check did not succeed and Deepgram did not say why. Nothing was saved.",
      action: (err && err.action) || "try_again",
    };
    draw(ctx, "field");
    return;
  }

  if (ctx.disposed) return;
  // Saved. Rust hands back the last four characters and nothing else; they are
  // not shown here, because on this path the screen closes and the person is
  // returned to what they were doing (design/registry.md "Key setup, saved").
  void lastFour;
  ctx.onSaved();
}

async function openFixedPage(command) {
  try {
    await invoke(command);
  } catch (_) {
    // Rust already says what happened on stderr. The screen stays exactly as it
    // is, which is the drawn behaviour, and the action remains available.
  }
}

/** The busy state from design/registry.md "Primary button, on dark, busy".
 *  Deliberately not the `disabled` attribute: that drops focus and strands
 *  anyone using a keyboard or a screen reader mid-action. */
function setBusy(button, label) {
  button.classList.add("keysetup__btn--busy");
  button.setAttribute("aria-busy", "true");
  button.setAttribute("aria-disabled", "true");
  button.textContent = label;
}

/** The comp's step row in place of a brand lockup: this window is step two of
 *  the first run, sign-in being step one, and both bars are lit because both
 *  steps are reached. The bars are decoration beside the words, so a screen
 *  reader hears the words alone. */
function stepRow() {
  const row = el("div", "keysetup__step");
  row.append(withText(el("span", "keysetup__step-words"), "STEP 2 OF 2"));
  const bars = el("span", "keysetup__step-bars");
  bars.setAttribute("aria-hidden", "true");
  bars.append(el("i"), el("i"));
  row.append(bars);
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
  if (document.getElementById("keysetup-css")) return;
  const link = document.createElement("link");
  link.id = "keysetup-css";
  link.rel = "stylesheet";
  link.href = "dictate/key-setup.css";
  document.head.appendChild(link);
}
