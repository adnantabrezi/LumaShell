use super::*;

pub(crate) struct ClockApplet;
pub(crate) struct NetworkApplet;
pub(crate) struct VolumeApplet;
pub(crate) struct BluetoothApplet;
pub(crate) struct MediaApplet;
pub(crate) struct BatteryApplet;
pub(crate) struct NotificationApplet;
pub(crate) fn format_date(n: SYSTEMTIME, format: u8) -> String {
    let short_months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
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
        _ => format!(
            "{} {}",
            n.wDay,
            short_months[n.wMonth.saturating_sub(1).min(11) as usize]
        ),
    }
}

fn format_cosmic_clock(n: SYSTEMTIME, clock_24h: bool, date_format: u8) -> String {
    let time = if clock_24h {
        format!("{:02}:{:02}", n.wHour, n.wMinute)
    } else {
        let hour = if n.wHour == 0 {
            12
        } else if n.wHour > 12 {
            n.wHour - 12
        } else {
            n.wHour
        };
        let suffix = if n.wHour >= 12 { "pm" } else { "am" };
        format!("{}:{:02} {}", hour, n.wMinute, suffix)
    };
    let date = format_date(n, date_format);
    if date.is_empty() {
        time
    } else if date_format == 2 {
        format!("{} · {}", date, time)
    } else {
        format!("{}, {}", date, time)
    }
}

impl Applet for ClockApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_clock
    }
    fn width(&self, c: Config) -> i32 {
        let n: SYSTEMTIME = unsafe { GetLocalTime() };
        let value = format_cosmic_clock(n, c.clock_24h, c.date_format);
        (value.chars().count() as i32 * 8 + 16).max(96)
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let n: SYSTEMTIME = unsafe { GetLocalTime() };
        let value = format_cosmic_clock(n, c.clock_24h, c.date_format);
        let width = (value.chars().count() as i32 * 8 + 16).max(96);
        // Center ink precisely inside the allotted zone so center zone stays dead-center on monitor
        let content = unsafe { measure_text_with_face(hdc, &value, 13, "Segoe UI Variable Text") };
        let x = *right - width + (width - content) / 2;
        let col = if c.dark || c.panel_transparent {
            0xf8fafc
        } else {
            0x0f172a
        };
        unsafe {
            text(hdc, &value, x, 10, col, 13);
        }
        *right -= width;
    }
}

impl Applet for NetworkApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_network
    }
    fn width(&self, _c: Config) -> i32 {
        28
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
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
        let x = *right - 28 + (28 - 14) / 2;
        let col = if !connected {
            0xfb7185
        } else if c.panel_transparent {
            0xf8fafc
        } else {
            c.accent_color
        };
        unsafe {
            icon_text(hdc, icon, x, 11, col, 14);
        }
        *right -= 28;
    }
}

impl Applet for VolumeApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_volume
    }
    fn width(&self, c: Config) -> i32 {
        if c.panel_transparent {
            return 28;
        }
        let (level, _) = unsafe { default_audio_state().unwrap_or((0, false)) };
        let label = format!("{}%", level);
        label.len() as i32 * 8 + 26
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let (level, muted) = unsafe { default_audio_state().unwrap_or((0, false)) };
        let icon = if muted {
            "\u{E74F}"
        } else if level < 33 {
            "\u{E993}"
        } else if level < 66 {
            "\u{E994}"
        } else {
            "\u{E767}"
        };
        let icon_col = if muted {
            0xfb7185
        } else if c.panel_transparent {
            0xf8fafc
        } else {
            c.accent_color
        };
        if c.panel_transparent {
            let x = *right - 28 + (28 - 14) / 2;
            unsafe {
                icon_text(hdc, icon, x, 11, icon_col, 14);
            }
            *right -= 28;
            return;
        }
        let label = format!("{}%", level);
        let footprint = label.len() as i32 * 8 + 26;
        let content = unsafe { 18 + measure_text_with_face(hdc, &label, 12, "Segoe UI Variable Text") };
        let x = *right - footprint + (footprint - content) / 2;
        unsafe {
            icon_text(hdc, icon, x, 11, icon_col, 14);
            text(hdc, &label, x + 18, 11, if c.dark { 0xe2e8f0 } else { 0x1e293b }, 12);
        }
        *right -= footprint;
    }
}

impl Applet for BluetoothApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_bluetooth
    }
    fn width(&self, _c: Config) -> i32 {
        26
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let (present, _any_connected) = unsafe { bluetooth_status_snapshot() };
        let icon = if !present { "\u{E711}" } else { "\u{E702}" };
        let color = if !present {
            0x94a3b8
        } else if c.panel_transparent {
            0xf8fafc
        } else {
            c.accent_color
        };
        let x = *right - 26 + (26 - 14) / 2;
        unsafe {
            icon_text(hdc, icon, x, 11, color, 14);
        }
        *right -= 26;
    }
}

impl Applet for MediaApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_media
    }
    fn width(&self, c: Config) -> i32 {
        if c.panel_transparent {
            28
        } else if unsafe { media_playing() }.is_some() {
            42
        } else {
            28
        }
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let col = if c.panel_transparent {
            0xf1f5f9
        } else {
            c.accent_color
        };
        let w = self.width(c);
        unsafe {
            icon_text(hdc, "\u{E8D6}", *right - w + 7, 11, col, 14);
        }
        *right -= w;
    }
}

impl Applet for BatteryApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_battery
    }
    fn width(&self, c: Config) -> i32 {
        if c.panel_transparent { 28 } else { 62 }
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let mut status = SYSTEM_POWER_STATUS::default();
        let _ = unsafe { GetSystemPowerStatus(&mut status) };
        let (icon, col) = if c.panel_transparent {
            (
                if status.ACLineStatus == 1 { "\u{E945}" } else { "\u{E859}" },
                0xf8fafc,
            )
        } else if status.ACLineStatus == 1 {
            ("\u{E945}", 0x4ade80)
        } else if status.BatteryLifePercent <= 20 {
            ("\u{E850}", 0xf87171)
        } else if status.BatteryLifePercent <= 50 {
            ("\u{E854}", 0xfbbf24)
        } else {
            ("\u{E859}", 0x4ade80)
        };
        if c.panel_transparent {
            let x = *right - 28 + (28 - 14) / 2;
            unsafe {
                icon_text(hdc, icon, x, 11, col, 14);
            }
            *right -= 28;
            return;
        }
        let label = format!("{}%", status.BatteryLifePercent);
        let footprint = 62;
        let content = unsafe { 18 + measure_text_with_face(hdc, &label, 12, "Segoe UI Variable Text") };
        let x = *right - footprint + (footprint - content) / 2;
        unsafe {
            icon_text(hdc, icon, x, 11, col, 14);
            text(hdc, &label, x + 18, 11, col, 12);
        }
        *right -= footprint;
    }
}

impl Applet for NotificationApplet {
    fn enabled(&self, c: Config) -> bool {
        c.show_notifications
    }
    fn width(&self, _c: Config) -> i32 {
        28
    }
    fn render(&self, hdc: HDC, right: &mut i32) {
        let c = unsafe { CONFIG };
        let mut state = QUNS_ACCEPTS_NOTIFICATIONS;
        let _ = unsafe { SHQueryUserNotificationState().map(|value| state = value) };
        let quiet = state == QUNS_QUIET_TIME;
        let col = if quiet {
            0xfbbf24
        } else if c.panel_transparent {
            0xf8fafc
        } else {
            c.accent_color
        };
        unsafe {
            icon_text(
                hdc,
                if quiet { "\u{E7E7}" } else { "\u{E7F4}" },
                *right - 21,
                11,
                col,
                14,
            );
        }
        *right -= 28;
    }
}

pub(crate) unsafe fn right_applets_total_width(c: Config) -> i32 {
    let applets: [&dyn Applet; 7] = [
        &ClockApplet as &dyn Applet,
        &NetworkApplet,
        &VolumeApplet,
        &BluetoothApplet,
        &BatteryApplet,
        &NotificationApplet,
        &MediaApplet,
    ];
    let mut total = 0;
    for id in c.applet_order {
        if c.panel_zones[id.min(6) as usize] != 2 {
            continue;
        }
        let applet = applets[id.min(6) as usize];
        if applet.enabled(c) {
            total += applet.width(c);
        }
    }
    total
}

pub(crate) unsafe fn tray_total_width() -> i32 {
    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    let count = TRAY_ICONS.len().min(6) as i32;
    22 + count * 22
}

/// Returns `(tray_left, tray_right)` positioned directly to the left of the right-zone status applets
/// (immediately right of the `US` keyboard badge), matching the COSMIC top panel layout.
pub(crate) unsafe fn panel_tray_bounds(hwnd: HWND) -> Option<(i32, i32)> {
    let mut client = RECT::default();
    if GetClientRect(hwnd, &mut client).is_err() {
        return None;
    }
    let c = CONFIG;
    let right_applets_w = right_applets_total_width(c);
    let tray_right = client.right - 68 - right_applets_w - 6;
    let total_w = tray_total_width();
    let tray_left = tray_right - total_w;
    Some((tray_left, tray_right))
}

pub(crate) unsafe fn tray_icon_at_x(hwnd: HWND, x: i32) -> Option<usize> {
    let (tray_left, tray_right) = panel_tray_bounds(hwnd)?;
    if x < tray_left + 22 || x >= tray_right {
        return None;
    }
    let rel_x = x - (tray_left + 22);
    let index = (rel_x / 22) as usize;
    if index < TRAY_ICONS.len().min(6) {
        Some(index)
    } else {
        None
    }
}

pub(crate) unsafe fn is_tray_chevron_at_x(hwnd: HWND, x: i32) -> bool {
    let Some((tray_left, _)) = panel_tray_bounds(hwnd) else {
        return false;
    };
    x >= tray_left && x < tray_left + 22
}

pub(crate) unsafe fn is_tiling_button_at_x(hwnd: HWND, x: i32) -> bool {
    let Some((tray_left, _)) = panel_tray_bounds(hwnd) else {
        return false;
    };
    let tile_right = tray_left - 30;
    let tile_left = tile_right - 26;
    x >= tile_left && x < tile_right
}

#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct COPYDATASTRUCT {
    pub(crate) dwData: usize,
    pub(crate) cbData: u32,
    pub(crate) lpData: *mut std::ffi::c_void,
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
    // Match web COSMIC Orbital Earth top strip: pure obsidian space (#000000) in transparent mode
    let bg = brush(if accessible {
        0x000000
    } else if c.panel_transparent && c.dark {
        0x000000
    } else if c.dark {
        0x0f1219
    } else {
        0xf1f5f9
    });
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);

    if !c.panel_transparent {
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
    }

    let fg_col = if accessible || c.dark || c.panel_transparent {
        0xf8fafc
    } else {
        0x0f172a
    };
    let muted_col = if accessible {
        0xffffff
    } else if c.dark || c.panel_transparent {
        0xcbd5e1
    } else {
        0x475569
    };

    // ZONE 1 (Left): Clean COSMIC Workspaces "1  2  3  (4)" + Clipboard/Launcher icon
    let active_ws = 4usize;
    let mut left_cursor = 14;
    for ws in 1..=4usize {
        let ws_rect = RECT {
            left: left_cursor,
            top: 7,
            right: left_cursor + 24,
            bottom: 31,
        };
        if ws == active_ws {
            let ws_bg = brush(if accessible {
                0x333333
            } else if c.panel_transparent {
                0x3f434e
            } else {
                c.accent_color
            });
            fill_round_rect(hdc, &ws_rect, ws_bg, 24);
            let _ = DeleteObject(ws_bg);
            if c.panel_transparent && !accessible {
                stroke_round_rect(hdc, &ws_rect, 0x6b7280, 1, 24);
            }
        }
        let label = ws.to_string();
        let ws_text_col = if ws == active_ws && !c.panel_transparent && !accessible {
            0x020617
        } else {
            fg_col
        };
        text(hdc, &label, left_cursor + 8, 11, ws_text_col, 12);
        left_cursor += 26;
    }

    // Launcher / Clipboard Overview glyph right next to workspaces (x = 122..148)
    icon_text(hdc, "\u{E77F}", left_cursor + 6, 11, muted_col, 14);
    left_cursor += 32;

    // Far-right Settings & Power monochrome icons (matching web top panel)
    icon_text(hdc, "\u{E713}", r.right - 58, 11, fg_col, 14);
    icon_text(hdc, "\u{E7E8}", r.right - 28, 11, fg_col, 14);

    let applets: [&dyn Applet; 7] = [
        &ClockApplet as &dyn Applet,
        &NetworkApplet,
        &VolumeApplet,
        &BluetoothApplet,
        &BatteryApplet,
        &NotificationApplet,
        &MediaApplet,
    ];

    // Render all 3 zones: Left (after launcher), Center (exact mathematical viewport center), Right (before Settings/Power)
    for zone in 0..3u8 {
        let mut right = match zone {
            0 => {
                let mut left_total = 0;
                for id in c.applet_order {
                    if c.panel_zones[id.min(6) as usize] != 0 {
                        continue;
                    }
                    let applet = applets[id.min(6) as usize];
                    if applet.enabled(c) {
                        left_total += applet.width(c);
                    }
                }
                left_cursor + left_total
            }
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
            _ => r.right - 68,
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

    // System Tray Icons + Overflow Chevron positioned immediately left of right-zone applets, right of "US"
    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    let right_applets_w = right_applets_total_width(c);
    let tray_right = r.right - 68 - right_applets_w - 6;
    let total_tray_w = tray_total_width();
    let tray_left = tray_right - total_tray_w;

    // COSMIC Tiling Mode glyph immediately left of "US" (mirrors COSMIC top panel Tiling Applet)
    let tiling_active = tiling::is_tiling_enabled();
    let tile_pill = RECT {
        left: tray_left - 56,
        top: 7,
        right: tray_left - 32,
        bottom: 31,
    };
    if tiling_active {
        let tile_bg = brush(if accessible {
            0x333333
        } else if c.panel_transparent {
            0x273042
        } else if c.dark {
            0x1e293b
        } else {
            0xe2e8f0
        });
        fill_round_rect(hdc, &tile_pill, tile_bg, 8);
        let _ = DeleteObject(tile_bg);
        if !accessible {
            stroke_round_rect(hdc, &tile_pill, c.accent_color, 1, 8);
        }
    }
    let tiling_glyph = if tiling_active { "\u{F0E2}" } else { "\u{E8A9}" };
    let tiling_col = if tiling_active {
        if accessible { fg_col } else { c.accent_color }
    } else {
        muted_col
    };
    icon_text(hdc, tiling_glyph, tray_left - 51, 11, tiling_col, 13);

    // Keyboard layout indicator "US" immediately left of system tray icons
    text(hdc, "US", tray_left - 26, 11, fg_col, 12);

    // Chevron icon '^' to open System Tray overflow popup
    icon_text(hdc, "\u{E70E}", tray_left + 4, 12, muted_col, 11);

    // Render live Windows system tray icons (Shell_NotifyIcon) crisply in the panel
    for (idx, item) in TRAY_ICONS.iter().take(6).enumerate() {
        let ix = tray_left + 22 + (idx as i32 * 22) + 3;
        let iy = 11;
        if !item.icon.is_invalid() {
            let _ = DrawIconEx(hdc, ix, iy, item.icon, 16, 16, 0, None, DI_NORMAL);
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
    let left_cursor = 14 + 4 * 26 + 32;
    for zone in 0..3u8 {
        let mut right = match zone {
            0 => {
                let mut left_total = 0;
                for id in c.applet_order {
                    if c.panel_zones[id.min(6) as usize] != 0 {
                        continue;
                    }
                    let applet = applets[id.min(6) as usize];
                    if applet.enabled(c) {
                        left_total += applet.width(c);
                    }
                }
                left_cursor + left_total
            }
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
                (client.right / 2) + (total / 2)
            }
            _ => client.right - 68,
        };
        for id in c.applet_order {
            if c.panel_zones[id.min(6) as usize] != zone {
                continue;
            }
            let applet = applets[id.min(6) as usize];
            if !applet.enabled(c) {
                continue;
            }
            let width = applet.width(c);
            let left = right - width;
            if x >= left && x < right {
                return Some(id);
            }
            right = left;
        }
    }
    None
}
