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
static mut CACHED_ORBITAL_DIB: Option<(HBITMAP, i32, i32)> = None;

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn orbital_hash(x: f32, y: f32) -> f32 {
    let n = (x * 127.1 + y * 311.7).sin() * 43758.5453;
    n - n.floor()
}

fn orbital_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let a = orbital_hash(ix, iy);
    let b = orbital_hash(ix + 1.0, iy);
    let c = orbital_hash(ix, iy + 1.0);
    let d = orbital_hash(ix + 1.0, iy + 1.0);
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}

fn orbital_fbm(mut x: f32, mut y: f32) -> f32 {
    let mut sum = 0.0f32;
    let mut amp = 0.52f32;
    for _ in 0..4 {
        sum += amp * orbital_noise(x, y);
        let nx = 0.8 * x - 0.6 * y + 11.4;
        let ny = 0.6 * x + 0.8 * y + 7.2;
        x = nx * 2.04;
        y = ny * 2.04;
        amp *= 0.5;
    }
    sum
}

/// Generates and caches a 32-bit DIB rendering the COSMIC Orbital Earth planetary horizon
/// (deep space upper-left, cobalt Rayleigh atmosphere arc, terracotta Sahara terrain lower-right).
unsafe fn get_cached_orbital_earth_bitmap(w: i32, h: i32) -> Option<(HBITMAP, i32, i32)> {
    let rw = (w / 2).clamp(320, 960);
    let rh = (h / 2).clamp(180, 540);
    if let Some((bmp, cw, ch)) = CACHED_ORBITAL_DIB {
        if cw == rw && ch == rh && !bmp.is_invalid() {
            return Some((bmp, cw, ch));
        }
        let _ = DeleteObject(bmp);
        CACHED_ORBITAL_DIB = None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: rw,
            biHeight: -rh,
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
    let aspect = rw as f32 / rh.max(1) as f32;
    let center_x = 1.18 * aspect;
    let center_y = -0.78f32;
    let radius = 1.62f32;
    let out = bits as *mut u32;

    for py in 0..rh {
        let uv_y = 1.0 - (py as f32 / rh as f32);
        let row_off = (py * rw) as usize;
        for px in 0..rw {
            let uv_x = px as f32 / rw as f32;
            let dx = uv_x * aspect - center_x;
            let dy = uv_y - center_y;
            let d = (dx * dx + dy * dy).sqrt();
            let dist_from_limb = d - radius;

            let (mut r, mut g, mut b) = (0.002f32, 0.0025f32, 0.004f32);
            if dist_from_limb < 0.0 {
                let depth = (-dist_from_limb).clamp(0.0, 1.0);
                let tx = dx * 2.1 - dy * 1.45;
                let ty = dx * 1.35 + dy * 2.15;
                let warp = orbital_fbm(tx * 1.5, ty * 1.5);
                let ridges = orbital_fbm(tx * 3.1 + warp * 1.2, ty * 2.8 - warp * 0.9);
                let dunes = orbital_fbm(tx * 6.0, ty * 5.2 + ridges * 1.1);

                // Warm ochre / terracotta Sahara palette with dark basalt plateaus
                let mut tr = 0.86 * (1.0 - ridges) + 0.78 * ridges;
                let mut tg = 0.56 * (1.0 - ridges) + 0.42 * ridges;
                let mut tb = 0.34 * (1.0 - ridges) + 0.25 * ridges;

                let basalt = smoothstep(0.47, 0.72, orbital_fbm(tx * 2.3 + 4.1, ty * 2.5 - 2.3));
                tr = tr * (1.0 - basalt * 0.68) + 0.16 * (basalt * 0.68);
                tg = tg * (1.0 - basalt * 0.68) + 0.15 * (basalt * 0.68);
                tb = tb * (1.0 - basalt * 0.68) + 0.17 * (basalt * 0.68);

                let dune_boost = smoothstep(0.45, 0.82, dunes) * 0.18;
                tr = (tr + dune_boost * 0.95).min(1.0);
                tg = (tg + dune_boost * 0.68).min(1.0);
                tb = (tb + dune_boost * 0.42).min(1.0);

                // Atmospheric Rayleigh haze near the curved planetary limb
                let limb_haze = (-depth * 11.0).exp();
                let mid_haze = (-depth * 3.6).exp();
                r = tr * (1.0 - mid_haze * 0.42) + 0.30 * (mid_haze * 0.42);
                g = tg * (1.0 - mid_haze * 0.42) + 0.52 * (mid_haze * 0.42);
                b = tb * (1.0 - mid_haze * 0.42) + 0.92 * (mid_haze * 0.42);

                r = r * (1.0 - limb_haze * 0.82) + 0.22 * (limb_haze * 0.82);
                g = g * (1.0 - limb_haze * 0.82) + 0.58 * (limb_haze * 0.82);
                b = b * (1.0 - limb_haze * 0.82) + 1.00 * (limb_haze * 0.82);
            } else {
                // Electric cobalt outer atmospheric limb glow into pitch-black space
                let inner_glow = (-dist_from_limb * 26.0).exp();
                let outer_glow = (-dist_from_limb * 9.5).exp() * 0.55;
                r += inner_glow * 0.14 + outer_glow * 0.04;
                g += inner_glow * 0.48 + outer_glow * 0.18;
                b += inner_glow * 1.00 + outer_glow * 0.62;
            }

            let ur = (r.clamp(0.0, 1.0) * 255.0) as u32;
            let ug = (g.clamp(0.0, 1.0) * 255.0) as u32;
            let ub = (b.clamp(0.0, 1.0) * 255.0) as u32;
            *out.add(row_off + px as usize) = (ur << 16) | (ug << 8) | ub;
        }
    }

    CACHED_ORBITAL_DIB = Some((bitmap, rw, rh));
    Some((bitmap, rw, rh))
}

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
        if let Some((orb_bmp, ow, oh)) = get_cached_orbital_earth_bitmap(w, h) {
            let img_dc = CreateCompatibleDC(hdc);
            let img_old = SelectObject(img_dc, orb_bmp);
            let _ = SetStretchBltMode(hdc, HALFTONE);
            let _ = SetBrushOrgEx(hdc, 0, 0, None);
            let _ = StretchBlt(hdc, 0, 0, w, h, img_dc, 0, 0, ow, oh, SRCCOPY);
            SelectObject(img_dc, img_old);
            let _ = DeleteDC(img_dc);
        } else {
            let b = brush(0x020408);
            FillRect(hdc, &r, b);
            let _ = DeleteObject(b);
        }
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
