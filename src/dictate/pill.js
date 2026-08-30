// The floating pill's own script.
//
// It listens for the three events the Rust core sends and draws them. That is
// all it does. The pill's mouse behaviour is entirely in Rust
// (src-tauri/src/dictate/pill_mouse.rs): this page never sees a mouse event,
// because the web view is kept out of the input path so it can never take focus
// from the app the person is typing into (record 0002 AC-27). Nothing here asks
// to be dragged, and no coordinate is ever computed or sent.
//
// No audio ever reaches this page. `dictation:level` carries a single number
// between 0 and 1 saying how loud the last 60 ms were, and nothing else. There
// is no way to hear, save or reconstruct anything from it.

const { event } = window.__TAURI__;

const pill = document.querySelector(".pill");
const bars = [...document.querySelectorAll(".pill__meter i")];

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

event.listen("dictation:opened", () => {
  flatten();
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

event.listen("dictation:closed", () => {
  pill.dataset.state = "idle";
  flatten();
});
