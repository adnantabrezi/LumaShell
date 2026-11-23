use super::*;

pub(crate) unsafe fn query_system_brightness() -> u8 {
    let mut active_scheme: *mut GUID = null_mut();
    let res = PowerGetActiveScheme(None, &mut active_scheme);
    if res == WIN32_ERROR(0) && !active_scheme.is_null() {
        let video_subgroup = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
        let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
        let mut val = 0u32;
        let read = PowerReadACValueIndex(
            None,
            Some(active_scheme),
            Some(&video_subgroup),
            Some(&brightness_guid),
            &mut val,
        );
        let _ = LocalFree(HLOCAL(active_scheme as *mut _));
        if read == WIN32_ERROR(0) {
            CURRENT_BRIGHTNESS = val.clamp(0, 100) as u8;
            return CURRENT_BRIGHTNESS;
        }
    }
    CURRENT_BRIGHTNESS
}

pub(crate) unsafe fn adjust_system_brightness(delta: i8) -> u8 {
    let cur = CURRENT_BRIGHTNESS as i16 + delta as i16;
    let target = cur.clamp(0, 100) as u8;
    CURRENT_BRIGHTNESS = target;

    let mut active_scheme: *mut GUID = null_mut();
    let res = PowerGetActiveScheme(None, &mut active_scheme);
    if res == WIN32_ERROR(0) && !active_scheme.is_null() {
        let video_subgroup = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
        let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
        let _ = PowerWriteACValueIndex(
            None,
            active_scheme,
            Some(&video_subgroup),
            Some(&brightness_guid),
            target as u32,
        );
        let _ = PowerSetActiveScheme(None, Some(active_scheme));
        let _ = LocalFree(HLOCAL(active_scheme as *mut _));
    }
    target
}

pub(crate) unsafe fn show_volume_osd() {
    if let Some((level, muted)) = default_audio_state() {
        OSD_KIND = OsdKind::Volume;
        OSD_LEVEL = level;
        OSD_MUTED = muted;
        present_osd();
    }
}

pub(crate) unsafe fn show_brightness_osd(level: u8) {
    OSD_KIND = OsdKind::Brightness;
    OSD_LEVEL = level;
    OSD_MUTED = false;
    CURRENT_BRIGHTNESS = level;
    present_osd();
}

pub(crate) unsafe fn show_screenshot_osd() {
    OSD_KIND = OsdKind::Screenshot;
    present_osd();
}

pub(crate) unsafe fn present_osd() {
    if OSD_WINDOW.is_invalid() {
        return;
    }
    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    let mut mi = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(hmon, &mut mi);

    let mon_left = mi.rcWork.left;
    let mon_right = mi.rcWork.right;
    let mon_bottom = mi.rcWork.bottom;

    let x = mon_left + ((mon_right - mon_left) - OSD_WIDTH) / 2;
    let y = (mon_bottom - OSD_HEIGHT - 96).max(mi.rcWork.top + 20);

    let _ = SetWindowPos(
        OSD_WINDOW,
        HWND_TOPMOST,
        x,
        y,
        OSD_WIDTH,
        OSD_HEIGHT,
        SWP_SHOWWINDOW | SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(OSD_WINDOW, None, false);
    let _ = SetTimer(OSD_WINDOW, ID_OSD_TIMER, 1500, None);
}

pub(crate) unsafe fn paint_osd(hwnd: HWND, hdc: HDC) {
    let mut r = RECT::default();
    let _ = GetClientRect(hwnd, &mut r);

    let mem_dc = CreateCompatibleDC(hdc);
    let mem_bmp = CreateCompatibleBitmap(hdc, r.right, r.bottom);
    let old_bmp = SelectObject(mem_dc, mem_bmp);

    let is_dark = CONFIG.dark;
    let is_hc = high_contrast();

    let bg_col = if is_hc {
        0x000000
    } else if is_dark {
        0x201c18
    } else {
        0xfafbfe
    };

    let bg = brush(bg_col);
    FillRect(mem_dc, &r, bg);
    let _ = DeleteObject(bg);

    let border_col = if is_hc {
        0xffffff
    } else if is_dark {
        0x383430
    } else {
        0xd6dbe1
    };
    let border_pen = CreatePen(PS_SOLID, 1, colorref(border_col));
    let old_pen = SelectObject(mem_dc, border_pen);
    let null_brush =
        windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH);
    let old_brush = SelectObject(mem_dc, null_brush);
    let _ = RoundRect(mem_dc, r.left, r.top, r.right, r.bottom, 16, 16);
    SelectObject(mem_dc, old_brush);
    SelectObject(mem_dc, old_pen);
    let _ = DeleteObject(border_pen);

    let (icon, icon_col, label, fill_col) = match OSD_KIND {
        OsdKind::Volume => {
            if OSD_MUTED {
                ("\u{E74F}", 0xf87171, "Mute".to_string(), 0xf87171)
            } else {
                let ic = if OSD_LEVEL < 33 {
                    "\u{E993}"
                } else if OSD_LEVEL < 66 {
                    "\u{E994}"
                } else {
                    "\u{E767}"
                };
                (ic, 0x38bdf8, format!("{}%", OSD_LEVEL), 0x38bdf8)
            }
        }
        OsdKind::Brightness => ("\u{E706}", 0xfbbf24, format!("{}%", OSD_LEVEL), 0xfbbf24),
        OsdKind::Screenshot => (
            "\u{E7C5}",
            0x34d399,
            "Screenshot saved + copied".to_string(),
            0x34d399,
        ),
    };

    let is_screenshot = OSD_KIND == OsdKind::Screenshot;
    icon_text(
        mem_dc,
        icon,
        16,
        15,
        if is_hc { 0xffffff } else { icon_col },
        16,
    );

    if !is_screenshot {
        let track_x = 48;
        let track_y = 22;
        let track_w = 148;
        let track_h = 10;

        let track_col = if is_hc {
            0x333333
        } else if is_dark {
            0x322d28
        } else {
            0xe2e8f0
        };
        let track_brush = brush(track_col);
        let track_pen = CreatePen(PS_NULL, 0, colorref(0));
        let t_old_pen = SelectObject(mem_dc, track_pen);
        let t_old_brush = SelectObject(mem_dc, track_brush);
        let _ = RoundRect(
            mem_dc,
            track_x,
            track_y,
            track_x + track_w,
            track_y + track_h,
            8,
            8,
        );

        let level = if OSD_KIND == OsdKind::Volume && OSD_MUTED {
            0
        } else {
            OSD_LEVEL
        };
        if level > 0 {
            let fill_w = ((track_w * level as i32) / 100).max(8).min(track_w);
            let f_col = if is_hc { 0x00ffff } else { fill_col };
            let fill_brush = brush(f_col);
            SelectObject(mem_dc, fill_brush);
            let _ = RoundRect(
                mem_dc,
                track_x,
                track_y,
                track_x + fill_w,
                track_y + track_h,
                8,
                8,
            );
            SelectObject(mem_dc, t_old_brush);
            let _ = DeleteObject(fill_brush);
        } else {
            SelectObject(mem_dc, t_old_brush);
        }
        SelectObject(mem_dc, t_old_pen);
        let _ = DeleteObject(track_brush);
        let _ = DeleteObject(track_pen);
    }

    let text_col = if is_hc {
        0xffffff
    } else if is_dark {
        0xf1f5f9
    } else {
        0x0f172a
    };
    text(
        mem_dc,
        &label,
        if is_screenshot { 48 } else { 206 },
        17,
        text_col,
        13,
    );

    let _ = BitBlt(hdc, 0, 0, r.right, r.bottom, mem_dc, 0, 0, SRCCOPY);

    SelectObject(mem_dc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(mem_dc);
}

pub(crate) extern "system" fn osd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_osd(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_TIMER if w.0 == 2 => {
                // Windows does not send a useful notification to this custom
                // desktop window when the user changes wallpaper. A cheap
                // refresh keeps the shell in sync without reloading anything
                // until the next paint.
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_TIMER => {
                if w.0 == ID_OSD_TIMER {
                    let _ = KillTimer(hwnd, ID_OSD_TIMER);
                    ShowWindow(hwnd, SW_HIDE);
                }
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
