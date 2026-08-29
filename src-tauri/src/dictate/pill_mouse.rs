//! The pill's mouse behaviour, owned by Rust.
//!
//! Record 0002 refuses a pill that ever takes focus, and the live run of
//! 2026-08-29 showed it taking focus on a plain single click. The record's
//! amendment names three layers that can each take focus on their own, and all
//! three are closed here:
//!
//!   1. **The frame refuses activation at the moment of the click.** It answers
//!      `WM_MOUSEACTIVATE` with `MA_NOACTIVATE`, so pressing the mouse on the
//!      pill leaves the foreground window untouched. This is the click-time half
//!      of the promise; the standing half is the non-activating window style,
//!      which is Tauri's to keep (`focusable(false)` in `pill_window.rs`) and is
//!      checked on every open by `refuses_activation` below. Setting that style
//!      by hand was the original fault: Tauri rewrites a window's whole style
//!      word from its own flags whenever any flag changes, and showing the pill
//!      changes one, so the hand-set style was wiped on every open.
//!   2. **The web view never takes keyboard focus.** Its window is disabled for
//!      input, so no mouse message reaches Chromium and Chromium never calls
//!      `SetFocus` on itself. That call was the fault: focusing a child of a
//!      window that is not active makes Windows activate that window's
//!      top-level parent, which walks straight past `WS_EX_NOACTIVATE`. Windows
//!      routes a click over a disabled child to its parent instead, so the
//!      frame below receives it and the code here decides what happens.
//!   3. **The drag never uses Windows' own move loop.** It is run here with
//!      `SetWindowPos` and `SWP_NOACTIVATE`. Windows' move loop activates
//!      whatever it moves, and it is a modal loop that blocks the thread it
//!      runs on, which is why the hotkey went dead after the pill was touched.
//!
//! Because the web view no longer sees the mouse, the pill's hit area is decided
//! here too. The grip answers the mouse. Every other pixel of the window
//! swallows the click and does nothing, so touching the pill can neither move
//! the pill nor reach the app underneath it.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, Ordering};
use std::sync::OnceLock;

use tauri::AppHandle;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GetClassNameW, GetCursorPos, GetWindow, GetWindowLongPtrW, GetWindowRect,
    LoadCursorW, SetCursor, SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, GWL_EXSTYLE, GW_CHILD,
    GW_HWNDNEXT, HTCLIENT, IDC_SIZEALL, MA_NOACTIVATE, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    WM_CAPTURECHANGED, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_MOUSEMOVE, WM_SETCURSOR, WNDPROC, WS_EX_NOACTIVATE,
};

use super::pill_window::{GRIP_W, PAD, PILL_H};

/// The window procedure Tauri installed. Everything not handled here is passed
/// on to it. Zero until `take_over` has run.
static ORIGINAL_PROC: AtomicIsize = AtomicIsize::new(0);

/// Whether the grip is being dragged right now.
static DRAGGING: AtomicBool = AtomicBool::new(false);

/// Where in the window the grip was grabbed, so the pill keeps its offset from
/// the pointer for the whole drag instead of jumping under it.
static GRAB_DX: AtomicI32 = AtomicI32::new(0);
static GRAB_DY: AtomicI32 = AtomicI32::new(0);

/// Needed to remember the spot when a drag ends. Set once by `take_over`.
static APP: OnceLock<AppHandle> = OnceLock::new();

/// Put this file's window procedure in front of Tauri's for the pill's frame.
/// Called once, right after the window is built.
pub fn take_over(app: &AppHandle, hwnd: HWND) {
    let _ = APP.set(app.clone());
    let ours = pill_proc as *const () as isize;
    let previous = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, ours) };
    if previous == 0 {
        // Without this the pill can be activated by a click, which record 0002
        // refuses outright. Say so loudly rather than pretend it is fine.
        eprintln!("dictate: could not take over the pill's window procedure");
        return;
    }
    ORIGINAL_PROC.store(previous, Ordering::SeqCst);
}

/// Disable the web view's window so no mouse message ever reaches it. Windows
/// then delivers clicks over that area to the pill's frame, and Chromium never
/// gets the chance to focus itself.
///
/// Called every time the pill opens rather than once at startup: the web view's
/// window does not exist until the web view has been created, and disabling an
/// already-disabled window costs nothing. Disabling a window disables its own
/// children too, so the direct children cover the whole web view.
pub fn keep_mouse_out_of_the_web_view(hwnd: HWND) {
    let mut child = unsafe { GetWindow(hwnd, GW_CHILD) }.unwrap_or_default();
    while !child.is_invalid() {
        // EnableWindow returns whether the window was *already* disabled, so
        // this reports the first open only rather than every one.
        let already = unsafe { EnableWindow(child, false) };
        if !already.as_bool() {
            eprintln!(
                "dictate: pill web view kept out of the mouse path: {}",
                class_name_of(child)
            );
        }
        child = unsafe { GetWindow(child, GW_HWNDNEXT) }.unwrap_or_default();
    }
}

/// Whether Windows currently has the pill marked as a window it must never
/// activate.
///
/// This is a tripwire, not decoration. The fault of 2026-08-29 was that this
/// style was set by hand at startup and silently wiped on every open, because
/// Tauri rewrites a window's whole style word from its own flags whenever any
/// flag changes. The style is Tauri's to keep now (`focusable(false)`), and this
/// checks that it really is kept, every time the pill opens, so the same silent
/// loss can never go unnoticed again.
pub fn refuses_activation(hwnd: HWND) -> bool {
    let ex = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    ex & WS_EX_NOACTIVATE.0 as isize != 0
}

fn class_name_of(hwnd: HWND) -> String {
    let mut buffer = [0u16; 128];
    let written = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if written <= 0 {
        return "unknown".into();
    }
    String::from_utf16_lossy(&buffer[..written as usize])
}

/// The pill's window procedure. Everything it does not handle goes on to
/// Tauri's, untouched.
unsafe extern "system" fn pill_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Point 1: refuse activation at the moment of the click. The mouse
        // message itself still arrives, which is what MA_NOACTIVATE means and
        // what the drag below needs.
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),

        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
            let (x, y) = client_point(lparam);
            if in_grip(hwnd, x, y) {
                begin_drag(hwnd, x, y);
            }
            // Swallowed either way. Nothing outside the grip answers the mouse,
            // and nothing is passed down to the app underneath.
            return LRESULT(0);
        }

        WM_MOUSEMOVE if DRAGGING.load(Ordering::SeqCst) => {
            follow_the_pointer(hwnd);
            return LRESULT(0);
        }

        WM_LBUTTONUP if DRAGGING.load(Ordering::SeqCst) => {
            end_drag();
            return LRESULT(0);
        }

        // Capture can be taken away, for instance by Alt-Tab mid-drag. Treat
        // that as the drag ending so the pill cannot get stuck to the pointer.
        WM_CAPTURECHANGED if DRAGGING.load(Ordering::SeqCst) => {
            end_drag();
            return LRESULT(0);
        }

        // The web view no longer sees the mouse, so the grip's CSS grab cursor
        // never fires. Windows has no grab cursor; the move cursor is its
        // nearest equivalent and carries the same meaning (design-system.md:
        // the grip has a grab cursor).
        WM_SETCURSOR if (lparam.0 as u32 & 0xffff) == HTCLIENT => {
            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_ok()
                && ScreenToClient(hwnd, &mut point).as_bool()
                && in_grip(hwnd, point.x, point.y)
            {
                if let Ok(cursor) = LoadCursorW(None, IDC_SIZEALL) {
                    SetCursor(Some(cursor));
                    return LRESULT(1);
                }
            }
        }

        _ => {}
    }

    let original: WNDPROC = std::mem::transmute(ORIGINAL_PROC.load(Ordering::SeqCst));
    CallWindowProcW(original, hwnd, msg, wparam, lparam)
}

/// The click position from a mouse message's `lparam`, in physical pixels from
/// the window's top-left. The pill has no decorations, so its client area and
/// its window are the same rectangle.
fn client_point(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam.0 & 0xffff) as u16 as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xffff) as u16 as i16 as i32;
    (x, y)
}

/// Whether a point in the window is inside the grip, at this window's screen
/// scale.
fn in_grip(hwnd: HWND, x: i32, y: i32) -> bool {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let scale = if dpi == 0 { 1.0 } else { dpi as f64 / 96.0 };
    grip_contains(scale, x, y)
}

/// The grip is the leftmost 44 logical pixels of the visible pill, full height,
/// and the visible pill starts `PAD` in from the window's top-left. Everything
/// else in the window is inert.
fn grip_contains(scale: f64, x: i32, y: i32) -> bool {
    let at = |logical: f64| (logical * scale).round() as i32;
    x >= at(PAD) && x < at(PAD + GRIP_W) && y >= at(PAD) && y < at(PAD + PILL_H)
}

fn begin_drag(hwnd: HWND, x: i32, y: i32) {
    GRAB_DX.store(x, Ordering::SeqCst);
    GRAB_DY.store(y, Ordering::SeqCst);
    DRAGGING.store(true, Ordering::SeqCst);
    unsafe { SetCapture(hwnd) };
}

/// Move the window to keep the grabbed point under the pointer. `SWP_NOACTIVATE`
/// and no move loop: Windows is told where the window goes, never asked to move
/// it (point 3).
fn follow_the_pointer(hwnd: HWND) {
    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_err() {
        return;
    }
    let x = point.x - GRAB_DX.load(Ordering::SeqCst);
    let y = point.y - GRAB_DY.load(Ordering::SeqCst);
    let _ = unsafe {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
}

fn end_drag() {
    DRAGGING.store(false, Ordering::SeqCst);
    let _ = unsafe { ReleaseCapture() };

    // Off this thread: this is the window's own thread, and saving touches
    // SQLite. The window procedure must not block on it.
    if let Some(app) = APP.get().cloned() {
        std::thread::spawn(move || super::remember_pill_spot(&app));
    }
}

/// Where the pill's window is now, in physical screen pixels. Read straight from
/// Windows so it can be called from any thread without waiting on the window's
/// own thread.
pub fn window_topleft(hwnd: HWND) -> Option<(i32, i32)> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    Some((rect.left, rect.top))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Record 0002: the grip is the only area that answers the mouse. These pin
    // the boundary the window procedure hit-tests against, at both the scales
    // this machine's two screens run at.

    #[test]
    fn the_middle_of_the_grip_is_the_grip() {
        assert!(grip_contains(1.0, 38, 38));
        assert!(grip_contains(1.5, 57, 57));
    }

    #[test]
    fn the_words_and_the_ring_are_not_the_grip() {
        // The signal area starts where the 44px grip ends.
        assert!(!grip_contains(1.0, 61, 38));
        assert!(!grip_contains(1.5, 91, 57));
    }

    #[test]
    fn the_transparent_margin_is_not_the_grip() {
        // Inside the window, outside the visible pill: shadow room, not a grip.
        assert!(!grip_contains(1.0, 4, 4));
        assert!(!grip_contains(1.0, 38, 4));
        assert!(!grip_contains(1.0, 38, 70));
    }

    #[test]
    fn the_grip_reaches_the_top_and_bottom_of_the_pill() {
        assert!(grip_contains(1.0, 20, 16));
        assert!(grip_contains(1.0, 20, 59));
        assert!(!grip_contains(1.0, 20, 15));
        assert!(!grip_contains(1.0, 20, 60));
    }
}
