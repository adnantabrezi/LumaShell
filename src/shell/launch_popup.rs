use super::*;

pub(crate) extern "system" fn launcher_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                LAUNCHER_EDIT = CreateWindowExW(
                    WS_EX_CLIENTEDGE,
                    PCWSTR(wide("EDIT").as_ptr()),
                    PCWSTR(wide("").as_ptr()),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | ES_AUTOHSCROLL as u32),
                    28,
                    76,
                    LAUNCHER_WIDTH - 56,
                    30,
                    hwnd,
                    HMENU(ID_SEARCH as *mut _),
                    GetModuleHandleW(None).unwrap(),
                    None,
                )
                .unwrap();
                let cue = wide("Search applications");
                let _ = SendMessageW(
                    LAUNCHER_EDIT,
                    EM_SETCUEBANNER,
                    WPARAM(1),
                    LPARAM(cue.as_ptr() as isize),
                );
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_launcher(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_COMMAND => {
                if (w.0 & 0xffff) == ID_SEARCH {
                    LAUNCHER_SELECTION = 0;
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                if y >= 135 {
                    LAUNCHER_SELECTION = ((y - 135) / 34).max(0);
                    notify_accessibility_focus(hwnd, LAUNCHER_SELECTION);
                    let query = launcher_query();
                    if let Some((_, target)) =
                        launcher_entries(&query).get(((y - 135) / 34) as usize)
                    {
                        launch(target);
                        ShowWindow(hwnd, SW_HIDE);
                    }
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                if y >= 135 {
                    let query = launcher_query();
                    if let Some((_, target)) =
                        launcher_entries(&query).get(((y - 135) / 34) as usize)
                    {
                        toggle_favorite(target);
                        let _ = InvalidateRect(hwnd, None, false);
                    }
                }
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_DPICHANGED => {
                apply_dpi_suggested_rect(hwnd, l);
                LRESULT(0)
            }
            WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
