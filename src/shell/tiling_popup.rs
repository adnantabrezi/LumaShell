//! Native Win32 COSMIC Tiling & Floating Popup (`LumaTilingPopup`).
//!
//! Styled identically to the System76 COSMIC Tiling Applet popover:
//! - Auto-Tiling vs Floating Mode switch (`Alt+Shift+T`)
//! - Split Direction selector (`Horizontal` | `Vertical`, `Alt+V`)
//! - Active Window quick controls (`Float/Tile`, `Flip Split`, `Swap`)
//! - Live Inner Gap & Outer Gap steppers (`0..24px`)
//! - COSMIC Active Focus Hint (`2px` DWM border color) switch
//! - Smooth Animation Speed selector (`Off` / `160ms` / `240ms` / `340ms`)
//! - `Tile All` & `Float All` workspace actions
//!
//! Owns its HWND via `OnceLock<Mutex<isize>>` (no new `static mut`).

use std::mem::size_of;
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
    EndPaint, FillRect, GetMonitorInfoW, HALFTONE, HDC, InvalidateRect, MONITOR_DEFAULTTONEAREST,
    MONITORINFO, MonitorFromPoint, PAINTSTRUCT, SRCCOPY, SelectObject, SetBrushOrgEx,
    SetStretchBltMode,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DestroyWindow, GetClientRect, GetCursorPos, GetWindowRect, HWND_TOPMOST,
    IsWindow, IsWindowVisible, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SetForegroundWindow, SetWindowPos,
    ShowWindow, WM_CLOSE, WM_ERASEBKGND, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONUP, WM_PAINT,
};

use super::config::{CONFIG, save_config};
use super::tiling::{self, TilingDirection};
use super::{
    AUDIO_MIXER, BLUETOOTH_POPUP, NETWORK_POPUP, PANEL, TRAY_POPUP, brush, fill_round_rect,
    icon_text, invalidate_shell_surfaces, measure_text_with_face, stroke_round_rect, text,
};

pub(crate) const TILING_POPUP_WIDTH: i32 = 336;
pub(crate) const TILING_POPUP_HEIGHT: i32 = 412;

static TILING_POPUP_HWND: OnceLock<Mutex<isize>> = OnceLock::new();

fn popup_slot() -> &'static Mutex<isize> {
    TILING_POPUP_HWND.get_or_init(|| Mutex::new(0))
}

pub(crate) fn set_tiling_popup_hwnd(hwnd: HWND) {
    if let Ok(mut guard) = popup_slot().lock() {
        *guard = hwnd.0 as isize;
    }
}

pub(crate) fn tiling_popup_hwnd() -> HWND {
    let raw = popup_slot().lock().map(|g| *g).unwrap_or(0);
    HWND(raw as *mut _)
}

pub(crate) fn is_tiling_popup_hwnd(hwnd: HWND) -> bool {
    !hwnd.is_invalid() && hwnd == tiling_popup_hwnd()
}

pub(crate) fn destroy_tiling_popup() {
    let hwnd = tiling_popup_hwnd();
    if !hwnd.is_invalid() {
        // SAFETY: Validated live HWND owned by Luma Shell's UI thread.
        unsafe {
            if IsWindow(hwnd).as_bool() {
                let _ = DestroyWindow(hwnd);
            }
        }
        set_tiling_popup_hwnd(HWND::default());
    }
}

pub(crate) fn hide_tiling_popup() {
    let popup = tiling_popup_hwnd();
    if !popup.is_invalid() {
        // SAFETY: Validated owned popup HWND on UI thread.
        unsafe {
            if IsWindowVisible(popup).as_bool() {
                let _ = ShowWindow(popup, SW_HIDE);
            }
        }
    }
}

pub(crate) unsafe fn show_tiling_popup(anchor_panel: HWND) {
    let popup = tiling_popup_hwnd();
    if popup.is_invalid() {
        return;
    }
    if IsWindowVisible(popup).as_bool() {
        let _ = ShowWindow(popup, SW_HIDE);
        return;
    }
    for other in [AUDIO_MIXER, NETWORK_POPUP, BLUETOOTH_POPUP, TRAY_POPUP] {
        if !other.is_invalid() && IsWindowVisible(other).as_bool() {
            let _ = ShowWindow(other, SW_HIDE);
        }
    }

    let mut cursor = POINT::default();
    let _ = GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info);

    let x = (cursor.x - TILING_POPUP_WIDTH / 2)
        .max(info.rcWork.left + 8)
        .min(info.rcWork.right - TILING_POPUP_WIDTH - 8);
    let target_panel = if !anchor_panel.is_invalid() {
        anchor_panel
    } else {
        PANEL
    };
    let mut panel_r = RECT::default();
    let panel_bottom = if !target_panel.is_invalid()
        && GetWindowRect(target_panel, &mut panel_r).is_ok()
    {
        panel_r.bottom
    } else {
        info.rcWork.top
    };
    let y = panel_bottom + 6;

    let _ = SetWindowPos(
        popup,
        HWND_TOPMOST,
        x,
        y,
        TILING_POPUP_WIDTH,
        TILING_POPUP_HEIGHT,
        SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(popup, None, false);
    let _ = ShowWindow(popup, SW_SHOW);
    let _ = SetForegroundWindow(popup);
    let _ = SetFocus(popup);
}

unsafe fn draw_switch(hdc: HDC, x: i32, y: i32, on: bool, accent: u32, dark: bool) {
    let track = RECT {
        left: x,
        top: y,
        right: x + 40,
        bottom: y + 20,
    };
    let track_brush = brush(if on {
        accent
    } else if dark {
        0x3f4756
    } else {
        0xcbd5e1
    });
    fill_round_rect(hdc, &track, track_brush, 20);
    let _ = DeleteObject(track_brush);

    let knob_x = if on { x + 22 } else { x + 2 };
    let knob = RECT {
        left: knob_x,
        top: y + 2,
        right: knob_x + 16,
        bottom: y + 18,
    };
    let knob_brush = brush(0xffffff);
    fill_round_rect(hdc, &knob, knob_brush, 16);
    let _ = DeleteObject(knob_brush);
}

unsafe fn draw_button(
    hdc: HDC,
    r: RECT,
    label: &str,
    active: bool,
    accent: u32,
    dark: bool,
    primary_fg: u32,
) {
    let bg = brush(if active {
        accent
    } else if dark {
        0x262d3b
    } else {
        0xe2e8f0
    });
    fill_round_rect(hdc, &r, bg, 10);
    let _ = DeleteObject(bg);
    if !active {
        stroke_round_rect(hdc, &r, if dark { 0x394356 } else { 0xcbd5e1 }, 1, 10);
    }
    let text_col = if active { 0x090d16 } else { primary_fg };
    let text_w = measure_text_with_face(hdc, label, 12, "Segoe UI Variable Text");
    let tx = r.left + ((r.right - r.left) - text_w).max(4) / 2;
    let ty = r.top + ((r.bottom - r.top) - 14).max(4) / 2;
    text(hdc, label, tx, ty, text_col, 12);
}

unsafe fn paint_tiling_popup(hwnd: HWND, screen_dc: HDC) {
    let mut r = RECT::default();
    if GetClientRect(hwnd, &mut r).is_err() {
        return;
    }
    let w = r.right - r.left;
    let h = r.bottom - r.top;
    if w <= 0 || h <= 0 {
        return;
    }

    let hdc = CreateCompatibleDC(screen_dc);
    let mem_bmp = CreateCompatibleBitmap(screen_dc, w, h);
    let old_bmp = SelectObject(hdc, mem_bmp);
    let _ = SetStretchBltMode(hdc, HALFTONE);
    let _ = SetBrushOrgEx(hdc, 0, 0, None);

    let c = CONFIG;
    let snap = tiling::snapshot();
    let dark = c.dark;
    let accent = c.accent_color & 0x00FF_FFFF;
    let bg_col = if dark { 0x131722 } else { 0xf8fafc };
    let card_col = if dark { 0x1c2231 } else { 0xffffff };
    let border_col = if dark { 0x2f384c } else { 0xdbe3ee };
    let primary = if dark { 0xf8fafc } else { 0x0f172a };
    let secondary = if dark { 0x94a3b8 } else { 0x64748b };

    let bg = brush(bg_col);
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);

    // Header
    icon_text(
        hdc,
        if snap.enabled { "\u{F0E2}" } else { "\u{E8A9}" },
        18,
        18,
        if snap.enabled { accent } else { 0xf59e0b },
        16,
    );
    text(hdc, "COSMIC Tiling", 42, 16, primary, 15);
    text(
        hdc,
        &format!(
            "GlazeWM Tree · {} tiled, {} floating",
            snap.tiled_count, snap.floating_count
        ),
        42,
        36,
        secondary,
        11,
    );

    // Card 1: Tile Windows primary switch (y = 60..112)
    let card1 = RECT {
        left: 16,
        top: 60,
        right: w - 16,
        bottom: 112,
    };
    let card_b = brush(card_col);
    fill_round_rect(hdc, &card1, card_b, 12);
    stroke_round_rect(hdc, &card1, border_col, 1, 12);
    text(hdc, "Tile Windows", 28, 70, primary, 13);
    text(
        hdc,
        if snap.enabled {
            "Auto-tiling enabled (Alt+Shift+T)"
        } else {
            "Floating mode active (Alt+Shift+T)"
        },
        28,
        89,
        secondary,
        11,
    );
    draw_switch(hdc, w - 68, 76, snap.enabled, accent, dark);

    // Row 2: Split Direction selector (y = 124..166)
    text(hdc, "New Split Direction", 18, 122, secondary, 11);
    text(hdc, "Alt+V", w - 52, 122, secondary, 11);
    let half_w = (w - 40) / 2;
    draw_button(
        hdc,
        RECT {
            left: 16,
            top: 140,
            right: 16 + half_w,
            bottom: 170,
        },
        "Horizontal",
        snap.direction == TilingDirection::Horizontal,
        accent,
        dark,
        primary,
    );
    draw_button(
        hdc,
        RECT {
            left: 24 + half_w,
            top: 140,
            right: w - 16,
            bottom: 170,
        },
        "Vertical",
        snap.direction == TilingDirection::Vertical,
        accent,
        dark,
        primary,
    );

    // Row 3: Active Window Quick Actions (y = 182..212)
    let third_w = (w - 44) / 3;
    draw_button(
        hdc,
        RECT {
            left: 16,
            top: 182,
            right: 16 + third_w,
            bottom: 212,
        },
        if snap.focused_is_floating {
            "Tile Active"
        } else {
            "Float Active"
        },
        false,
        accent,
        dark,
        primary,
    );
    draw_button(
        hdc,
        RECT {
            left: 22 + third_w,
            top: 182,
            right: 22 + third_w * 2,
            bottom: 212,
        },
        "Flip Split",
        false,
        accent,
        dark,
        primary,
    );
    draw_button(
        hdc,
        RECT {
            left: 28 + third_w * 2,
            top: 182,
            right: w - 16,
            bottom: 212,
        },
        "Swap Tile",
        false,
        accent,
        dark,
        primary,
    );

    // Card 2: Gaps, Active Hint & Animation Speed (y = 224..346)
    let card2 = RECT {
        left: 16,
        top: 224,
        right: w - 16,
        bottom: 348,
    };
    fill_round_rect(hdc, &card2, card_b, 12);
    stroke_round_rect(hdc, &card2, border_col, 1, 12);
    let _ = DeleteObject(card_b);

    // Inner Gap row (y = 234..258)
    text(hdc, "Inner Window Gap", 28, 238, primary, 12);
    draw_button(
        hdc,
        RECT {
            left: w - 118,
            top: 234,
            right: w - 92,
            bottom: 256,
        },
        "-",
        false,
        accent,
        dark,
        primary,
    );
    text(
        hdc,
        &format!("{}px", snap.inner_gap),
        w - 86,
        238,
        accent,
        12,
    );
    draw_button(
        hdc,
        RECT {
            left: w - 52,
            top: 234,
            right: w - 26,
            bottom: 256,
        },
        "+",
        false,
        accent,
        dark,
        primary,
    );

    // Outer Gap row (y = 264..286)
    text(hdc, "Outer Screen Gap", 28, 268, primary, 12);
    draw_button(
        hdc,
        RECT {
            left: w - 118,
            top: 264,
            right: w - 92,
            bottom: 286,
        },
        "-",
        false,
        accent,
        dark,
        primary,
    );
    text(
        hdc,
        &format!("{}px", snap.outer_gap),
        w - 86,
        268,
        accent,
        12,
    );
    draw_button(
        hdc,
        RECT {
            left: w - 52,
            top: 264,
            right: w - 26,
            bottom: 286,
        },
        "+",
        false,
        accent,
        dark,
        primary,
    );

    // Active Focus Hint row (y = 294..316)
    text(hdc, "COSMIC Active Hint (2px)", 28, 297, primary, 12);
    draw_switch(hdc, w - 68, 294, snap.active_hint, accent, dark);

    // Animation speed row (y = 322..342)
    text(hdc, "Smooth Animation", 28, 324, primary, 12);
    let anim_label = if snap.animation_ms == 0 {
        "Instant".to_string()
    } else {
        format!("{}ms ease-out", snap.animation_ms)
    };
    let anim_w = measure_text_with_face(hdc, &anim_label, 12, "Segoe UI Variable Text");
    text(hdc, &anim_label, w - 28 - anim_w, 324, accent, 12);

    // Footer Buttons: Tile All | Float All (y = 360..392)
    draw_button(
        hdc,
        RECT {
            left: 16,
            top: 360,
            right: 16 + half_w,
            bottom: 392,
        },
        "Tile All",
        snap.enabled,
        accent,
        dark,
        primary,
    );
    draw_button(
        hdc,
        RECT {
            left: 24 + half_w,
            top: 360,
            right: w - 16,
            bottom: 392,
        },
        "Float All",
        !snap.enabled,
        accent,
        dark,
        primary,
    );

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    let _ = SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) extern "system" fn tiling_popup_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_tiling_popup(hwnd, h);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let w_client = r.right - r.left;
                let half_w = (w_client - 40) / 2;
                let third_w = (w_client - 44) / 3;

                if (60..=112).contains(&y) {
                    let enabled = tiling::toggle_tiling_mode();
                    CONFIG.tiling_enabled = enabled;
                    save_config();
                    invalidate_shell_surfaces();
                } else if (140..=170).contains(&y) {
                    if x < 16 + half_w {
                        CONFIG.tiling_vertical = false;
                        tiling::set_split_direction(TilingDirection::Horizontal);
                    } else {
                        CONFIG.tiling_vertical = true;
                        tiling::set_split_direction(TilingDirection::Vertical);
                    }
                    save_config();
                } else if (182..=212).contains(&y) {
                    if x < 16 + third_w {
                        let _ = ShowWindow(hwnd, SW_HIDE);
                        tiling::toggle_active_window_float();
                    } else if x < 22 + third_w * 2 {
                        CONFIG.tiling_vertical = !CONFIG.tiling_vertical;
                        tiling::flip_split_direction();
                        save_config();
                    } else {
                        tiling::swap_focused_window(true);
                    }
                } else if (232..=258).contains(&y) {
                    if x >= w_client - 118 && x <= w_client - 92 {
                        let (inner, outer) = tiling::adjust_gaps(-2, 0);
                        CONFIG.tiling_inner_gap = inner;
                        CONFIG.tiling_outer_gap = outer;
                        save_config();
                    } else if x >= w_client - 52 && x <= w_client - 26 {
                        let (inner, outer) = tiling::adjust_gaps(2, 0);
                        CONFIG.tiling_inner_gap = inner;
                        CONFIG.tiling_outer_gap = outer;
                        save_config();
                    }
                } else if (262..=288).contains(&y) {
                    if x >= w_client - 118 && x <= w_client - 92 {
                        let (inner, outer) = tiling::adjust_gaps(0, -2);
                        CONFIG.tiling_inner_gap = inner;
                        CONFIG.tiling_outer_gap = outer;
                        save_config();
                    } else if x >= w_client - 52 && x <= w_client - 26 {
                        let (inner, outer) = tiling::adjust_gaps(0, 2);
                        CONFIG.tiling_inner_gap = inner;
                        CONFIG.tiling_outer_gap = outer;
                        save_config();
                    }
                } else if (290..=318).contains(&y) {
                    CONFIG.tiling_active_hint = tiling::toggle_active_hint();
                    save_config();
                } else if (319..=348).contains(&y) {
                    CONFIG.tiling_animation_ms = tiling::cycle_animation_speed();
                    save_config();
                } else if (360..=392).contains(&y) {
                    let tile_all = x < 16 + half_w;
                    CONFIG.tiling_enabled = tile_all;
                    tiling::set_all_windows_mode(tile_all);
                    save_config();
                    invalidate_shell_surfaces();
                }

                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_KILLFOCUS | WM_CLOSE => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
