// The floating pill's own script.
//
// It listens for the events the Rust core sends and draws them. That is all it
// does. The pill's mouse behaviour is entirely in Rust
// (src-tauri/src/dictate/pill_mouse.rs): this page never sees a mouse event,
// because the web view is kept out of the input path so it can never take focus
// from the app the person is typing into (record 0002 AC-27). Nothing here asks
// to be dragged, and no coordinate is ever computed or sent.
//
// No audio ever reaches this page. `dictation:level` carries a single number
// between 0 and 1 saying how loud the last 60 ms were, and nothing else. There
// is no way to hear, save or reconstruct anything from it.
//
// One thing here changes without an event arriving, and it is the only one: the
// chip's clock, which ticks once a second while a dictation is open. It is
// stopped by every ending and cleared on close, so there is one path out and
// not two.
//
// The transcript is drawn and then gone (record 0002 AC-33, and the Risk
// section). The grey interim line is never typed, never stored, never logged
// and never read by anything but this page; it lives in these two spans and
// nowhere else. Everything arriving on these events is outside input, so it
// only ever lands via textContent, never as markup.

const { event } = window.__TAURI__;

const pill = document.querySelector(".pill");
const bars = [...document.querySelectorAll(".pill__meter i")];
const label = document.querySelector(".pill__label");
const line = document.querySelector(".pill__line");
const finalSpan = document.querySelector(".pill__final");
const interimSpan = document.querySelector(".pill__interim");
const faultCode = document.querySelector(".pill__fault-code");
const faultText = document.querySelector(".pill__fault-text");
const chip = document.querySelector(".pill-chip");
const chipElapsed = document.querySelector(".pill-chip__elapsed");
const chipCount = document.querySelector(".pill-chip__count");

// Silence is a thin flat line rather than an empty gap: an instrument reading
// zero, not a broken one (design/design-system.md, "flat at silence").
const FLAT = 0.08;

// The last 18 readings, oldest first. One reading arrives every 60 ms, so the
// meter shows roughly the last second of loudness, newest at the right.
let recent = bars.map(() => FLAT);

function draw() {
  recent.forEach((value, i) => bars[i].style.setProperty("--bar", value));
}

function flatten() {
  recent = bars.map(() => FLAT);
  draw();
}

// Whether the pill is on screen at all. Broadcast events also fire when the
// microphone never opened and the pill is hidden; none of them are this page's
// to draw then, because a pill on screen means the microphone is open.
function showing() {
  return pill.dataset.state !== "idle";
}

// The locked front of the sentence: every finalised phrase so far, joined by
// the same single space the typing path uses, so the line reads exactly as the
// document does.
let settled = "";

function drawTranscript() {
  finalSpan.textContent = settled;
  // The fade only once the line genuinely overflows; a short line sits flush
  // left with no mask, which is the drawn short state.
  //
  // Measured on the inner row, not the outer box. The inner row hangs out of
  // the outer box to the LEFT (the outer's flex-end pins its right edge), and
  // scrollWidth never counts left-side overflow in left-to-right writing, so
  // the outer box reports no overflow while visibly overflowing. The inner
  // row is sized by its content, so its own width against the outer box's is
  // the real question. Found live 2026-10-01, together with the shrink bug in
  // pill.css that had kept this line from ever sliding.
  const inner = line.firstElementChild;
  line.dataset.overflowing = String(inner.scrollWidth > line.clientWidth + 1);
}

/* ---- The elapsed and count chip (AC-36) ------------------------------
 *
 * design/registry.md's `Elapsed and word count`, in the band already reserved
 * beneath the shell. Two figures: how long this dictation has been going, and
 * how many characters have been typed.
 *
 * The count is Rust's and arrives on `dictation:text`. It is a count of
 * characters and not of words, because no rule for counting words is true in
 * every language this app offers, and it is taken from the same string that
 * becomes the history row, so the figure here and the figure History shows
 * afterwards cannot disagree (record 0002's twentieth amendment; record 0007's
 * first). It counts finalised wording only: the grey interim tail is revised
 * as Deepgram changes its mind, so a count including it would fall while a
 * person is still speaking.
 *
 * The clock is this page's, and it is the one thing here that changes without
 * an event arriving. A clock is not a decision and a per second event from
 * Rust would be traffic for a value this page can read off its own wall. It
 * ticks from `dictation:opened`, stops at whatever ends the dictation, and is
 * cleared on `dictation:closed`. */

/** After how long the chip appears (design/registry.md). Before this, the
 *  band is simply empty, which it already was. */
const CHIP_AFTER_MS = 20000;

let startedAt = 0;
let ticking = 0;

/** `M:SS`, the comp's own shape. There is no hour case to write: AC-8 closes a
 *  dictation at 5 minutes, so the largest figure this can show is 5:00. */
function elapsedOf(ms) {
  const seconds = Math.max(0, Math.floor(ms / 1000));
  const rest = seconds % 60;
  return Math.floor(seconds / 60) + ":" + String(rest).padStart(2, "0");
}

function tick() {
  const ms = Date.now() - startedAt;
  // Measured against the clock rather than counted up per interval, so a
  // delayed timer shows the right time rather than a slow one.
  if (ms < CHIP_AFTER_MS) return;
  chipElapsed.textContent = elapsedOf(ms);
  chip.dataset.shown = "true";
}

/** Every ending freezes the chip where it is. The pill holds its last words
 *  for two seconds under record 0002's eleventh amendment and the chip is part
 *  of what is held: the dictation is over, and a clock still running under a
 *  finished transcript would be the pill asserting something false. */
function freezeChip() {
  clearInterval(ticking);
  ticking = 0;
}

function clearChip() {
  freezeChip();
  chip.dataset.shown = "false";
  chipElapsed.textContent = "";
  chipCount.textContent = "";
  startedAt = 0;
}

event.listen("dictation:opened", () => {
  clearChip();
  // Nothing has been typed yet, and the chip says so rather than leaving a
  // dangling separator: a dictation can reach 20 seconds with no finalised
  // wording at all, and "0:20 · " with nothing after it would look broken
  // where "0 characters" is simply true.
  chipCount.textContent = "0 characters";
  startedAt = Date.now();
  ticking = setInterval(tick, 1000);
  flatten();
  settled = "";
  finalSpan.textContent = "";
  interimSpan.textContent = "";
  faultCode.textContent = "";
  faultText.textContent = "";
  label.textContent = "MIC OPEN";
  pill.dataset.state = "open";
});

event.listen("dictation:level", ({ payload }) => {
  // Anything the core did not send as a usable number reads as silence, rather
  // than as an undefined bar height.
  const level = Number(payload?.level);
  const height = Number.isFinite(level) ? Math.min(Math.max(level, 0), 1) : 0;
  recent = [...recent.slice(1), FLAT + height * (1 - FLAT)];
  draw();
});

// One finalised phrase (record 0002 AC-33: the line visibly hardens left to
// right). Broadcast, so it also fires with the pill hidden; ignored then.
event.listen("dictation:text", ({ payload }) => {
  if (!showing()) return;
  const phrase = typeof payload?.text === "string" ? payload.text : "";
  if (phrase === "") return;
  settled = settled === "" ? phrase : settled + " " + phrase;
  interimSpan.textContent = "";
  pill.dataset.state = "words";
  drawTranscript();
  // The count is Rust's, off the string it is about to save. Nothing is counted
  // here: `String.length` counts UTF-16 code units, so an emoji would come out
  // as two here and as one in the history row, for the same dictation. A
  // payload without a usable number leaves the last figure standing rather
  // than replacing it with a guess.
  // The type and not `Number()`: Rust sends null for the one case where it
  // could not take the lock to count, and `Number(null)` is 0, which would
  // print "0 characters" over a dictation that has words in it.
  const characters = payload?.characters;
  if (typeof characters === "number" && Number.isFinite(characters)) {
    chipCount.textContent =
      characters === 1 ? "1 character" : characters + " characters";
  }
});

// The wording Deepgram has not yet settled: grey, dotted, replaced as it goes,
// and never anything else (record 0002 AC-33).
event.listen("dictation:interim", ({ payload }) => {
  if (!showing()) return;
  const tail = typeof payload?.text === "string" ? payload.text : "";
  if (tail === "") return;
  interimSpan.textContent = settled === "" ? tail : " " + tail;
  pill.dataset.state = "words";
  drawTranscript();
});

// A word ending: the drawn error pill, a mono code over one sentence, no
// action at all (design/registry.md "Error pill"). Rust holds the pill open
// long enough to read it, then closes it.
function showFault(code, message) {
  faultCode.textContent = code;
  faultText.textContent = message;
  pill.dataset.state = "fault";
  freezeChip();
}

// Typing was refused (record 0002 AC-20). The payload carries the fixed code
// and sentence; nothing is worded here.
event.listen("dictation:blocked", ({ payload }) => {
  if (!showing()) return;
  showFault(String(payload?.code || ""), String(payload?.message || ""));
});

// Something ended the dictation while the microphone was open (record 0002
// AC-13, AC-14, AC-30). A microphone kind is the device dying: the drawn MIC
// STOPPED state, which keeps the transcript's last words and carries no code,
// because the code and the action belong to the EchoScribe window that
// follows. Everything else is a word ending with its fixed code and sentence.
// With the pill hidden this event is the microphone failing to open, which is
// never read on a pill (AC-15).
event.listen("dictation:error", ({ payload }) => {
  if (!showing()) return;
  const kind = String(payload?.kind || "");
  if (kind.startsWith("microphone_") || kind === "no_microphone_found") {
    label.textContent = "MIC STOPPED";
    flatten();
    pill.dataset.state = "stopped";
    freezeChip();
    return;
  }
  showFault(String(payload?.code || ""), String(payload?.message || ""));
});

event.listen("dictation:closed", () => {
  clearChip();
  pill.dataset.state = "idle";
  label.textContent = "MIC OPEN";
  flatten();
  settled = "";
  finalSpan.textContent = "";
  interimSpan.textContent = "";
  faultCode.textContent = "";
  faultText.textContent = "";
});
