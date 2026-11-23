use super::*;

pub(crate) struct ClockApplet;
pub(crate) struct NetworkApplet;
pub(crate) struct VolumeApplet;
pub(crate) struct BluetoothApplet;
pub(crate) struct MediaApplet;
pub(crate) struct BatteryApplet;
pub(crate) struct NotificationApplet;
pub(crate) fn format_date(n: SYSTEMTIME, format: u8) -> String {
    match format {
        0 => String::new(),
        2 => {
            let weekdays = [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ];
            let months = [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ];
            format!(
                "{}, {} {}",
                weekdays[n.wDayOfWeek.min(6) as usize],
                months[n.wMonth.saturating_sub(1).min(11) as usize],
                n.wDay
            )
        }
        _ => format!("{:04}-{:02}-{:02}", n.wYear, n.wMonth, n.wDay),
    }
}
impl Applet for ClockApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_clock
    }
    fn width(&self, _c: Config) -> i32 {
        let n: SYSTEMTIME = unsafe { GetLocalTime() };
        let time = if unsafe { CONFIG.clock_24h } {
            format!("{:02}:{:02}", n.wHour, n.wMinute)
        } else {
            let hour = if n.wHour == 0 {
                12
            } else if n.wHour > 12 {
                n.wHour - 12
            } else {
                n.wHour
            };
            let suffix = if n.wHour >= 12 { "PM" } else { "AM" };
            format!("{}:{:02} {}", hour, n.wMinute, suffix)
        };
        let date = format_date(n, unsafe { CONFIG.date_format });
        let value = if date.is_empty() {
            time
        } else {
            format!("{} · {}", time, date)
        };
        (value.chars().count() as i32 * 8 + 32).max(96)
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let n: SYSTEMTIME = unsafe { GetLocalTime() };
        let time = if unsafe { CONFIG.clock_24h } {
            format!("{:02}:{:02}", n.wHour, n.wMinute)
        } else {
            let hour = if n.wHour == 0 {
                12
            } else if n.wHour > 12 {
                n.wHour - 12
            } else {
                n.wHour
            };
            let suffix = if n.wHour >= 12 { "PM" } else { "AM" };
            format!("{}:{:02} {}", hour, n.wMinute, suffix)
        };
        let date = format_date(n, unsafe { CONFIG.date_format });
        let value = if date.is_empty() {
            time
        } else {
            format!("{} · {}", time, date)
        };
        let width = (value.chars().count() as i32 * 8 + 32).max(96);
        // Center the ink within the applet's allotted box so a centered zone is visually
        // centered instead of sitting at the left edge of its estimated width.
        let content = unsafe { 20 + measure_text_with_face(hdc, &value, 13, "Segoe UI") };
        let x = *right - width + (width - content) / 2;
        unsafe {
            icon_text(hdc, "\u{E823}", x, 10, 0x94a3b8, 14);
            text(hdc, &value, x + 20, 10, 0xe8ecf2, 13);
        }
        *right -= width;
    }
}
impl Applet for NetworkApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_network
    }
    fn width(&self, _c: Config) -> i32 {
        36
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let (connected, wifi, wifi_details) = unsafe { network_snapshot() };
        let icon = if wifi {
            if !connected {
                "\u{E783}"
            } else {
                let quality = wifi_details.as_ref().map(|(_, q)| *q).unwrap_or(100);
                if quality >= 75 {
                    "\u{E701}"
                } else if quality >= 50 {
                    "\u{E874}"
                } else if quality >= 25 {
                    "\u{E873}"
                } else {
                    "\u{E872}"
                }
            }
        } else if connected {
            "\u{E839}"
        } else {
            "\u{E783}"
        };
        let x = *right - 36 + (36 - 16) / 2;
        unsafe {
            icon_text(
                hdc,
                icon,
                x,
                10,
                if connected { 0x38bdf8 } else { 0xf87171 },
                16,
            );
        }
        *right -= 36;
    }
}
impl Applet for VolumeApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_volume
    }
    fn width(&self, _c: Config) -> i32 {
        let (_level, _muted) = unsafe { default_audio_state().unwrap_or((0, false)) };
        let sessions = unsafe { active_audio_session_count() };
        let label = if sessions > 0 {
            format!("{}% · {}", _level, sessions)
        } else {
            format!("{}%", _level)
        };
        let text_w = label.len() as i32 * 8 + 4;
        text_w + 24 + 10
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let (level, muted) = unsafe { default_audio_state().unwrap_or((0, false)) };
        let sessions = unsafe { active_audio_session_count() };
        let icon = if muted {
            "\u{E74F}"
        } else if level < 33 {
            "\u{E993}"
        } else if level < 66 {
            "\u{E994}"
        } else {
            "\u{E767}"
        };
        let icon_col = if muted { 0xf87171 } else { 0x38bdf8 };
        let label = if sessions > 0 {
            format!("{}% · {}", level, sessions)
        } else {
            format!("{}%", level)
        };
        let text_w = label.len() as i32 * 8 + 4;
        let width = text_w + 24;
        let footprint = width + 10;
        let content = unsafe { 20 + measure_text_with_face(hdc, &label, 13, "Segoe UI") };
        let x = *right - footprint + (footprint - content) / 2;
        unsafe {
            icon_text(hdc, icon, x, 10, icon_col, 15);
            text(hdc, &label, x + 20, 10, 0xe2e8f0, 13);
        }
        *right -= footprint;
    }
}
impl Applet for BluetoothApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_bluetooth
    }
    fn width(&self, _c: Config) -> i32 {
        36
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let (present, any_connected) = unsafe { bluetooth_status_snapshot() };
        let icon = if !present { "\u{E711}" } else { "\u{E702}" };
        let color = if any_connected {
            0x38bdf8
        } else if present {
            0x38bdf8
        } else {
            0x7f8a9a
        };
        let x = *right - 36 + (36 - 16) / 2;
        unsafe {
            icon_text(hdc, icon, x, 10, color, 15);
        }
        *right -= 36;
    }
}
impl Applet for MediaApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_media
    }
    fn width(&self, _c: Config) -> i32 {
        if unsafe { media_playing() }.is_some() {
            46
        } else {
            0
        }
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let Some(playing) = (unsafe { media_playing() }) else {
            return;
        };
        unsafe {
            icon_text(hdc, "\u{E8D6}", *right - 38, 10, 0xf472b6, 13);
            icon_text(
                hdc,
                if playing { "\u{E768}" } else { "\u{E769}" },
                *right - 20,
                10,
                0xf472b6,
                15,
            );
        }
        *right -= 46;
    }
}
impl Applet for BatteryApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_battery
    }
    fn width(&self, _c: Config) -> i32 {
        64 + 8
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let mut status = SYSTEM_POWER_STATUS::default();
        let _ = unsafe { GetSystemPowerStatus(&mut status) };
        let (icon, col) = if status.ACLineStatus == 1 {
            ("\u{E945}", 0x4ade80)
        } else if status.BatteryLifePercent <= 20 {
            ("\u{E850}", 0xf87171)
        } else if status.BatteryLifePercent <= 50 {
            ("\u{E854}", 0xfbbf24)
        } else {
            ("\u{E859}", 0x4ade80)
        };
        let label = format!("{}%", status.BatteryLifePercent);
        let footprint = 72;
        let content = unsafe { 20 + measure_text_with_face(hdc, &label, 13, "Segoe UI") };
        let x = *right - footprint + (footprint - content) / 2;
        unsafe {
            icon_text(hdc, icon, x, 10, col, 15);
            text(hdc, &label, x + 20, 10, col, 13);
        }
        *right -= footprint;
    }
}
impl Applet for NotificationApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_notifications
    }
    fn width(&self, _c: Config) -> i32 {
        38
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let mut state = QUNS_ACCEPTS_NOTIFICATIONS;
        let _ = unsafe { SHQueryUserNotificationState().map(|value| state = value) };
        let quiet = state == QUNS_QUIET_TIME;
        unsafe {
            icon_text(
                hdc,
                if quiet { "\u{E7E7}" } else { "\u{E7F4}" },
                *right - 28,
                10,
                if quiet { 0xfbbf24 } else { 0xc4b5fd },
                16,
            );
        }
        *right -= 38;
    }
}

pub(crate) unsafe fn tray_total_width() -> i32 {
    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    let count = TRAY_ICONS.len().min(4) as i32;
    if count == 0 { 28 } else { count * 24 + 28 }
}

pub(crate) unsafe fn tray_icon_at_x(hwnd: HWND, x: i32) -> Option<usize> {
    let mut client = RECT::default();
    if GetClientRect(hwnd, &mut client).is_err() {
        return None;
    }
    let tray_right = client.right - 78;
    let total_w = tray_total_width();
    let tray_left = tray_right - total_w;
    if x < tray_left || x >= tray_right {
        return None;
    }
    if x < tray_left + 28 {
        return None;
    }
    let rel_x = x - (tray_left + 28);
    let index = (rel_x / 24) as usize;
    if index < TRAY_ICONS.len().min(4) {
        Some(index)
    } else {
        None
    }
}

pub(crate) unsafe fn is_tray_chevron_at_x(hwnd: HWND, x: i32) -> bool {
    let mut client = RECT::default();
    if GetClientRect(hwnd, &mut client).is_err() {
        return false;
    }
    let tray_right = client.right - 78;
    let total_w = tray_total_width();
    let tray_left = tray_right - total_w;
    x >= tray_left && x < tray_left + 28
}

#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct COPYDATASTRUCT {
    dwData: usize,
    cbData: u32,
    lpData: *mut std::ffi::c_void,
}

pub(crate) unsafe fn broadcast_taskbar_created() {
    let taskbar_created = RegisterWindowMessageW(PCWSTR(wide("TaskbarCreated").as_ptr()));
    let _ = ChangeWindowMessageFilter(taskbar_created, CHANGE_WINDOW_MESSAGE_FILTER_FLAGS(1));
    let _ = SendNotifyMessageW(HWND_BROADCAST, taskbar_created, WPARAM(0), LPARAM(0));
    let _ = PostMessageW(HWND_BROADCAST, taskbar_created, WPARAM(0), LPARAM(0));
}

pub(crate) unsafe fn paint_panel(hwnd: HWND, screen_dc: HDC) {
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
    let accessible = high_contrast();
    let bg = brush(if accessible {
        0x000000
    } else if c.dark {
        0x13171f
    } else {
        0xf1f5f9
    });
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);

    let border = brush(if accessible {
        0x404040
    } else if c.dark {
        0x242b38
    } else {
        0xe2e8f0
    });
    let border_r = RECT {
        left: 0,
        top: r.bottom - 1,
        right: r.right,
        bottom: r.bottom,
    };
    FillRect(hdc, &border_r, border);
    let _ = DeleteObject(border);

    let pill_bg = brush(if accessible {
        0x1a1a1a
    } else if c.dark {
        0x1e2430
    } else {
        0xe2e8f0
    });
    let luma_pill = RECT {
        left: 10,
        top: 5,
        right: 74,
        bottom: 33,
    };
    fill_round_rect(hdc, &luma_pill, pill_bg, 14);
    let _ = DeleteObject(pill_bg);

    let luma_border_col = if accessible {
        0x404040
    } else if c.dark {
        0x2d3748
    } else {
        0xcbd5e1
    };
    stroke_round_rect(hdc, &luma_pill, luma_border_col, 1, 14);

    icon_text(
        hdc,
        "\u{E707}",
        18,
        11,
        if accessible { 0xffffff } else { 0x38bdf8 },
        16,
    );
    text(
        hdc,
        "Luma",
        39,
        11,
        if accessible {
            0xffffff
        } else if c.dark {
            0xf1f5f9
        } else {
            0x0f172a
        },
        13,
    );

    let act_pill = RECT {
        left: 80,
        top: 5,
        right: 172,
        bottom: 33,
    };
    let act_bg = brush(if accessible {
        0x1a1a1a
    } else if c.dark {
        0x1a202a
    } else {
        0xe8edf5
    });
    fill_round_rect(hdc, &act_pill, act_bg, 14);
    let _ = DeleteObject(act_bg);
    stroke_round_rect(hdc, &act_pill, luma_border_col, 1, 14);

    icon_text(
        hdc,
        "\u{E7C4}",
        88,
        11,
        if accessible { 0xffffff } else { 0x94a3b8 },
        14,
    );
    text(
        hdc,
        "Activities",
        107,
        11,
        if accessible {
            0xffffff
        } else if c.dark {
            0x94a3b8
        } else {
            0x475569
        },
        13,
    );

    let pwr_pill = RECT {
        left: r.right - 36,
        top: 5,
        right: r.right - 8,
        bottom: 33,
    };
    let pwr_bg = brush(if accessible {
        0x1a1a1a
    } else if c.dark {
        0x241d24
    } else {
        0xfee2e2
    });
    fill_round_rect(hdc, &pwr_pill, pwr_bg, 14);
    let _ = DeleteObject(pwr_bg);
    icon_text(
        hdc,
        "\u{E7E8}",
        r.right - 29,
        11,
        if accessible { 0xffffff } else { 0xf87171 },
        14,
    );

    let set_pill = RECT {
        left: r.right - 70,
        top: 5,
        right: r.right - 42,
        bottom: 33,
    };
    let set_bg = brush(if accessible {
        0x1a1a1a
    } else if c.dark {
        0x1e2430
    } else {
        0xe2e8f0
    });
    fill_round_rect(hdc, &set_pill, set_bg, 14);
    let _ = DeleteObject(set_bg);
    icon_text(
        hdc,
        "\u{E713}",
        r.right - 63,
        11,
        if accessible { 0xffffff } else { 0x94a3b8 },
        14,
    );

    // System tray pill & icons on the right (before Settings & Power buttons)
    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    let total_tray_w = tray_total_width();
    let tray_right = r.right - 78;
    let tray_left = tray_right - total_tray_w;

    let tray_pill = RECT {
        left: tray_left,
        top: 5,
        right: tray_right,
        bottom: 33,
    };
    let tray_bg = brush(if accessible {
        0x1a1a1a
    } else if c.dark {
        0x1a202a
    } else {
        0xe8edf5
    });
    fill_round_rect(hdc, &tray_pill, tray_bg, 14);
    let _ = DeleteObject(tray_bg);
    stroke_round_rect(hdc, &tray_pill, luma_border_col, 1, 14);

    // Chevron icon '^' at tray_left + 8
    icon_text(
        hdc,
        "\u{E70E}",
        tray_left + 8,
        11,
        if accessible { 0xffffff } else { 0x94a3b8 },
        12,
    );

    // Render active tray icons (up to 4 on the panel, rest in popup)
    for (idx, item) in TRAY_ICONS.iter().take(4).enumerate() {
        let ix = tray_left + 28 + (idx as i32 * 24) + 3;
        let iy = 10;
        if !item.icon.is_invalid() {
            let _ = DrawIconEx(hdc, ix, iy, item.icon, 18, 18, 0, None, DI_NORMAL);
        }
    }

    let applets: [&dyn Applet; 7] = [
        &ClockApplet as &dyn Applet,
        &NetworkApplet,
        &VolumeApplet,
        &BluetoothApplet,
        &BatteryApplet,
        &NotificationApplet,
        &MediaApplet,
    ];
    for zone in 0..3u8 {
        let mut right = match zone {
            0 => r.right / 3,
            1 => {
                let mut total = 0;
                for id in c.applet_order {
                    if c.panel_zones[id.min(6) as usize] != 1 {
                        continue;
                    }
                    let applet = applets[id.min(6) as usize];
                    if applet.enabled(c) {
                        total += applet.width(c);
                    }
                }
                (r.right / 2) + (total / 2)
            }
            _ => tray_left - 8,
        };
        for id in c.applet_order {
            if c.panel_zones[id.min(6) as usize] != zone {
                continue;
            }
            let applet = applets[id.min(6) as usize];
            if applet.enabled(c) {
                applet.render(hdc, &mut right);
            }
        }
    }
    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}
pub(crate) unsafe fn panel_applet_at(hwnd: HWND, x: i32) -> Option<u8> {
    let mut client = RECT::default();
    GetClientRect(hwnd, &mut client).ok()?;
    let c = CONFIG;
    let applets: [&dyn Applet; 7] = [
        &ClockApplet as &dyn Applet,
        &NetworkApplet,
        &VolumeApplet,
        &BluetoothApplet,
        &BatteryApplet,
        &NotificationApplet,
        &MediaApplet,
    ];
    for zone in 0..3u8 {
        let mut right = match zone {
            0 => client.right / 3,
            1 => {
                let mut total = 0;
                for id in c.applet_order {
                    if c.panel_zones[id.min(6) as usize] != 1 {
                        continue;
                    }
                    let applet = applets[id.min(6) as usize];
                    let enabled = match id {
                        0 => c.show_clock,
                        1 => c.show_network,
                        2 => c.show_volume,
                        3 => c.show_bluetooth,
                        4 => c.show_battery,
                        5 => c.show_notifications,
                        _ => c.show_media,
                    };
                    if enabled {
                        total += applet.width(c);
                    }
                }
                (client.right / 2) + (total / 2)
            }
            _ => client.right - 78 - tray_total_width() - 8,
        };
        for id in c.applet_order {
            if c.panel_zones[id.min(6) as usize] != zone {
                continue;
            }
            let enabled = match id {
                0 => c.show_clock,
                1 => c.show_network,
                2 => c.show_volume,
                3 => c.show_bluetooth,
                4 => c.show_battery,
                5 => c.show_notifications,
                _ => c.show_media,
            };
            if !enabled {
                continue;
            }
            let width = applets[id.min(6) as usize].width(c);
            let left = right - width;
            if x >= left && x < right {
                return Some(id);
            }
            right = left;
        }
    }
    None
}
