use super::*;

pub(crate) extern "system" fn dock_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_SIZE => {
                apply_dock_region(hwnd);
                paint_dock(hwnd, HDC::default());
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_dock(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_TIMER => {
                if w.0 == 2 {
                    update_fullscreen_state();
                    update_auto_hide();
                    let _ = running_window_count();
                    let desired = scaled(hwnd, dock_logical_width());
                    let mut client = RECT::default();
                    if GetClientRect(hwnd, &mut client).is_ok()
                        && client.right - client.left != desired
                    {
                        position_dock_on_monitor(
                            hwnd,
                            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                        );
                    }
                    for &secondary in SECONDARY_DOCKS.iter().take(SECONDARY_COUNT) {
                        let mut secondary_client = RECT::default();
                        if GetClientRect(secondary, &mut secondary_client).is_ok()
                            && secondary_client.right - secondary_client.left
                                != scaled(secondary, dock_logical_width())
                        {
                            position_dock_on_monitor(
                                secondary,
                                MonitorFromWindow(secondary, MONITOR_DEFAULTTONEAREST),
                            );
                        }
                    }
                    paint_dock(hwnd, HDC::default());
                } else if w.0 == 3 {
                    let target = if DOCK_POINTER_X >= 0 { 1.0 } else { 0.0 };
                    if (target - DOCK_MAGNIFICATION).abs() > 0.005 {
                        if !animations_enabled() {
                            DOCK_MAGNIFICATION = target;
                        } else {
                            let delta = target - DOCK_MAGNIFICATION;
                            if delta.abs() < 0.02 {
                                DOCK_MAGNIFICATION = target;
                            } else {
                                DOCK_MAGNIFICATION += delta * 0.22;
                            }
                        }
                        paint_dock(hwnd, HDC::default());
                    }
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let index = dock_index_at(x);
                HOVERED_DOCK_ICON = index;
                paint_dock(hwnd, HDC::default());
                let mut point = POINT::default();
                let _ = GetCursorPos(&mut point);
                show_dock_context_menu(hwnd, index, point.x, point.y);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let x = (l.0 as u32 & 0xffff) as i32;
                DOCK_POINTER_X = x;
                let next = dock_index_at(x);
                if DRAGGED_DOCK_ICON >= 0 && next >= 0 && next != DRAGGED_DOCK_ICON {
                    reorder_dock_icon(DRAGGED_DOCK_ICON, next);
                    DRAGGED_DOCK_ICON = next;
                    DOCK_DRAG_MOVED = true;
                }
                if HOVERED_DOCK_ICON != next {
                    HOVERED_DOCK_ICON = next;
                    notify_accessibility_focus(hwnd, next);
                    paint_dock(hwnd, HDC::default());
                }
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                let _ = TrackMouseEvent(&mut tracking);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let index = dock_index_at(x);
                if y <= scaled(hwnd, DOCK_HEIGHT) && index >= 0 {
                    DRAGGED_DOCK_ICON = index;
                    DOCK_DRAG_MOVED = false;
                    SetCapture(hwnd);
                }
                LRESULT(0)
            }
            WM_MOUSE_LEAVE => {
                HOVERED_DOCK_ICON = -1;
                DOCK_POINTER_X = -1;
                paint_dock(hwnd, HDC::default());
                LRESULT(0)
            }
            WM_CAPTURECHANGED => {
                DRAGGED_DOCK_ICON = -1;
                DOCK_DRAG_MOVED = false;
                LRESULT(0)
            }
            WM_KEYDOWN => {
                match w.0 as u32 {
                    value if value == VK_LEFT.0 as u32 => {
                        HOVERED_DOCK_ICON = (HOVERED_DOCK_ICON - 1).max(0);
                        notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                    }
                    value if value == VK_RIGHT.0 as u32 => {
                        HOVERED_DOCK_ICON =
                            (HOVERED_DOCK_ICON + 1).min(dock_total_count() as i32 - 1);
                        notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                    }
                    value if value == VK_HOME.0 as u32 => {
                        HOVERED_DOCK_ICON = 0;
                        notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                    }
                    value if value == VK_END.0 as u32 => {
                        HOVERED_DOCK_ICON = dock_total_count() as i32 - 1;
                        notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                    }
                    value if value == VK_RETURN.0 as u32 => activate_dock_icon(HOVERED_DOCK_ICON),
                    value if value == VK_ESCAPE.0 as u32 => toggle_dock(),
                    _ => {}
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let _y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let was_dragged = DRAGGED_DOCK_ICON >= 0;
                let was_moved = DOCK_DRAG_MOVED;
                if was_dragged {
                    ReleaseCapture();
                    DRAGGED_DOCK_ICON = -1;
                    DOCK_DRAG_MOVED = false;
                }
                if was_dragged && was_moved {
                    return LRESULT(0);
                }
                let index = dock_index_at(x);
                if index >= 0 {
                    activate_dock_icon(index);
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
