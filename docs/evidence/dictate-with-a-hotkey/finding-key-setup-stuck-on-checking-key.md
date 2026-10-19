# Finding: the key setup screen sticks on "Checking key" after a key is accepted

Found live by the user, 2026-10-13 06:09 local, on the installed release build
of `ae230df`, signed in as a second account with no key. The same symptom was
reported by the user on 2026-10-05 ("stuck on Checking key, the X does
nothing") and was never reproduced then.

Diagnosis: **isolated bug**, from one wrong belief written into one comment.

## What was observed on the stuck app, before anything was touched

- The key was **accepted and saved**. `deepgram_credential` holds a row for the
  account, last four `b624`, saved `2026-10-13T03:09:41Z`, three seconds after
  the user pressed Verify. The 2026-10-05 key (`caf3`) was likewise saved at
  `12:19:38Z` that afternoon.
- Windows, read by handle at 07:11:
  - `EchoScribe` (the dashboard) **exists and is not visible**, 2400x1600.
  - `echoscribe` (the small window) is visible, minimised, still drawing
    "Checking key".
  - The pill exists, hidden, as always.
- The process answers Windows (`Responding: True`), has used 0.9 s of CPU in an
  hour, and has no network connection open. It is waiting, not working.

So the check finished, the key was stored, and the dashboard was begun but
never shown, and the screen never heard that its check had finished.

## The cause

When a key is saved, the app announces it, and the part that arranges the
windows hears the announcement and builds the dashboard. The announcement is
heard on whatever thread made it. Saving the key is a command, and in Tauri 2
a command that is not marked `async` runs **on the app's main thread**, inside
the very message the window sent. Building a window from there is a known
deadlock on Windows. Tauri's own documentation for building a window says so:

> On Windows, this function deadlocks when used in a synchronous command or
> event handlers ... You should use `async` commands and separate threads when
> creating webviews.
> (tauri 2.11.5, `src/webview/webview_window.rs`)

The window build starts, waits on WebView2 for a reply that cannot come while
the main thread is inside the command, and never returns. So:

- the dashboard exists but `show()` is never reached: built hidden, left hidden;
- the command never returns, so the screen's "Checking key" never ends;
- the small window's X does nothing, because hiding it needs the same stuck
  main thread. That is the 2026-10-05 report, word for word.

The comment above the command said the opposite:
"Declared `fn`, not `async fn`, so Tauri runs it on a worker thread". That was
true of Tauri 1 and is not true of Tauri 2. Every other path that builds the
dashboard announces from a thread of its own (the browser sign-in hand-off,
the background session renewal), which is why sign-in has always worked.

## Why it looked intermittent

It is not. It happens **every time** a key is saved from the small window by
an account that has no dashboard yet, which is the first-run path and nothing
else. That path has run for real exactly twice since the dashboard was built
(record 0004, 2026-09-03), on 2026-10-05 and today, and it hung both times.
The key setup proofs of 2026-08-30 predate the dashboard, so nothing was built
when a key was saved and nothing hung. Replacing a key from Settings does not
hang either, because the dashboard is already open and nothing is built.

## Other places with the same shape

Checked every `#[tauri::command]` (23) and every announcement the window
arranger listens to. Only this one builds a window from inside a synchronous
command. Signing out and removing a key announce from a command too, but on
those paths the dashboard is closed, not built.

Related and not fixed here: this command and `start_sign_in` both wait on the
network on the main thread (up to 10 s and 5 s), which freezes the window's
animation for that long. A freeze, not a hang. Recorded for a separate change.

## Residue

None in the data. Both keys were saved correctly. The only residue is the
stuck process itself, which nothing inside it can recover.
