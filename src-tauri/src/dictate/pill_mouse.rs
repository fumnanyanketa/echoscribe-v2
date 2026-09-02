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
        // Full height of the 52px shell (the twelfth amendment's one
        // geometry), starting PAD (16) in from the window's top: y in [16, 68).
        assert!(grip_contains(1.0, 20, 16));
        assert!(grip_contains(1.0, 20, 67));
        assert!(!grip_contains(1.0, 20, 15));
        assert!(!grip_contains(1.0, 20, 68));
    }

    // ---- Regression tests for the focus fault of 2026-08-29 ----
    //
    // These run against real Windows windows, created here and thrown away
    // again, because what is under test are answers Windows itself gives.
    //
    // What they cannot reach, and why. The window procedure `pill_proc` is not
    // called by any test in this file. Calling it links the Tauri runtime into
    // the unit-test binary, by way of `end_drag` saving the pill's spot, and
    // that binary then will not start at all: it picks up an import of
    // `TaskDialogIndirect` from Tauri's menu crate, which needs the Common
    // Controls version 6 manifest that only the app binary gets. So the pieces
    // the window procedure calls are tested directly instead, and the
    // procedure's own dispatch is left to the live pass. This was measured, not
    // assumed; see the hand-off notes for 2026-08-29.

    use std::sync::{Mutex, Once};
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WINDOW_EX_STYLE, WNDCLASSW,
        WS_CHILD, WS_POPUP,
    };

    /// A drag is held in process-wide state, so the tests that start one run
    /// one at a time.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    static TEST_CLASS: Once = Once::new();
    const TEST_CLASS_NAME: PCWSTR = w!("EchoScribeTestPillFrame");

    fn serially() -> std::sync::MutexGuard<'static, ()> {
        let guard = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        // A test that panicked mid-drag would otherwise leave this set and make
        // the next one fail for the wrong reason.
        DRAGGING.store(false, Ordering::SeqCst);
        guard
    }

    /// The stock window procedure, wrapped so it has the calling convention a
    /// window class asks for.
    unsafe extern "system" fn stock_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    fn test_instance() -> HINSTANCE {
        HINSTANCE(
            unsafe { GetModuleHandleW(None) }
                .expect("a module handle")
                .0,
        )
    }

    /// A real, hidden Windows window the size of the pill's frame.
    fn a_real_window(parent: Option<HWND>) -> HWND {
        TEST_CLASS.call_once(|| {
            let class = WNDCLASSW {
                lpfnWndProc: Some(stock_proc),
                hInstance: test_instance(),
                lpszClassName: TEST_CLASS_NAME,
                ..Default::default()
            };
            assert!(
                unsafe { RegisterClassW(&class) } != 0,
                "could not register the test window class"
            );
        });
        let style = if parent.is_some() { WS_CHILD } else { WS_POPUP };
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                TEST_CLASS_NAME,
                w!("echoscribe test"),
                style,
                100,
                100,
                264,
                76,
                parent,
                None,
                Some(test_instance()),
                None,
            )
        }
        .expect("could not create a test window")
    }

    fn destroy(hwnd: HWND) {
        let _ = unsafe { DestroyWindow(hwnd) };
    }

    fn cursor() -> POINT {
        let mut point = POINT::default();
        let _ = unsafe { GetCursorPos(&mut point) };
        point
    }

    /// Whether `value` sits between two readings of the pointer taken either
    /// side of the move, give or take a pixel of rounding.
    fn between(value: i32, first: i32, second: i32) -> bool {
        value >= first.min(second) - 1 && value <= first.max(second) + 1
    }

    // (50, 50) is inside the grip at every screen scale from 1.0 to 3.0, and
    // (200, 50) is outside it at every one of them: the grip runs from 16 to 60
    // logical pixels in from the window's top-left, so 50 clears 16 x 3 and
    // stays short of 60 x 1, while 200 clears 60 x 3. That keeps the two tests
    // below true whichever monitor they happen to run on.
    const IN_THE_GRIP: (i32, i32) = (50, 50);
    const ON_THE_WORDS: (i32, i32) = (200, 50);

    #[test]
    fn the_web_view_is_taken_out_of_the_mouse_path() {
        // covers: AC-27. Layer 2 of the three in record 0002. A click must never
        // reach Chromium, because Chromium focuses itself, and focusing a child
        // activates its top-level parent, walking straight past the
        // non-activating style. Disabling the child is what stops that.
        let frame = a_real_window(None);
        let web_view = a_real_window(Some(frame));
        assert!(
            unsafe { IsWindowEnabled(web_view) }.as_bool(),
            "a fresh child window starts enabled, so the next assertion means something"
        );

        keep_mouse_out_of_the_web_view(frame);

        assert!(
            !unsafe { IsWindowEnabled(web_view) }.as_bool(),
            "the pill's child window can still receive the mouse, so Chromium can \
             focus itself and take the person's typing cursor with it"
        );
        destroy(frame);
    }

    #[test]
    fn the_tripwire_notices_the_non_activating_style_going_missing() {
        // covers: AC-27. The standing window style is Tauri's to keep and no
        // test can see it; the source guards in pill_window.rs are the nearest
        // thing. What can be proved is that the run-time tripwire watching that
        // style really does read the right bit, in both directions. If this were
        // wrong the style could be lost live and nothing would say so, which is
        // exactly what happened on 2026-08-29.
        let window = a_real_window(None);
        assert!(
            !refuses_activation(window),
            "a window created without WS_EX_NOACTIVATE was reported as refusing \
             activation, so the tripwire would stay silent while the pill stole focus"
        );

        unsafe {
            let existing = GetWindowLongPtrW(window, GWL_EXSTYLE);
            SetWindowLongPtrW(window, GWL_EXSTYLE, existing | WS_EX_NOACTIVATE.0 as isize);
        }

        assert!(
            refuses_activation(window),
            "a window carrying WS_EX_NOACTIVATE was reported as activatable, so the \
             tripwire would cry wolf on every open"
        );
        destroy(window);
    }

    #[test]
    fn the_grip_is_hit_tested_at_the_windows_own_screen_scale() {
        // covers: AC-27. The four tests above pin the grip's geometry at scales
        // chosen by hand. This one goes through `in_grip`, which asks Windows
        // for the scale of the screen the window is actually on, so a wrong
        // reading of the scale is caught as well as a wrong rectangle.
        let window = a_real_window(None);
        assert!(
            in_grip(window, IN_THE_GRIP.0, IN_THE_GRIP.1),
            "the middle of the grip was not treated as the grip, so the pill \
             cannot be dragged"
        );
        assert!(
            !in_grip(window, ON_THE_WORDS.0, ON_THE_WORDS.1),
            "the words were treated as the grip, so the surface that answers the \
             mouse is wider than the one part of the pill meant to"
        );
        destroy(window);
    }

    #[test]
    fn a_drag_is_moved_by_this_code_and_never_by_windows_own_move_loop() {
        // covers: AC-5. The hotkey went dead after the pill was touched because
        // the drag used Windows' own move loop. That loop is modal: it takes
        // over the thread it runs on and does not hand it back until the mouse
        // comes up, and the pill's thread is not free to do anything else
        // meanwhile. The replacement is these two calls, which move the window
        // and return. This walks them and checks the window really did move to
        // keep the grabbed point under the pointer.
        let _serial = serially();
        let window = a_real_window(None);

        begin_drag(window, IN_THE_GRIP.0, IN_THE_GRIP.1);
        assert!(
            DRAGGING.load(Ordering::SeqCst),
            "grabbing the grip did not start a drag this code owns"
        );

        let before = cursor();
        follow_the_pointer(window);
        let after = cursor();

        let (left, top) = window_topleft(window).expect("the test window has a position");
        // The pointer is read either side of the move as well as inside it, so
        // a real hand nudging the mouse mid-test cannot make this flap.
        assert!(
            between(left + IN_THE_GRIP.0, before.x, after.x),
            "the window was not moved to keep the grabbed point under the pointer: \
             its left edge was {left}, and the pointer was at x {} then {}",
            before.x,
            after.x
        );
        assert!(
            between(top + IN_THE_GRIP.1, before.y, after.y),
            "the window was not moved to keep the grabbed point under the pointer: \
             its top edge was {top}, and the pointer was at y {} then {}",
            before.y,
            after.y
        );

        // `end_drag` cannot be called from here: it reaches back into the app to
        // save the spot, which would link the Tauri runtime into this binary and
        // stop it starting. Undo by hand instead.
        DRAGGING.store(false, Ordering::SeqCst);
        let _ = unsafe { ReleaseCapture() };
        destroy(window);
    }
}
