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

/// The visible pill, in logical pixels. One geometry for its whole life, from
/// open to close, in every state: the drawn 472x52 listening form, settled
/// 2026-08-31 by record 0002's twelfth amendment. The old 232x44 MIC OPEN form
/// is retired as a window size; what it showed survives as this shell's content
/// before words arrive (design/registry.md, "Pill shell").
const PILL_W: f64 = 472.0;
pub(super) const PILL_H: f64 = 52.0;
/// The counter band beneath the shell: a `--space-3` (12px) gap plus one chip
/// row. Part of the fixed footprint from the first moment, so the elapsed and
/// count chip's arrival after 20 seconds resizes nothing and moves nothing
/// (design/registry.md, "Elapsed and word count"; record 0002 AC-36). The chip
/// was built 2026-09-04 into this reserved band, and `pill.css` positions it
/// against these constants: 16px from the right is the shell's right edge, and
/// 80px from the top is `PAD` plus `PILL_H` plus the band's 12px gap.
const COUNTER_BAND_H: f64 = 34.0;
/// The grip: the leftmost part of the pill, full height, and the only part that
/// answers the mouse (design/design-system.md, record 0002).
pub(super) const GRIP_W: f64 = 44.0;
/// Transparent margin around the pill, each side, so the shadow can render.
pub(super) const PAD: f64 = 16.0;
/// Smallest gap kept between the pill and any edge of the working area.
const EDGE_GAP: f64 = 8.0;

const WINDOW_W: f64 = PILL_W + PAD * 2.0;
const WINDOW_H: f64 = PILL_H + COUNTER_BAND_H + PAD * 2.0;

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
        let band = scaled(COUNTER_BAND_H, scale);
        let gap = scaled(EDGE_GAP, scale);
        let pad = scaled(PAD, scale);
        let (px, py) = pill_topleft(
            area,
            pill_w,
            pill_h,
            band,
            gap,
            setting.pill_x,
            setting.pill_y,
        );
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
    // Broadcast, not sent to the pill alone. The EchoScribe window listens for
    // this too: a microphone error it is still showing is a claim that the
    // microphone cannot open, and this is the proof that it can (record 0002
    // AC-32, and the clearing table in The decision). It goes out on the
    // capability that window already has, so nothing widens for it. The window
    // is not shown, focused or moved by this: the person is dictating into
    // another app, and this record brings a window forward for one reason only.
    let _ = app.emit("dictation:opened", json!({}));
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
/// whole footprint, the shell plus the counter band below it, stays inside the
/// working area with at least `gap` to every edge (record 0002, the twelfth
/// amendment: the footprint is clamped at open, and the window is never resized
/// while it is open). The fractions still name the shell's own centre, so no
/// stored value changes meaning.
fn pill_topleft(
    area: WorkArea,
    pill_w: i32,
    pill_h: i32,
    band: i32,
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
    let max_y = (area.top + area.height - pill_h - band - gap).max(min_y);
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

    /// The one drawn geometry, in the physical pixels of a 100% scale screen.
    /// Tied to the real constants so a geometry change can never leave these
    /// tests proving the old shape.
    const W: i32 = PILL_W as i32;
    const H: i32 = PILL_H as i32;
    const BAND: i32 = COUNTER_BAND_H as i32;

    #[test]
    fn the_default_spot_is_bottom_centre_above_the_taskbar() {
        // covers: AC-23. frac 0.5, 1.0 with the one 472x52 form, its counter
        // band, and an 8px gap. The band is below the shell, so the shell sits
        // a band's height further up: the whole footprint stays on screen.
        let (x, y) = pill_topleft(FHD, W, H, BAND, 8, 0.5, 1.0);
        assert_eq!(x, 1920 / 2 - W / 2); // horizontally centred
        assert_eq!(y, 1040 - H - BAND - 8); // footprint on the bottom edge, gap kept
    }

    #[test]
    fn a_spot_past_the_edge_is_pulled_fully_on_screen() {
        // covers: AC-23, AC-26.
        let (x, y) = pill_topleft(FHD, W, H, BAND, 8, 1.0, 0.0);
        assert_eq!(x, 1920 - W - 8);
        assert_eq!(y, 8);
        let (x0, _) = pill_topleft(FHD, W, H, BAND, 8, 0.0, 0.5);
        assert_eq!(x0, 8);
    }

    #[test]
    fn a_spot_saved_against_the_old_small_pill_is_reclamped_not_migrated() {
        // covers: AC-24, AC-26, and the twelfth amendment's "no stored value
        // changes". A bottom-centre fraction stored while the pill was 232x44
        // simply lands the wide footprint fully on screen, by the same clamp
        // that already covers a smaller screen. No migration exists.
        let (x, y) = pill_topleft(FHD, W, H, BAND, 8, 0.5, 1.0);
        assert!(x >= 8 && x + W <= 1920 - 8);
        assert!(y >= 8 && y + H + BAND <= 1040 - 8);
    }

    #[test]
    fn topleft_and_fraction_round_trip_near_the_middle() {
        // covers: AC-24.
        let (x, y) = pill_topleft(FHD, W, H, BAND, 8, 0.4, 0.6);
        let (fx, fy) = pill_fraction(FHD, W, H, x, y);
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
        let (fx, fy) = pill_fraction(small, W, H, 1920 + 5000, 0);
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
        let (px, py) = pill_topleft(primary, W, H, BAND, 8, 0.3, 0.7);
        let (sx, sy) = pill_topleft(secondary, W, H, BAND, 8, 0.3, 0.7);
        assert_eq!(sx - 1920, px);
        assert_eq!(sy, py);
    }

    // ---- Source guards for the focus fault of 2026-08-29 ----
    //
    // Read this before trusting them. These are NOT behaviour tests. They read
    // this feature's own source text and check that the shape of the fix is
    // still there. They cannot run the app, cannot build a real pill, and
    // cannot see the window style Windows ends up with.
    //
    // They exist because the fault they guard has no honest automated test.
    // The style is put on a real window by Tauri, at run time, and it was lost
    // silently: Tauri rewrites a window's whole style word from its own flags
    // whenever any flag changes, and showing the pill changes one, so the style
    // that was set by hand at startup was wiped on every open. Proving the
    // style is present needs a real pill from a real Tauri app, which a unit
    // test cannot make. So there are three layers instead, and this is the
    // weakest of them:
    //
    //   1. Live, and the only real proof: click the pill with the caret in a
    //      text editor and check the next thing typed lands in the editor.
    //      Manual, and owed by record 0002's step 1a.
    //   2. Run time: `open` calls `pill_mouse::refuses_activation` after every
    //      show and shouts if the style has gone. That check is tested for real
    //      in pill_mouse.rs.
    //   3. These: if someone deletes `focusable(false)` or goes back to setting
    //      the style by hand, the suite goes red at once rather than at the
    //      next live run.

    /// This feature's source, minus its own tests, so a guard cannot be
    /// satisfied by the test that checks it.
    fn source_without_tests(whole: &str) -> &str {
        whole
            .split("#[cfg(test)]")
            .next()
            .expect("a source file always has a first part")
    }

    /// Whitespace removed and anything non-ASCII dropped, so a guard survives
    /// `cargo fmt` moving a call across lines.
    fn flattened(source: &str) -> String {
        source
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_whitespace())
            .collect()
    }

    fn this_file() -> &'static str {
        source_without_tests(include_str!("pill_window.rs"))
    }

    fn the_mouse_file() -> &'static str {
        source_without_tests(include_str!("pill_mouse.rs"))
    }

    #[test]
    fn the_pill_asks_tauri_for_the_non_activating_window_style() {
        // covers: AC-27, as a source guard only. `focusable(false)` is what
        // makes the style Tauri's own, and so what makes it survive every open.
        // Removing it brings back the fault of 2026-08-29 in full: a single
        // click on the pill takes the typing cursor and the keystrokes that
        // follow are lost.
        assert!(
            flattened(this_file()).contains(".focusable(false)"),
            "the pill window is no longer built with focusable(false). Tauri is \
             then free to give it an activating style, and clicking the pill will \
             take the person's typing cursor (record 0002 AC-27)"
        );
    }

    #[test]
    fn the_non_activating_style_is_never_set_by_hand() {
        // covers: AC-27, as a source guard only. Setting the extended window
        // style directly is the exact thing that did not work: Tauri rewrites
        // the whole style word from its own flags on every change, so a
        // hand-set style is wiped on every single open, silently.
        for (name, source) in [
            ("pill_window.rs", this_file()),
            ("pill_mouse.rs", the_mouse_file()),
        ] {
            let flat = flattened(source);
            let by_hand = flat.match_indices("GWL_EXSTYLE").any(|(index, _)| {
                let look_back = index.saturating_sub(40);
                flat[look_back..index].contains("SetWindowLong")
            });
            assert!(
                !by_hand,
                "{name} writes the extended window style by hand. Tauri wipes that \
                 on the next open and the pill starts taking focus again; the style \
                 has to come from focusable(false) instead (record 0002 AC-27)"
            );
        }
    }

    #[test]
    fn the_style_tripwire_runs_after_the_pill_is_shown_not_before() {
        // covers: AC-27, as a source guard only. Showing the window is the
        // moment the style used to be lost, so a check made before the show
        // would pass every time while the pill stole focus every time.
        let flat = flattened(this_file());
        let shown = flat
            .find("window.show()")
            .expect("open() still shows the pill window");
        let checked = flat
            .find("refuses_activation")
            .expect("open() still checks that the pill refuses activation");
        assert!(
            checked > shown,
            "the non-activating style is checked before the pill is shown. That is \
             the one order in which the check cannot see the fault it exists for"
        );
    }

    #[test]
    fn the_transcript_line_can_actually_slide_left() {
        // covers: AC-33's left-fade truncation, as a source guard only. The
        // inner transcript row must refuse to shrink, and the overflow check
        // must measure that inner row, or the line never slides at all: a
        // default flex item shrinks to its min-width floor, measures exactly
        // as wide as the outer box, and the text spills out of it to the
        // RIGHT, clipped, with the newest words the hidden ones. The pill then
        // sits on the first words of every long dictation for its whole run,
        // which a person watching mid-sentence reads as frozen. Found live on
        // 2026-10-01 by the user, nine frames deep into a 229-character
        // dictation whose pill never moved. The outer box cannot be measured
        // instead: its overflow hangs out to the LEFT, and scrollWidth never
        // counts left-side overflow in left-to-right writing, so the outer box
        // reports no overflow while visibly overflowing. Proving the slide
        // needs a real dictation past 472px of words, so /check verify owns
        // that; this stops both halves of the cure being simplified away.
        let css = flattened(include_str!("../../../src/dictate/pill.css"));
        let inner = css
            .find(".pill__line-inner{")
            .expect("pill.css no longer styles the transcript's inner row");
        let block = &css[inner..css[inner..].find('}').map_or(css.len(), |e| inner + e)];
        assert!(
            block.contains("flex:none"),
            "the transcript's inner row no longer refuses to shrink. It will shrink \
             to its min-width floor, the outer box will never see overflow, and the \
             line will sit on the first words of every long dictation instead of \
             sliding left"
        );
        let js = flattened(include_str!("../../../src/dictate/pill.js"));
        assert!(
            js.contains("inner.scrollWidth>line.clientWidth"),
            "the overflow check no longer measures the inner row against the outer \
             box. Measuring the outer box against itself reports no overflow while \
             the line visibly overflows leftward, and the fade never appears"
        );
    }
}
