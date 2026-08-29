//! The floating pill window: a small, always-on-top window that Windows will
//! not activate, so clicking or dragging it never moves the person's typing
//! cursor (record 0002 AC-27, and the hard refusal in the Risk section).
//!
//! This file owns where the pill goes. Its mouse behaviour, which is what keeps
//! the promise above, lives next door in `pill_mouse.rs`.
//!
//! On open it is placed once, on the screen holding the focused window, at the
//! spot remembered for the account as fractions of that screen's working area,
//! clamped so it is always fully on screen with a small gap (AC-23, AC-24,
//! AC-26). It is not moved again for the rest of that dictation, even if focus
//! moves to another screen (AC-25). When the person drags the grip,
//! `pill_mouse.rs` moves the window and this file reads where it landed and
//! converts that back to fractions.
//!
//! Showing, hiding and placing go through Tauri so they run on the window's own
//! thread. Windows is asked directly only for things it is the only source of:
//! the focused window, a screen's working area (the desktop minus the taskbar),
//! and that screen's scale. The non-activating style is left to Tauri, which
//! owns it and rewrites it on every show; setting it by hand does not survive.

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use super::pill_mouse;
use super::store::DictationSetting;

/// The window label. The `pill.json` capability is scoped to exactly this.
pub const LABEL: &str = "pill";

/// The visible pill, in logical pixels. Fixed geometry across every state
/// (design/registry.md).
const PILL_W: f64 = 232.0;
pub(super) const PILL_H: f64 = 44.0;
/// The grip: the leftmost part of the pill, full height, and the only part that
/// answers the mouse (design/design-system.md, record 0002).
pub(super) const GRIP_W: f64 = 44.0;
/// Transparent margin around the pill, each side, so the shadow can render.
pub(super) const PAD: f64 = 16.0;
/// Smallest gap kept between the pill and any edge of the working area.
const EDGE_GAP: f64 = 8.0;

const WINDOW_W: f64 = PILL_W + PAD * 2.0;
const WINDOW_H: f64 = PILL_H + PAD * 2.0;

/// A screen's working area in physical pixels: the desktop minus the taskbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkArea {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

/// Create the pill window, hidden and non-activating. Called once at startup so
/// the first open has nothing to load.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("dictate/pill.html".into()))
        .title("EchoScribe dictation")
        .inner_size(WINDOW_W, WINDOW_H)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        // The load-bearing one. Tauri turns this into the Windows style that
        // means "never activate this window", and, because Tauri owns that
        // style, keeps it there. Setting the same style by hand does not last:
        // Tauri rewrites a window's whole style word from its own flags every
        // time any flag changes, and showing the pill changes one, so a
        // hand-set style is wiped on every single open. That is what the live
        // run of 2026-08-29 was hitting.
        .focusable(false)
        // A different thing, and still wanted: do not activate it the first
        // time it appears.
        .focused(false)
        .visible(false)
        .build()?;

    match window.hwnd() {
        Ok(hwnd) => pill_mouse::take_over(app, hwnd),
        Err(e) => {
            // Without this the pill cannot refuse activation at the moment of a
            // click, which record 0002 refuses outright. Say so loudly rather
            // than ship a pill that steals focus.
            eprintln!("dictate: could not reach the pill's window: {e}");
        }
    }
    Ok(())
}

/// Place the pill on the focused screen at the account's remembered spot and
/// show it without taking focus. Emits `dictation:opened` for the interface.
pub fn open(app: &AppHandle, setting: &DictationSetting) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };

    if let Some((area, scale)) = focused_screen() {
        let (pill_w, pill_h) = (scaled(PILL_W, scale), scaled(PILL_H, scale));
        let gap = scaled(EDGE_GAP, scale);
        let pad = scaled(PAD, scale);
        let (px, py) = pill_topleft(area, pill_w, pill_h, gap, setting.pill_x, setting.pill_y);
        let _ = window.set_position(PhysicalPosition::new(px - pad, py - pad));
    }

    // Before it is on screen, and every time: the web view's window is only
    // there once the web view exists, and it must never see a mouse message.
    if let Ok(hwnd) = window.hwnd() {
        pill_mouse::keep_mouse_out_of_the_web_view(hwnd);
    }

    let _ = window.show();

    // Showing the window is the moment the style used to be lost, so check it
    // after showing, not before. Record 0002 treats a pill that can take focus
    // as a hard refusal, so if this ever goes missing it must be loud rather
    // than silently costing somebody their keystrokes.
    if let Ok(hwnd) = window.hwnd() {
        if !pill_mouse::refuses_activation(hwnd) {
            eprintln!(
                "dictate: THE PILL CAN BE ACTIVATED. Clicking it will take the \
                 typing cursor from whatever app the person is in (record 0002 \
                 AC-27). The non-activating window style is missing."
            );
        }
    }
    let _ = app.emit_to(LABEL, "dictation:opened", json!({}));
}

/// Hide the pill and tell the interface why it closed.
pub fn close(app: &AppHandle, reason: &str) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.hide();
        let _ = app.emit_to(LABEL, "dictation:closed", json!({ "reason": reason }));
    }
}

/// Where the pill sits now, as fractions of its current screen's working area,
/// after the person has dragged it. `None` if the window or a screen cannot be
/// read.
pub fn spot_after_drag(app: &AppHandle) -> Option<(f64, f64)> {
    let window = app.get_webview_window(LABEL)?;
    let hwnd = window.hwnd().ok()?;
    // Read from Windows, not from Tauri: this runs on a thread of its own once a
    // drag ends, and asking Tauri would mean waiting on the window's thread.
    let (outer_x, outer_y) = pill_mouse::window_topleft(hwnd)?;

    let hmon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let (area, scale) = work_area_of(hmon)?;
    let pad = scaled(PAD, scale);
    // The visible pill is the window rect inset by the transparent margin.
    let px = outer_x + pad;
    let py = outer_y + pad;
    let (pill_w, pill_h) = (scaled(PILL_W, scale), scaled(PILL_H, scale));
    Some(pill_fraction(area, pill_w, pill_h, px, py))
}

/// The working area and scale of the screen holding the focused window, or the
/// primary screen if there is no foreground window.
fn focused_screen() -> Option<(WorkArea, f64)> {
    let fg = unsafe { GetForegroundWindow() };
    let hmon = unsafe { MonitorFromWindow(fg, MONITOR_DEFAULTTOPRIMARY) };
    work_area_of(hmon)
}

fn work_area_of(hmon: HMONITOR) -> Option<(WorkArea, f64)> {
    if hmon.is_invalid() {
        return None;
    }
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(hmon, &mut info) }.as_bool() {
        return None;
    }
    let RECT {
        left,
        top,
        right,
        bottom,
    } = info.rcWork;
    let area = WorkArea {
        left,
        top,
        width: right - left,
        height: bottom - top,
    };

    let mut dpi_x = 96u32;
    let mut dpi_y = 96u32;
    let scale = unsafe {
        match GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) {
            Ok(()) => dpi_x as f64 / 96.0,
            Err(_) => 1.0,
        }
    };
    Some((area, scale))
}

fn scaled(logical: f64, scale: f64) -> i32 {
    (logical * scale).round() as i32
}

/// Physical top-left of the visible pill for a fractional spot, clamped so the
/// whole pill stays inside the working area with at least `gap` to every edge.
fn pill_topleft(
    area: WorkArea,
    pill_w: i32,
    pill_h: i32,
    gap: i32,
    frac_x: f64,
    frac_y: f64,
) -> (i32, i32) {
    let center_x = area.left as f64 + frac_x.clamp(0.0, 1.0) * area.width as f64;
    let center_y = area.top as f64 + frac_y.clamp(0.0, 1.0) * area.height as f64;
    let x = (center_x - pill_w as f64 / 2.0).round() as i32;
    let y = (center_y - pill_h as f64 / 2.0).round() as i32;

    let min_x = area.left + gap;
    let max_x = (area.left + area.width - pill_w - gap).max(min_x);
    let min_y = area.top + gap;
    let max_y = (area.top + area.height - pill_h - gap).max(min_y);
    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

/// The inverse: fractional spot of the pill's centre from its physical top-left.
/// The result is clamped to 0..=1 so a drag onto a sliver of another screen
/// still stores a sane fraction.
fn pill_fraction(area: WorkArea, pill_w: i32, pill_h: i32, x: i32, y: i32) -> (f64, f64) {
    let center_x = x as f64 + pill_w as f64 / 2.0;
    let center_y = y as f64 + pill_h as f64 / 2.0;
    let fx = (center_x - area.left as f64) / area.width as f64;
    let fy = (center_y - area.top as f64) / area.height as f64;
    (fx.clamp(0.0, 1.0), fy.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FHD: WorkArea = WorkArea {
        left: 0,
        top: 0,
        width: 1920,
        height: 1040, // 1080 minus a 40px taskbar
    };

    #[test]
    fn the_default_spot_is_bottom_centre_above_the_taskbar() {
        // frac 0.5, 1.0 with a 232x44 pill and an 8px gap.
        let (x, y) = pill_topleft(FHD, 232, 44, 8, 0.5, 1.0);
        assert_eq!(x, 1920 / 2 - 232 / 2); // horizontally centred
        assert_eq!(y, 1040 - 44 - 8); // sitting on the bottom edge, gap kept
    }

    #[test]
    fn a_spot_past_the_edge_is_pulled_fully_on_screen() {
        let (x, y) = pill_topleft(FHD, 232, 44, 8, 1.0, 0.0);
        assert_eq!(x, 1920 - 232 - 8);
        assert_eq!(y, 8);
        let (x0, _) = pill_topleft(FHD, 232, 44, 8, 0.0, 0.5);
        assert_eq!(x0, 8);
    }

    #[test]
    fn topleft_and_fraction_round_trip_near_the_middle() {
        let (x, y) = pill_topleft(FHD, 232, 44, 8, 0.4, 0.6);
        let (fx, fy) = pill_fraction(FHD, 232, 44, x, y);
        assert!((fx - 0.4).abs() < 0.01, "fx was {fx}");
        assert!((fy - 0.6).abs() < 0.01, "fy was {fy}");
    }

    #[test]
    fn fraction_stays_inside_zero_to_one_on_a_smaller_screen() {
        let small = WorkArea {
            left: 1920,
            top: 0,
            width: 1280,
            height: 720,
        };
        let (fx, fy) = pill_fraction(small, 232, 44, 1920 + 5000, 0);
        assert!((0.0..=1.0).contains(&fx));
        assert!((0.0..=1.0).contains(&fy));
        assert_eq!(fx, 1.0);
    }

    #[test]
    fn a_second_screen_offset_does_not_leak_into_the_fraction() {
        // The same relative spot on a screen that starts at x = 1920 gives the
        // same fraction as on the primary screen (record 0002: a spot within a
        // screen, not a point on one screen).
        let primary = FHD;
        let secondary = WorkArea {
            left: 1920,
            top: 0,
            width: 1920,
            height: 1040,
        };
        let (px, py) = pill_topleft(primary, 232, 44, 8, 0.3, 0.7);
        let (sx, sy) = pill_topleft(secondary, 232, 44, 8, 0.3, 0.7);
        assert_eq!(sx - 1920, px);
        assert_eq!(sy, py);
    }
}
