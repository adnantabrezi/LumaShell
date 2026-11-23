use super::*;

pub(crate) unsafe fn read_registry_wallpaper() -> Result<String, ()> {
    let subkey = wide("Control Panel\\Desktop");
    let mut key = HKEY::default();
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        PCWSTR(subkey.as_ptr()),
        0,
        KEY_READ,
        &mut key,
    )
    .is_err()
    {
        return Err(());
    }
    let value_name = wide("Wallpaper");
    let mut buf = [0u16; 512];
    let mut buf_len = (buf.len() * 2) as u32;
    let mut reg_type = REG_SZ;
    let res = RegQueryValueExW(
        key,
        PCWSTR(value_name.as_ptr()),
        None,
        Some(&mut reg_type),
        Some(buf.as_mut_ptr() as *mut u8),
        Some(&mut buf_len),
    );
    let _ = RegCloseKey(key);
    if res.is_ok() {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            return Ok(String::from_utf16_lossy(&buf[..len]));
        }
    }
    Err(())
}

static mut CACHED_WALLPAPER: Option<(HBITMAP, i32, i32, String)> = None;

pub(crate) unsafe fn get_cached_wallpaper(path: &str) -> Option<(HBITMAP, i32, i32)> {
    if let Some((bmp, w, h, cached_path)) = &CACHED_WALLPAPER {
        if cached_path == path && !bmp.is_invalid() {
            return Some((*bmp, *w, *h));
        }
    }
    if let Some((new_bmp, w, h)) = load_wallpaper_bitmap(path) {
        if let Some((old_bmp, _, _, _)) = CACHED_WALLPAPER.take() {
            let _ = DeleteObject(old_bmp);
        }
        CACHED_WALLPAPER = Some((new_bmp, w, h, path.to_string()));
        return Some((new_bmp, w, h));
    }
    None
}

pub(crate) unsafe fn desktop_items() -> Vec<PathBuf> {
    let mut items = Vec::new();
    if let Some(profile) = env::var_os("USERPROFILE") {
        let desktop = PathBuf::from(profile).join("Desktop");
        if let Ok(entries) = fs::read_dir(&desktop) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.eq_ignore_ascii_case("desktop.ini") || name_str.starts_with('.') {
                    continue;
                }
                items.push(entry.path());
            }
        }
    }
    items.sort();
    items
}

pub(crate) unsafe fn desktop_item_at(hwnd: HWND, px: i32, py: i32) -> Option<PathBuf> {
    let items = desktop_items();
    if items.is_empty() {
        return None;
    }
    let mut r = RECT::default();
    let _ = GetClientRect(hwnd, &mut r);
    let w = r.right - r.left;
    let h = r.bottom - r.top;
    let origin_x = scaled(hwnd, 20);
    let origin_y = scaled(hwnd, 48);
    let cell_w = scaled(hwnd, 92);
    let cell_h = scaled(hwnd, 104);
    if px < origin_x || py < origin_y {
        return None;
    }
    let rows = ((h - origin_y - scaled(hwnd, 70)) / cell_h).max(1);
    let cols = ((w - origin_x) / cell_w).max(1);
    let col = (px - origin_x) / cell_w;
    let row = (py - origin_y) / cell_h;
    if col < 0 || row < 0 || row >= rows || col >= cols {
        return None;
    }
    let index = (col * rows + row) as usize;
    items.get(index).cloned()
}

/// Decode the configured wallpaper through WIC. Unlike `LoadImageW(...,
/// IMAGE_BITMAP, ...)`, WIC handles the JPG, PNG, GIF, TIFF, and BMP formats
/// that Windows allows in its wallpaper settings. The returned DIB is owned by
/// the caller and must be deleted with `DeleteObject`.
pub(crate) unsafe fn load_wallpaper_bitmap(path: &str) -> Option<(HBITMAP, i32, i32)> {
    let factory = CoCreateInstance::<_, windows::Win32::Graphics::Imaging::IWICImagingFactory>(
        &CLSID_WICImagingFactory,
        None,
        CLSCTX_INPROC_SERVER,
    )
    .ok()?;
    let filename = wide(path);
    let decoder = factory
        .CreateDecoderFromFilename(
            PCWSTR(filename.as_ptr()),
            None,
            windows::Win32::Foundation::GENERIC_ACCESS_RIGHTS(0x80000000),
            WICDecodeMetadataCacheOnLoad,
        )
        .ok()?;
    let frame = decoder.GetFrame(0).ok()?;
    let converter = factory.CreateFormatConverter().ok()?;
    converter
        .Initialize(
            &frame,
            &GUID_WICPixelFormat32bppBGR,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeCustom,
        )
        .ok()?;

    let mut width = 0u32;
    let mut height = 0u32;
    converter.GetSize(&mut width, &mut height).ok()?;
    if width == 0 || height == 0 || width > i32::MAX as u32 || height > i32::MAX as u32 {
        return None;
    }
    let stride = width.checked_mul(4)?;
    let mut pixels = vec![0u8; (stride as usize).checked_mul(height as usize)?];
    converter.CopyPixels(null_mut(), stride, &mut pixels).ok()?;

    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            // A negative height makes this a top-down DIB, matching WIC's
            // row order and avoiding an extra full-image vertical flip.
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = null_mut();
    let bitmap = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
    if bits.is_null() {
        let _ = DeleteObject(bitmap);
        return None;
    }
    std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
    Some((bitmap, width as i32, height as i32))
}

pub(crate) unsafe fn paint_desktop(hwnd: HWND, screen_dc: HDC) {
    let mut r = RECT::default();
    let _ = GetClientRect(hwnd, &mut r);
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

    let mut wallpaper_buf = [0u16; 512];
    let mut drawn = false;
    let mut wallpaper_path = String::new();

    if SystemParametersInfoW(
        SPI_GETDESKWALLPAPER,
        wallpaper_buf.len() as u32,
        Some(wallpaper_buf.as_mut_ptr() as *mut _),
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    )
    .is_ok()
    {
        let len = wallpaper_buf.iter().position(|&c| c == 0).unwrap_or(0);
        if len > 0 {
            let path = String::from_utf16_lossy(&wallpaper_buf[..len]);
            if Path::new(&path).exists() {
                wallpaper_path = path;
            }
        }
    }

    if wallpaper_path.is_empty() {
        if let Ok(reg_path) = read_registry_wallpaper() {
            if Path::new(&reg_path).exists() {
                wallpaper_path = reg_path;
            }
        }
    }

    if !wallpaper_path.is_empty() {
        if let Some((hbitmap, image_w, image_h)) = get_cached_wallpaper(&wallpaper_path) {
            let img_dc = CreateCompatibleDC(hdc);
            let img_old = SelectObject(img_dc, hbitmap);
            let _ = SetStretchBltMode(hdc, HALFTONE);
            let _ = SetBrushOrgEx(hdc, 0, 0, None);
            let _ = StretchBlt(hdc, 0, 0, w, h, img_dc, 0, 0, image_w, image_h, SRCCOPY);
            drawn = true;
            SelectObject(img_dc, img_old);
            let _ = DeleteDC(img_dc);
        }
    }

    if !drawn {
        let steps = h.min(256);
        for i in 0..steps {
            let y1 = (i * h) / steps;
            let y2 = ((i + 1) * h) / steps;
            let t = i as f32 / steps as f32;
            let r_col = (13.0 + t * 15.0) as u32;
            let g_col = (18.0 + t * 8.0) as u32;
            let b_col = (30.0 + t * 35.0) as u32;
            let color = (r_col << 16) | (g_col << 8) | b_col;
            let b = brush(color);
            let band = RECT {
                left: 0,
                top: y1,
                right: w,
                bottom: y2 + 1,
            };
            FillRect(hdc, &band, b);
            let _ = DeleteObject(b);
        }

        let cx = w / 2;
        let cy = (h / 2) - 40;
        icon_text(hdc, "\u{E707}", cx - 24, cy - 24, 0x334155, 48);
        text(hdc, "Luma Desktop", cx - 55, cy + 32, 0x475569, 18);
        text(
            hdc,
            "Right-click anywhere for menu · Win for applications",
            cx - 130,
            cy + 58,
            0x334155,
            12,
        );
    }

    // Desktop icons
    let items = desktop_items();
    if !items.is_empty() {
        let origin_x = scaled(hwnd, 20);
        let origin_y = scaled(hwnd, 48);
        let cell_w = scaled(hwnd, 92);
        let cell_h = scaled(hwnd, 104);
        let icon_size = scaled(hwnd, 48);
        let rows = ((h - origin_y - scaled(hwnd, 70)) / cell_h).max(1);
        let cols = ((w - origin_x) / cell_w).max(1);
        for (i, path) in items.iter().enumerate() {
            let row = (i as i32) % rows;
            let col = (i as i32) / rows;
            if col >= cols {
                break;
            }
            let x = origin_x + col * cell_w + (cell_w - icon_size) / 2;
            let y = origin_y + row * cell_h + 8;
            let icon = high_res_icon_for_path(&path.to_string_lossy());
            if !icon.is_invalid() {
                let _ = DrawIconEx(hdc, x, y, icon, icon_size, icon_size, 0, None, DI_NORMAL);
            }
            let label = path.file_name().and_then(|v| v.to_str()).unwrap_or("Item");
            let label = if label.len() > 14 {
                format!("{}...", &label[..14])
            } else {
                label.to_string()
            };
            let text_x = origin_x + col * cell_w + 4;
            let text_y = y + icon_size + 6;
            // Draw subtle drop shadow for high legibility
            text(hdc, &label, text_x + 1, text_y + 1, 0x000000, 12);
            text(hdc, &label, text_x, text_y, 0xFFFFFF, 12);
        }
    }

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) unsafe fn show_desktop_context_menu(hwnd: HWND, x: i32, y: i32) {
    let menu = CreatePopupMenu().unwrap();
    let menu_brush = apply_dark_menu_theme(menu);
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_TERMINAL,
        PCWSTR(wide("Terminal\tWin+T").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_FILES,
        PCWSTR(wide("File Explorer\tWin+E").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_LAUNCHER,
        PCWSTR(wide("Applications\tWin").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_SWITCHER,
        PCWSTR(wide("Window Switcher\tWin+Tab").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_SCREENSHOT,
        PCWSTR(wide("Take Screenshot\tPrint Screen").as_ptr()),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_SETTINGS,
        PCWSTR(wide("Luma Settings").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_DISPLAY,
        PCWSTR(wide("Display Settings").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_DESKTOP_PERSONALIZE,
        PCWSTR(wide("Personalization").as_ptr()),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_LOCK,
        PCWSTR(wide("Lock Workstation\tWin+L").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_LOGOFF,
        PCWSTR(wide("Sign Out").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_SLEEP,
        PCWSTR(wide("Sleep").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_RESTART,
        PCWSTR(wide("Restart").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_SHUTDOWN,
        PCWSTR(wide("Shut Down").as_ptr()),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_RESTORE_EXPLORER,
        PCWSTR(wide("Restore Explorer Shell").as_ptr()),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        ID_MENU_EXIT_SHELL,
        PCWSTR(wide("Exit to Explorer").as_ptr()),
    );

    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, x, y, 0, hwnd, None);
    let _ = DestroyMenu(menu);
    let _ = DeleteObject(menu_brush);

    match cmd.0 as usize {
        ID_DESKTOP_TERMINAL => launch_terminal(),
        ID_DESKTOP_FILES => launch("explorer.exe"),
        ID_DESKTOP_LAUNCHER => show_launcher(),
        ID_DESKTOP_SWITCHER => show_switcher(),
        ID_DESKTOP_SCREENSHOT => start_snip(),
        ID_DESKTOP_DISPLAY => launch("ms-settings:display"),
        ID_DESKTOP_PERSONALIZE => launch("ms-settings:personalization"),
        ID_MENU_SETTINGS => {
            show_luma_settings();
        }
        ID_MENU_LOCK => {
            let _ = LockWorkStation();
        }
        ID_MENU_LOGOFF => {
            let _ = ExitWindowsEx(EWX_LOGOFF, SHUTDOWN_REASON(0));
        }
        ID_MENU_SLEEP => {
            launch("rundll32.exe powrprof.dll,SetSuspendState 0,1,0");
        }
        ID_MENU_RESTART => {
            let _ = ExitWindowsEx(EWX_REBOOT, SHUTDOWN_REASON(0));
        }
        ID_MENU_SHUTDOWN => {
            let _ = ExitWindowsEx(EWX_SHUTDOWN, SHUTDOWN_REASON(0));
        }
        ID_MENU_RESTORE_EXPLORER => restore_explorer(),
        ID_MENU_EXIT_SHELL => exit_to_explorer(),
        _ => {}
    }
}

pub(crate) extern "system" fn desktop_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_WINDOWPOSCHANGING => {
                let pos = &mut *(l.0 as *mut WINDOWPOS);
                if pos.flags.0 & SWP_NOZORDER.0 == 0 {
                    pos.hwndInsertAfter = HWND_BOTTOM;
                }
                DefWindowProcW(hwnd, msg, w, l)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                paint_desktop(hwnd, hdc);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                if IsWindowVisible(LAUNCHER).as_bool() {
                    ShowWindow(LAUNCHER, SW_HIDE);
                }
                if IsWindowVisible(SWITCHER).as_bool() {
                    clear_switcher_thumbnail();
                    ShowWindow(SWITCHER, SW_HIDE);
                }
                if IsWindowVisible(AUDIO_MIXER).as_bool() {
                    ShowWindow(AUDIO_MIXER, SW_HIDE);
                }
                LRESULT(0)
            }
            WM_LBUTTONDBLCLK => {
                let x = ((l.0 & 0xffff) as i16) as i32;
                let y = (((l.0 >> 16) & 0xffff) as i16) as i32;
                if let Some(path) = desktop_item_at(hwnd, x, y) {
                    launch(&path.to_string_lossy());
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                let pt_x = ((l.0 & 0xffff) as i16) as i32;
                let pt_y = (((l.0 >> 16) & 0xffff) as i16) as i32;
                let mut pt = POINT { x: pt_x, y: pt_y };
                let _ = ClientToScreen(hwnd, &mut pt);
                show_desktop_context_menu(hwnd, pt.x, pt.y);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
