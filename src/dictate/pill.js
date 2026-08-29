// The floating pill's own script.
//
// It listens for the two lifecycle events the Rust core sends and shows or
// clears the pill. That is all it does. The pill's mouse behaviour is entirely
// in Rust (src-tauri/src/dictate/pill_mouse.rs): this page never sees a mouse
// event, because the web view is kept out of the input path so it can never take
// focus from the app the person is typing into (record 0002 AC-27). Nothing here
// asks to be dragged, and no coordinate is ever computed or sent.

const { event } = window.__TAURI__;

const pill = document.querySelector(".pill");

event.listen("dictation:opened", () => {
  pill.dataset.state = "open";
});

event.listen("dictation:closed", () => {
  pill.dataset.state = "idle";
});
