//! The dashboard window: the 1200x800 two-tone surface with the dark rail on
//! the left and the white reading surface on the right.
//!
//! It exists whenever a person is signed in with a Deepgram key saved, and at
//! no other time. That is the invariant everything in this feature leans on
//! (record 0004, The decision), and `super::settle` is the one place that
//! decides it.
//!
//! This file creates, places, shows and closes it, and reads its size and place
//! back from the system after the person has finished moving or resizing it.
//! Nothing about the geometry crosses into the interface as a number, in either
//! direction: the person resizes through the window's own title bar, which is
//! Windows doing it, and Rust reads the result. That is the pill's precedent
//! after the AC-27 fix, and it is written down so nobody adds a convenience
//! getter later.

use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Size, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use super::geometry::{self, Rect};
use super::store::{RememberedWindow, DEFAULT_CENTER_X, DEFAULT_CENTER_Y};

/// The window label. `dashboard.json` is scoped to exactly this, and to
/// nothing else.
pub const LABEL: &str = "dashboard";

/// Whether the dashboard window exists right now.
pub fn is_open(app: &AppHandle) -> bool {
    app.get_webview_window(LABEL).is_some()
}

/// Create the dashboard at the remembered size and place, or at the first-open
/// size when nothing is remembered, and show it. Does nothing if it is already
/// there.
///
/// It is built hidden and shown only once it has been sized and placed, so a
/// person never sees it appear in one spot and jump to another.
pub fn open(app: &AppHandle, remembered: Option<RememberedWindow>) -> tauri::Result<()> {
    if is_open(app) {
        return Ok(());
    }

    let screen = geometry::chosen_screen(app, remembered.as_ref().map(|r| r.screen_name.as_str()));
    let (want_w, want_h) = remembered
        .as_ref()
        .map(|r| (r.width, r.height))
        .unwrap_or((geometry::FIRST_OPEN_W, geometry::FIRST_OPEN_H));
    let (frac_x, frac_y) = remembered
        .as_ref()
        .map(|r| (r.center_x, r.center_y))
        .unwrap_or((DEFAULT_CENTER_X, DEFAULT_CENTER_Y));

    let window =
        WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("shell/dashboard.html".into()))
            .title("EchoScribe")
            .inner_size(want_w, want_h)
            .min_inner_size(geometry::FLOOR_W, geometry::FLOOR_H)
            .resizable(true)
            .visible(false)
            .build()?;

    if let Some(screen) = screen {
        fit_and_place(&window, &screen, (want_w, want_h), (frac_x, frac_y));
    }

    // The only two things the window itself tells us. Windows moves and resizes
    // it through its own title bar and edges, and this is where the result is
    // read back from; there is no command through which the interface could
    // report either, because it never has a coordinate to report.
    let handle = app.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
            super::moved_or_resized(&handle);
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            // Held open for the moment it takes to write the row, so the size
            // and place a person just chose are not lost by closing.
            api.prevent_close();
            super::dashboard_closed(&handle);
        }
        _ => {}
    });

    window.show()?;
    Ok(())
}

/// Size the window to what the chosen screen can hold and put it where the
/// fractions say, both clamped. Best effort throughout: a system that will not
/// answer one of these questions leaves the window at the size and place the
/// builder gave it, which is on screen either way.
fn fit_and_place(
    window: &WebviewWindow,
    screen: &geometry::Screen,
    want: (f64, f64),
    frac: (f64, f64),
) {
    let scale = window.scale_factor().unwrap_or(screen.scale);
    // The window's own frame: the title bar and edges Windows draws around the
    // client area. It counts against the working area, so it has to be known
    // before the size can be fitted, and it can only be measured from a real
    // window.
    let Some(frame) = frame_of(window) else {
        return;
    };
    let floor = (
        (geometry::FLOOR_W * scale).round() as i32,
        (geometry::FLOOR_H * scale).round() as i32,
    );
    let want_px = (
        (want.0 * scale).round() as i32,
        (want.1 * scale).round() as i32,
    );
    let fitted = geometry::fit_inner((screen.work.w, screen.work.h), frame, floor, want_px);

    if fitted != want_px {
        // The floor has to come down with the size, or Windows refuses to make
        // the window as small as a working area smaller than the floor needs
        // (record 0004: the window is the working area in that one case).
        let _ = window.set_min_size(Some(Size::Physical(PhysicalSize::new(
            fitted.0.min(floor.0) as u32,
            fitted.1.min(floor.1) as u32,
        ))));
        let _ = window.set_size(Size::Physical(PhysicalSize::new(
            fitted.0 as u32,
            fitted.1 as u32,
        )));
    }

    let outer = match window.outer_size() {
        Ok(size) => (size.width as i32, size.height as i32),
        Err(_) => (fitted.0 + frame.0, fitted.1 + frame.1),
    };
    let (x, y) = geometry::outer_topleft(screen.work, outer, frac.0, frac.1);
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

/// The difference between the window's outer and inner rectangles, in physical
/// pixels: the frame Windows draws around the page.
fn frame_of(window: &WebviewWindow) -> Option<(i32, i32)> {
    let inner = window.inner_size().ok()?;
    let outer = window.outer_size().ok()?;
    Some((
        (outer.width as i32 - inner.width as i32).max(0),
        (outer.height as i32 - inner.height as i32).max(0),
    ))
}

/// Where and how big the window is now, ready to be remembered: the inner size
/// in logical pixels, the centre as two fractions of the working area of the
/// screen it is on, and that screen's name.
///
/// `None` while the window is minimised or gone. A minimised window has no size
/// worth keeping, and storing the one Windows reports for it would lose the
/// size the person actually chose.
pub fn placement_now(app: &AppHandle) -> Option<RememberedWindow> {
    let window = app.get_webview_window(LABEL)?;
    if window.is_minimized().unwrap_or(false) {
        return None;
    }
    let inner = window.inner_size().ok()?;
    let outer_size = window.outer_size().ok()?;
    let outer_pos = window.outer_position().ok()?;
    if inner.width == 0 || inner.height == 0 {
        return None;
    }
    let scale = window.scale_factor().unwrap_or(1.0);

    let outer = Rect {
        x: outer_pos.x,
        y: outer_pos.y,
        w: outer_size.width as i32,
        h: outer_size.height as i32,
    };
    let screen = geometry::screen_holding(
        app,
        outer.x as f64 + outer.w as f64 / 2.0,
        outer.y as f64 + outer.h as f64 / 2.0,
    )?;
    let (center_x, center_y) = geometry::fractions(screen.work, outer);
    Some(RememberedWindow {
        width: inner.width as f64 / scale,
        height: inner.height as f64 / scale,
        center_x,
        center_y,
        screen_name: screen.name,
    })
}

/// Close the dashboard for good. Signing out destroys it rather than hiding it:
/// it shows one account's settings and remembers one account's window place,
/// and a window kept alive across a sign out is a way for the second account on
/// a machine to see the first one's (record 0004, The decision).
pub fn close(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.destroy();
    }
}
