//! Where the dashboard window goes, and how big it is.
//!
//! Rust owns this and the interface never sees a coordinate, in either
//! direction (record 0004: there is deliberately no command for the window's
//! size or position). The shape of the memory is the pill's, for the pill's own
//! reason: a size in logical pixels, and a place as two fractions locating the
//! window's centre within the working area of a screen. A fraction survives a
//! resolution change where a pixel does not.
//!
//! The maths in here is pure and unit tested. The two things only the operating
//! system can answer are asked of it: the screens it currently reports, through
//! Tauri, and which window is in front, through the one Windows call at the
//! foot of this file. That call is the platform boundary AGENTS.md requires:
//! Windows today, and a macOS arm to fill in later.

use tauri::AppHandle;

/// The comp's Surface 2 geometry, in logical pixels. What the dashboard opens
/// at the first time (record 0004 AC-3).
pub const FIRST_OPEN_W: f64 = 1200.0;
pub const FIRST_OPEN_H: f64 = 800.0;

/// The smallest the dashboard may be, in logical pixels. Fixed, not a setting,
/// and chosen so a 1366x768 laptop can show the window (record 0004 AC-3,
/// AC-5, and its Still open note that this is a judgement).
pub const FLOOR_W: f64 = 960.0;
pub const FLOOR_H: f64 = 640.0;

/// A rectangle in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// One screen as the operating system currently reports it: the name it gives
/// it, its working area (the desktop minus the taskbar, wherever the person
/// keeps it) and its scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    pub name: String,
    pub work: Rect,
    pub scale: f64,
}

/// The screen to open on.
///
/// The remembered name first, matched against the screens the system reports
/// right now. A name that matches nothing is not an error and never was: the
/// screen it named has been unplugged, or renamed, and the primary screen is
/// the answer (record 0004 AC-5). With no name at all, which is a first ever
/// open, it is the screen holding the window that is in front, because that is
/// the screen the person is looking at.
pub fn chosen_screen(app: &AppHandle, remembered_name: Option<&str>) -> Option<Screen> {
    if let Some(name) = remembered_name.filter(|n| !n.is_empty()) {
        if let Some(screen) = screen_named(app, name) {
            return Some(screen);
        }
    }
    focused_screen(app).or_else(|| primary_screen(app))
}

/// The screen the system reports under `name`, if it still reports one.
fn screen_named(app: &AppHandle, name: &str) -> Option<Screen> {
    app.available_monitors()
        .ok()?
        .into_iter()
        .find(|monitor| monitor.name().map(String::as_str) == Some(name))
        .map(screen_of)
}

/// The screen holding the window that is in front, or nothing if the system
/// cannot say which that is.
fn focused_screen(app: &AppHandle) -> Option<Screen> {
    let (x, y) = focused_window_center()?;
    app.monitor_from_point(x, y).ok()?.map(screen_of)
}

fn primary_screen(app: &AppHandle) -> Option<Screen> {
    app.primary_monitor().ok()?.map(screen_of)
}

/// The screen holding a point, for reading back where the person left the
/// window. Falls back to the primary screen, so a window dragged to a screen
/// the system stops reporting mid-drag still stores a sane fraction.
pub fn screen_holding(app: &AppHandle, x: f64, y: f64) -> Option<Screen> {
    match app.monitor_from_point(x, y) {
        Ok(Some(monitor)) => Some(screen_of(monitor)),
        _ => primary_screen(app),
    }
}

fn screen_of(monitor: tauri::Monitor) -> Screen {
    let area = monitor.work_area();
    Screen {
        name: monitor.name().cloned().unwrap_or_default(),
        work: Rect {
            x: area.position.x,
            y: area.position.y,
            w: area.size.width as i32,
            h: area.size.height as i32,
        },
        scale: monitor.scale_factor(),
    }
}

/// The inner size to give the window, in physical pixels.
///
/// Three things at once, in the order record 0004 sets them out: what was asked
/// for, then never below the floor, then never larger than the working area can
/// hold once the window's own frame is accounted for. A working area smaller
/// than the floor is the one case where the floor gives way, and then the
/// window is the working area, which is the record's "never below the floor
/// unless the working area itself is smaller".
pub fn fit_inner(
    work: (i32, i32),
    frame: (i32, i32),
    floor: (i32, i32),
    want: (i32, i32),
) -> (i32, i32) {
    let cap = ((work.0 - frame.0).max(1), (work.1 - frame.1).max(1));
    (
        want.0.max(floor.0).min(cap.0),
        want.1.max(floor.1).min(cap.1),
    )
}

/// The physical top-left to put the window's outer rectangle at, so its centre
/// lands on the remembered fractions and the whole rectangle stays inside the
/// working area.
///
/// The clamp is the whole of AC-5: it never opens part way off an edge, and,
/// because the working area already excludes the taskbar, it never opens under
/// the taskbar either. This is the same clamp that makes AC-26 hold for the
/// pill.
pub fn outer_topleft(work: Rect, outer: (i32, i32), frac_x: f64, frac_y: f64) -> (i32, i32) {
    let center_x = work.x as f64 + frac_x.clamp(0.0, 1.0) * work.w as f64;
    let center_y = work.y as f64 + frac_y.clamp(0.0, 1.0) * work.h as f64;
    let x = (center_x - outer.0 as f64 / 2.0).round() as i32;
    let y = (center_y - outer.1 as f64 / 2.0).round() as i32;

    let max_x = (work.x + work.w - outer.0).max(work.x);
    let max_y = (work.y + work.h - outer.1).max(work.y);
    (x.clamp(work.x, max_x), y.clamp(work.y, max_y))
}

/// The inverse: where the window's centre sits within the working area, as two
/// fractions. Clamped to 0..=1 so a window dragged half onto another screen
/// still stores a fraction that means something.
pub fn fractions(work: Rect, outer: Rect) -> (f64, f64) {
    let center_x = outer.x as f64 + outer.w as f64 / 2.0;
    let center_y = outer.y as f64 + outer.h as f64 / 2.0;
    let fx = (center_x - work.x as f64) / work.w.max(1) as f64;
    let fy = (center_y - work.y as f64) / work.h.max(1) as f64;
    (fx.clamp(0.0, 1.0), fy.clamp(0.0, 1.0))
}

/// The centre of the window that is in front, in physical desktop coordinates.
///
/// The platform boundary. Windows can say which window has the foreground and
/// where it is; Tauri cannot, because a window belonging to another application
/// is not one of its own. Everything above this line is portable, and a macOS
/// arm goes beside it rather than through it (AGENTS.md architecture rules).
#[cfg(target_os = "windows")]
fn focused_window_center() -> Option<(f64, f64)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut rect) }.is_err() {
        return None;
    }
    Some((
        (rect.left as f64 + rect.right as f64) / 2.0,
        (rect.top as f64 + rect.bottom as f64) / 2.0,
    ))
}

#[cfg(not(target_os = "windows"))]
fn focused_window_center() -> Option<(f64, f64)> {
    // Not built yet. Callers already treat "the system cannot say" as the
    // primary screen, so this is a fallback rather than a gap.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1920x1080 screen with a 40px taskbar along the bottom.
    const FHD: Rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };

    /// A typical Windows frame: no border either side of the client area worth
    /// speaking of, and a title bar on top.
    const FRAME: (i32, i32) = (16, 39);
    const FLOOR: (i32, i32) = (FLOOR_W as i32, FLOOR_H as i32);

    #[test]
    fn the_first_open_size_fits_a_full_hd_screen() {
        // covers: AC-3.
        let want = (FIRST_OPEN_W as i32, FIRST_OPEN_H as i32);
        let got = fit_inner((FHD.w, FHD.h), FRAME, FLOOR, want);
        assert_eq!(got, want);
    }

    #[test]
    fn a_remembered_size_larger_than_the_screen_is_shrunk_to_fit() {
        // covers: AC-5. The size left on a big screen no longer fits, so the
        // window opens at what does.
        let small = (1366, 728); // 1366x768 with a taskbar
        let got = fit_inner(small, FRAME, FLOOR, (2400, 1500));
        assert_eq!(got, (1366 - FRAME.0, 728 - FRAME.1));
        assert!(got.0 <= small.0 && got.1 <= small.1);
    }

    #[test]
    fn a_remembered_size_below_the_floor_comes_up_to_the_floor() {
        // covers: AC-3.
        let got = fit_inner((FHD.w, FHD.h), FRAME, FLOOR, (400, 300));
        assert_eq!(got, FLOOR);
    }

    #[test]
    fn the_floor_gives_way_to_a_working_area_smaller_than_it() {
        // covers: AC-5. The window is the working area rather than being
        // unusable, which is the record's one exception to the floor.
        let tiny = (800, 600);
        let got = fit_inner(tiny, FRAME, FLOOR, (1200, 800));
        assert_eq!(got, (800 - FRAME.0, 600 - FRAME.1));
    }

    #[test]
    fn the_centre_fractions_place_the_window_in_the_middle() {
        // covers: AC-4. 0.5, 0.5 is the default, and it centres. Even
        // dimensions, so "centred" is one exact answer rather than a rounding
        // question that would tell us nothing about the clamp.
        let outer = (1216, 840);
        let (x, y) = outer_topleft(FHD, outer, 0.5, 0.5);
        assert_eq!(x, (1920 - outer.0) / 2);
        assert_eq!(y, (1040 - outer.1) / 2);
    }

    #[test]
    fn a_place_past_an_edge_is_pulled_wholly_on_screen() {
        // covers: AC-5. Never part way off an edge, and never under the
        // taskbar, because the working area already excludes it.
        let outer = (1216, 839);
        let (x, y) = outer_topleft(FHD, outer, 1.0, 1.0);
        assert_eq!(x, 1920 - outer.0);
        assert_eq!(y, 1040 - outer.1);
        let (x0, y0) = outer_topleft(FHD, outer, 0.0, 0.0);
        assert_eq!((x0, y0), (0, 0));
    }

    #[test]
    fn a_window_larger_than_the_working_area_still_starts_inside_it() {
        // covers: AC-5. The size is fitted before this runs, so this is the
        // belt and braces case: even an oversized rectangle starts at the
        // working area's own corner rather than off it.
        let (x, y) = outer_topleft(FHD, (3000, 2000), 0.5, 0.5);
        assert_eq!((x, y), (0, 0));
    }

    #[test]
    fn place_and_read_back_round_trip() {
        // covers: AC-4. What is stored on a move is what places the window on
        // the next open. A spot the clamp does not have to touch, because the
        // round trip is only ever exact where the whole window fits at the spot
        // asked for; the clamp's own behaviour is the two tests above.
        let outer = (1216, 700);
        let (x, y) = outer_topleft(FHD, outer, 0.4, 0.6);
        let (fx, fy) = fractions(
            FHD,
            Rect {
                x,
                y,
                w: outer.0,
                h: outer.1,
            },
        );
        assert!((fx - 0.4).abs() < 0.01, "fx was {fx}");
        assert!((fy - 0.6).abs() < 0.01, "fy was {fy}");
    }

    #[test]
    fn a_second_screens_offset_does_not_leak_into_the_fraction() {
        // covers: AC-4. A place is a spot within a screen, not a point on one
        // screen, so the same relative spot on a screen that starts at x = 1920
        // gives the same fraction. This is what makes the memory survive the
        // screen it was made on being unplugged.
        let secondary = Rect {
            x: 1920,
            y: 0,
            w: 1920,
            h: 1040,
        };
        let outer = (1216, 839);
        let (px, py) = outer_topleft(FHD, outer, 0.4, 0.6);
        let (sx, sy) = outer_topleft(secondary, outer, 0.4, 0.6);
        assert_eq!(sx - 1920, px);
        assert_eq!(sy, py);
    }

    #[test]
    fn a_place_off_the_edge_of_a_screen_still_stores_a_sane_fraction() {
        // covers: AC-5. A window sitting off a screen entirely, which is what a
        // monitor being unplugged mid-session leaves behind, still reads back as
        // a fraction the next open can place from rather than as nonsense.
        let outer = Rect {
            x: 9000,
            y: -400,
            w: 1216,
            h: 840,
        };
        let (fx, fy) = fractions(FHD, outer);
        assert!((0.0..=1.0).contains(&fx));
        assert!((0.0..=1.0).contains(&fy));
        // Far off the right edge, so the horizontal fraction saturates. The
        // vertical one does not: the window's centre is still just inside the
        // top of the screen even though its top edge is not.
        assert_eq!(fx, 1.0);
        assert!(fy > 0.0 && fy < 0.05, "fy was {fy}");
    }
}
