use super::*;

pub(crate) extern "system" fn switcher_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_switcher(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                if y >= 90 {
                    let index = ((y - 90) / 34) as usize;
                    SWITCHER_SELECTION = index.min(8);
                    notify_accessibility_focus(hwnd, SWITCHER_SELECTION as i32);
                    activate_window(index);
                    clear_switcher_thumbnail();
                    ShowWindow(hwnd, SW_HIDE);
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let max = WINDOW_LIST.len().min(9).saturating_sub(1);
                match w.0 as u32 {
                    value if value == VK_UP.0 as u32 => {
                        SWITCHER_SELECTION = SWITCHER_SELECTION.saturating_sub(1);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_DOWN.0 as u32 => {
                        SWITCHER_SELECTION = (SWITCHER_SELECTION + 1).min(max);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_HOME.0 as u32 => {
                        SWITCHER_SELECTION = 0;
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_END.0 as u32 => {
                        SWITCHER_SELECTION = max;
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_PRIOR.0 as u32 => {
                        SWITCHER_SELECTION = SWITCHER_SELECTION.saturating_sub(3);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_NEXT.0 as u32 => {
                        SWITCHER_SELECTION = (SWITCHER_SELECTION + 3).min(max);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                    value if value == VK_RETURN.0 as u32 => {
                        activate_window(SWITCHER_SELECTION);
                        clear_switcher_thumbnail();
                        ShowWindow(hwnd, SW_HIDE);
                    }
                    value if value == VK_ESCAPE.0 as u32 => {
                        clear_switcher_thumbnail();
                        ShowWindow(hwnd, SW_HIDE);
                    }
                    _ => {}
                }
                if !WINDOW_LIST.is_empty() {
                    notify_accessibility_focus(hwnd, SWITCHER_SELECTION as i32);
                }
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_DPICHANGED => {
                apply_dpi_suggested_rect(hwnd, l);
                LRESULT(0)
            }
            WM_CLOSE => {
                clear_switcher_thumbnail();
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

pub(crate) unsafe extern "system" fn tray_notify_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, w, l)
}
