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
  line.dataset.overflowing = String(line.scrollWidth > line.clientWidth + 1);
}

event.listen("dictation:opened", () => {
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
    return;
  }
  showFault(String(payload?.code || ""), String(payload?.message || ""));
});

event.listen("dictation:closed", () => {
  pill.dataset.state = "idle";
  label.textContent = "MIC OPEN";
  flatten();
  settled = "";
  finalSpan.textContent = "";
  interimSpan.textContent = "";
  faultCode.textContent = "";
  faultText.textContent = "";
});
