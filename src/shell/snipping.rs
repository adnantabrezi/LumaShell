use super::*;

pub(crate) unsafe fn snip_timestamp() -> String {
    let time = GetLocalTime();
    format!(
        "Luma_{:04}{:02}{:02}_{:02}{:02}{:02}.png",
        time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond
    )
}

pub(crate) unsafe fn capture_virtual_screen() -> bool {
    if !SNIP_BITMAP.is_invalid() {
        let _ = DeleteObject(SNIP_BITMAP);
        SNIP_BITMAP = HBITMAP(null_mut());
    }
    let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
    let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    if width <= 0 || height <= 0 {
        return false;
    }
    let screen = GetDC(None);
    if screen.is_invalid() {
        return false;
    }
    let dc = CreateCompatibleDC(screen);
    let bitmap = CreateCompatibleBitmap(screen, width, height);
    let old = SelectObject(dc, bitmap);
    let _ = BitBlt(dc, 0, 0, width, height, screen, x, y, SRCCOPY);
    SelectObject(dc, old);
    let _ = DeleteDC(dc);
    let _ = ReleaseDC(None, screen);
    if bitmap.is_invalid() {
        return false;
    }
    SNIP_BITMAP = bitmap;
    SNIP_SCREEN_X = x;
    SNIP_SCREEN_Y = y;
    SNIP_WIDTH = width;
    SNIP_HEIGHT = height;
    true
}

pub(crate) unsafe fn start_snip() {
    if !SNIP_WINDOW.is_invalid() {
        let _ = SetForegroundWindow(SNIP_WINDOW);
        return;
    }
    if !capture_virtual_screen() {
        log_event("screenshot capture failed");
        return;
    }
    SNIP_ORIGIN = POINT { x: 0, y: 0 };
    SNIP_CURRENT = POINT { x: 0, y: 0 };
    SNIP_DRAGGING = false;
    SNIP_HAS_SELECTION = false;
    let hwnd = match CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
        PCWSTR(wide("LumaSnip").as_ptr()),
        PCWSTR(wide("Luma Snip").as_ptr()),
        WS_POPUP | WS_VISIBLE,
        SNIP_SCREEN_X,
        SNIP_SCREEN_Y,
        SNIP_WIDTH,
        SNIP_HEIGHT,
        None,
        None,
        INSTANCE,
        None,
    ) {
        Ok(hwnd) => hwnd,
        Err(_) => {
            log_event("screenshot overlay creation failed");
            return;
        }
    };
    SNIP_WINDOW = hwnd;
    let _ = SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        SNIP_SCREEN_X,
        SNIP_SCREEN_Y,
        SNIP_WIDTH,
        SNIP_HEIGHT,
        SWP_SHOWWINDOW,
    );
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
    let _ = SetFocus(hwnd);
    let _ = InvalidateRect(hwnd, None, false);
    log_event("snipping overlay opened");
}

pub(crate) unsafe fn cleanup_snip() {
    SNIP_WINDOW = HWND(null_mut());
    SNIP_DRAGGING = false;
    SNIP_HAS_SELECTION = false;
    if !SNIP_BITMAP.is_invalid() {
        let _ = DeleteObject(SNIP_BITMAP);
        SNIP_BITMAP = HBITMAP(null_mut());
    }
}

pub(crate) unsafe fn close_snip() {
    if !SNIP_WINDOW.is_invalid() {
        let _ = DestroyWindow(SNIP_WINDOW);
    }
}

pub(crate) unsafe fn snip_selection() -> RECT {
    let left = SNIP_ORIGIN.x.min(SNIP_CURRENT.x).clamp(0, SNIP_WIDTH);
    let top = SNIP_ORIGIN.y.min(SNIP_CURRENT.y).clamp(0, SNIP_HEIGHT);
    let right = SNIP_ORIGIN.x.max(SNIP_CURRENT.x).clamp(0, SNIP_WIDTH);
    let bottom = SNIP_ORIGIN.y.max(SNIP_CURRENT.y).clamp(0, SNIP_HEIGHT);
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

pub(crate) unsafe fn bitmap_pixels(bitmap: HBITMAP, width: i32, height: i32) -> Option<Vec<u8>> {
    if width <= 0 || height <= 0 || bitmap.is_invalid() {
        return None;
    }
    let screen = GetDC(None);
    let dc = CreateCompatibleDC(screen);
    let mut info = BITMAPINFO::default();
    info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB.0;
    info.bmiHeader.biSizeImage = (width * height * 4) as u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        dc,
        bitmap,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut info,
        DIB_RGB_COLORS,
    );
    let _ = DeleteDC(dc);
    let _ = ReleaseDC(None, screen);
    if lines == 0 { None } else { Some(pixels) }
}

pub(crate) unsafe fn save_snip_png(bitmap: HBITMAP, width: i32, height: i32) -> Option<PathBuf> {
    let pixels = bitmap_pixels(bitmap, width, height)?;
    let stride = (width * 4) as usize;
    let mut rgba = vec![0u8; pixels.len()];
    for row in 0..height as usize {
        let source_row = (height as usize - 1 - row) * stride;
        let target_row = row * stride;
        for column in 0..width as usize {
            let source = source_row + column * 4;
            let target = target_row + column * 4;
            rgba[target] = pixels[source + 2];
            rgba[target + 1] = pixels[source + 1];
            rgba[target + 2] = pixels[source];
            rgba[target + 3] = 255;
        }
    }
    let directory = env::var_os("USERPROFILE")
        .map(PathBuf::from)?
        .join("Pictures")
        .join("Luma Screenshots");
    let _ = fs::create_dir_all(&directory);
    let path = directory.join(snip_timestamp());
    let file = fs::File::create(&path).ok()?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().ok()?;
    png_writer.write_image_data(&rgba).ok()?;
    let _ = png_writer.finish();
    Some(path)
}

pub(crate) unsafe fn copy_snip_to_clipboard(bitmap: HBITMAP, width: i32, height: i32) {
    let pixels = match bitmap_pixels(bitmap, width, height) {
        Some(pixels) => pixels,
        None => return,
    };
    if OpenClipboard(SNIP_WINDOW).is_err() {
        return;
    }
    let _ = EmptyClipboard();

    let screen = GetDC(None);
    let dc = CreateCompatibleDC(screen);
    let copy = CreateCompatibleBitmap(screen, width, height);
    let old = SelectObject(dc, copy);
    let source = CreateCompatibleDC(screen);
    let source_old = SelectObject(source, bitmap);
    let _ = BitBlt(dc, 0, 0, width, height, source, 0, 0, SRCCOPY);
    SelectObject(source, source_old);
    let _ = DeleteDC(source);
    SelectObject(dc, old);
    let _ = DeleteDC(dc);
    let _ = ReleaseDC(None, screen);
    if !copy.is_invalid() {
        let _ = SetClipboardData(2, HANDLE(copy.0));
    }

    let header_size = size_of::<BITMAPINFOHEADER>();
    let total = header_size + pixels.len();
    if let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, total) {
        let pointer = GlobalLock(memory);
        if !pointer.is_null() {
            let mut info = BITMAPINFO::default();
            info.bmiHeader.biSize = header_size as u32;
            info.bmiHeader.biWidth = width;
            info.bmiHeader.biHeight = height;
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB.0;
            info.bmiHeader.biSizeImage = pixels.len() as u32;
            std::ptr::copy_nonoverlapping(
                &info.bmiHeader as *const BITMAPINFOHEADER as *const u8,
                pointer as *mut u8,
                header_size,
            );
            std::ptr::copy_nonoverlapping(
                pixels.as_ptr(),
                (pointer as *mut u8).add(header_size),
                pixels.len(),
            );
            let _ = GlobalUnlock(memory);
            let _ = SetClipboardData(8, HANDLE(memory.0));
        }
    }
    let _ = CloseClipboard();
}

pub(crate) unsafe fn finish_snip() {
    let selection = snip_selection();
    let width = selection.right - selection.left;
    let height = selection.bottom - selection.top;
    if width < 4 || height < 4 || SNIP_BITMAP.is_invalid() {
        close_snip();
        return;
    }
    let screen = GetDC(None);
    let dc = CreateCompatibleDC(screen);
    let crop = CreateCompatibleBitmap(screen, width, height);
    let old = SelectObject(dc, crop);
    let source = CreateCompatibleDC(screen);
    let source_old = SelectObject(source, SNIP_BITMAP);
    let _ = BitBlt(
        dc,
        0,
        0,
        width,
        height,
        source,
        selection.left,
        selection.top,
        SRCCOPY,
    );
    SelectObject(source, source_old);
    let _ = DeleteDC(source);
    SelectObject(dc, old);
    let _ = DeleteDC(dc);
    let _ = ReleaseDC(None, screen);

    if let Some(path) = save_snip_png(crop, width, height) {
        log_event(&format!("screenshot saved to {}", path.display()));
    }
    copy_snip_to_clipboard(crop, width, height);
    show_screenshot_osd();
    let _ = DeleteObject(crop);
    close_snip();
}

pub(crate) unsafe fn paint_snip(hwnd: HWND, screen_dc: HDC) {
    let mut rect = RECT::default();
    if GetClientRect(hwnd, &mut rect).is_err() {
        return;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return;
    }
    let hdc = CreateCompatibleDC(screen_dc);
    let memory = CreateCompatibleBitmap(screen_dc, width, height);
    let old = SelectObject(hdc, memory);
    let source = CreateCompatibleDC(screen_dc);
    let source_old = if SNIP_BITMAP.is_invalid() {
        SelectObject(source, CreateCompatibleBitmap(screen_dc, 1, 1))
    } else {
        SelectObject(source, SNIP_BITMAP)
    };
    let _ = BitBlt(hdc, 0, 0, width, height, source, 0, 0, SRCCOPY);

    let dim_dc = CreateCompatibleDC(screen_dc);
    let dim_bitmap = CreateCompatibleBitmap(screen_dc, 1, 1);
    let dim_old = SelectObject(dim_dc, dim_bitmap);
    let black = brush(0x000000);
    let black_rect = RECT {
        left: 0,
        top: 0,
        right: 1,
        bottom: 1,
    };
    FillRect(dim_dc, &black_rect, black);
    let _ = DeleteObject(black);
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 110,
        AlphaFormat: 0,
    };
    let _ = AlphaBlend(hdc, 0, 0, width, height, dim_dc, 0, 0, 1, 1, blend);
    SelectObject(dim_dc, dim_old);
    let _ = DeleteObject(dim_bitmap);
    let _ = DeleteDC(dim_dc);

    if SNIP_HAS_SELECTION || SNIP_DRAGGING {
        let selection = snip_selection();
        if selection.right > selection.left && selection.bottom > selection.top {
            let _ = BitBlt(
                hdc,
                selection.left,
                selection.top,
                selection.right - selection.left,
                selection.bottom - selection.top,
                source,
                selection.left,
                selection.top,
                SRCCOPY,
            );
            let border = brush(0x38bdf8);
            let _ = FrameRect(hdc, &selection, border);
            let _ = DeleteObject(border);

            let label = format!(
                "{} x {}",
                selection.right - selection.left,
                selection.bottom - selection.top
            );
            let label_x = selection.left + 8;
            let label_y = (selection.top - 26).max(8);
            let label_background = brush(0x10141c);
            let label_rect = RECT {
                left: label_x - 6,
                top: label_y - 4,
                right: label_x + label.len() as i32 * 8 + 6,
                bottom: label_y + 20,
            };
            fill_round_rect(hdc, &label_rect, label_background, 8);
            let _ = DeleteObject(label_background);
            text(hdc, &label, label_x, label_y, 0xffffff, 14);
        }
    }

    let hint = "Drag to capture   -   Right-click or Esc to cancel";
    let hint_background = brush(0x10141c);
    let hint_rect = RECT {
        left: 20,
        top: 16,
        right: 20 + hint.len() as i32 * 8 + 20,
        bottom: 52,
    };
    fill_round_rect(hdc, &hint_rect, hint_background, 12);
    let _ = DeleteObject(hint_background);
    text(hdc, hint, 34, 24, 0xffffff, 15);

    let _ = BitBlt(screen_dc, 0, 0, width, height, hdc, 0, 0, SRCCOPY);
    SelectObject(source, source_old);
    let _ = DeleteDC(source);
    SelectObject(hdc, old);
    let _ = DeleteObject(memory);
    let _ = DeleteDC(hdc);
}

pub(crate) extern "system" fn snip_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_snip(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_SETCURSOR => {
                if let Ok(cursor) = LoadCursorW(None, IDC_CROSS) {
                    let _ = SetCursor(cursor);
                }
                LRESULT(1)
            }
            WM_LBUTTONDOWN => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                SNIP_ORIGIN = POINT { x, y };
                SNIP_CURRENT = POINT { x, y };
                SNIP_DRAGGING = true;
                SNIP_HAS_SELECTION = true;
                SetCapture(hwnd);
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if SNIP_DRAGGING {
                    SNIP_CURRENT = POINT {
                        x: (l.0 as u32 & 0xffff) as i32,
                        y: ((l.0 as u32 >> 16) & 0xffff) as i32,
                    };
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                if SNIP_DRAGGING {
                    let x = (l.0 as u32 & 0xffff) as i32;
                    let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                    SNIP_CURRENT = POINT { x, y };
                    SNIP_DRAGGING = false;
                    ReleaseCapture();
                    finish_snip();
                }
                LRESULT(0)
            }
            WM_RBUTTONUP | WM_MBUTTONUP => {
                close_snip();
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                close_snip();
                LRESULT(0)
            }
            WM_DESTROY => {
                cleanup_snip();
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
