use super::*;

pub(crate) extern "system" fn settings_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                SETTINGS = hwnd;
                SETTINGS_INITIALIZED = false;
                create_control(
                    "BUTTON",
                    "General",
                    button_style(BS_PUSHBUTTON),
                    36,
                    88,
                    100,
                    28,
                    ID_PAGE_GENERAL,
                );
                create_control(
                    "BUTTON",
                    "Dock",
                    button_style(BS_PUSHBUTTON),
                    140,
                    88,
                    100,
                    28,
                    ID_PAGE_DOCK,
                );
                create_control(
                    "BUTTON",
                    "Applets",
                    button_style(BS_PUSHBUTTON),
                    244,
                    88,
                    100,
                    28,
                    ID_PAGE_APPLETS,
                );
                create_control(
                    "BUTTON",
                    "Recovery",
                    button_style(BS_PUSHBUTTON),
                    348,
                    88,
                    100,
                    28,
                    ID_PAGE_RECOVERY,
                );
                create_control(
                    "EDIT",
                    "clock,network,volume,bluetooth,battery,notifications,media",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    520,
                    205,
                    190,
                    30,
                    ID_ORDER,
                );
                create_control(
                    "EDIT",
                    "home,files,terminal,browser,settings",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    520,
                    235,
                    190,
                    30,
                    ID_DOCK_ORDER,
                );
                create_control(
                    "EDIT",
                    "45",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    210,
                    360,
                    150,
                    30,
                    ID_MAGNIFICATION,
                );
                create_control(
                    "EDIT",
                    "238",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    390,
                    360,
                    150,
                    30,
                    ID_TRANSPARENCY,
                );
                create_control(
                    "EDIT",
                    "84",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    210,
                    400,
                    150,
                    30,
                    ID_SPACING,
                );
                create_control(
                    "EDIT",
                    "0,0",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    390,
                    400,
                    150,
                    30,
                    ID_OFFSETS,
                );
                create_control(
                    "EDIT",
                    "clock:right,network:right,volume:right,bluetooth:right,battery:right,notifications:right,media:right",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    520,
                    265,
                    190,
                    30,
                    ID_ZONES,
                );
                create_control(
                    "EDIT",
                    "short",
                    WINDOW_STYLE(
                        WS_CHILD.0
                            | WS_VISIBLE.0
                            | WS_TABSTOP.0
                            | WS_BORDER.0
                            | ES_AUTOHSCROLL as u32,
                    ),
                    270,
                    300,
                    190,
                    30,
                    ID_DATE_FORMAT,
                );
                create_control(
                    "BUTTON",
                    "Dark theme",
                    button_style(BS_AUTOCHECKBOX),
                    36,
                    145,
                    220,
                    28,
                    ID_DARK,
                );
                create_control(
                    "BUTTON",
                    "Desktop wallpaper & context menu",
                    button_style(BS_AUTOCHECKBOX),
                    36,
                    185,
                    260,
                    28,
                    ID_DESKTOP_TOGGLE,
                );
                create_control(
                    "BUTTON",
                    "Left",
                    button_style(BS_AUTORADIOBUTTON),
                    420,
                    145,
                    90,
                    28,
                    ID_LEFT,
                );
                create_control(
                    "BUTTON",
                    "Center",
                    button_style(BS_AUTORADIOBUTTON),
                    515,
                    145,
                    100,
                    28,
                    ID_CENTER,
                );
                create_control(
                    "BUTTON",
                    "Right",
                    button_style(BS_AUTORADIOBUTTON),
                    620,
                    145,
                    90,
                    28,
                    ID_RIGHT,
                );
                create_control(
                    "BUTTON",
                    "Clock",
                    button_style(BS_AUTOCHECKBOX),
                    36,
                    340,
                    220,
                    28,
                    ID_CLOCK_CHECK,
                );
                create_control(
                    "BUTTON",
                    "Network",
                    button_style(BS_AUTOCHECKBOX),
                    36,
                    374,
                    220,
                    28,
                    ID_NETWORK_CHECK,
                );
                create_control(
                    "BUTTON",
                    "Volume",
                    button_style(BS_AUTOCHECKBOX),
                    36,
                    408,
                    220,
                    28,
                    ID_VOLUME_CHECK,
                );
                create_control(
                    "BUTTON",
                    "Battery",
                    button_style(BS_AUTOCHECKBOX),
                    270,
                    340,
                    220,
                    28,
                    ID_BATTERY_CHECK,
                );
                create_control(
                    "BUTTON",
                    "Bluetooth",
                    button_style(BS_AUTOCHECKBOX),
                    270,
                    374,
                    220,
                    28,
                    ID_BLUETOOTH,
                );
                create_control(
                    "BUTTON",
                    "Notifications",
                    button_style(BS_AUTOCHECKBOX),
                    270,
                    408,
                    220,
                    28,
                    ID_NOTIFICATIONS,
                );
                create_control(
                    "BUTTON",
                    "Media",
                    button_style(BS_AUTOCHECKBOX),
                    270,
                    442,
                    220,
                    28,
                    ID_MEDIA,
                );
                create_control(
                    "BUTTON",
                    "Set Luma as Windows Shell",
                    button_style(BS_PUSHBUTTON),
                    210,
                    170,
                    230,
                    34,
                    ID_SET_LUMA_SHELL,
                );
                create_control(
                    "BUTTON",
                    "Start with Windows",
                    button_style(BS_AUTOCHECKBOX),
                    520,
                    250,
                    180,
                    28,
                    ID_STARTUP,
                );
                create_control(
                    "BUTTON",
                    "Open diagnostics",
                    button_style(BS_PUSHBUTTON),
                    520,
                    300,
                    180,
                    34,
                    ID_OPEN_DIAGNOSTICS,
                );
                create_control(
                    "BUTTON",
                    "Open watchdog log",
                    button_style(BS_PUSHBUTTON),
                    430,
                    300,
                    210,
                    34,
                    ID_OPEN_WATCHDOG,
                );
                create_control(
                    "BUTTON",
                    "Reset to defaults",
                    button_style(BS_PUSHBUTTON),
                    520,
                    400,
                    180,
                    34,
                    ID_RESET,
                );
                create_control(
                    "BUTTON",
                    "Restore Explorer",
                    button_style(BS_PUSHBUTTON),
                    520,
                    350,
                    180,
                    34,
                    ID_RESTORE_EXPLORER,
                );
                create_control(
                    "BUTTON",
                    "Restore settings backup",
                    button_style(BS_PUSHBUTTON),
                    430,
                    350,
                    210,
                    34,
                    ID_RESTORE_SETTINGS,
                );
                create_control(
                    "BUTTON",
                    "Auto-hide dock",
                    button_style(BS_AUTOCHECKBOX),
                    520,
                    270,
                    180,
                    28,
                    ID_AUTO_HIDE,
                );
                create_control(
                    "BUTTON",
                    "24-hour clock",
                    button_style(BS_AUTOCHECKBOX),
                    270,
                    270,
                    180,
                    28,
                    ID_CLOCK_24H,
                );
                create_control(
                    "BUTTON",
                    "Small",
                    button_style(BS_AUTORADIOBUTTON),
                    520,
                    330,
                    70,
                    28,
                    ID_ICON_SMALL,
                );
                create_control(
                    "BUTTON",
                    "Medium",
                    button_style(BS_AUTORADIOBUTTON),
                    595,
                    330,
                    85,
                    28,
                    ID_ICON_MEDIUM,
                );
                create_control(
                    "BUTTON",
                    "Large",
                    button_style(BS_AUTORADIOBUTTON),
                    520,
                    365,
                    70,
                    28,
                    ID_ICON_LARGE,
                );
                sync_settings_edit_fields();
                sync_settings_controls();
                SETTINGS_INITIALIZED = true;
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_settings(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_COMMAND => {
                if !SETTINGS_INITIALIZED || SYNCING_SETTINGS {
                    return LRESULT(0);
                }
                let code = ((w.0 >> 16) & 0xffff) as u16;
                let id = w.0 & 0xffff;
                if (ID_PAGE_GENERAL..=ID_PAGE_RECOVERY).contains(&id) {
                    SETTINGS_PAGE = id - ID_PAGE_GENERAL;
                    sync_settings_page();
                    return LRESULT(0);
                }
                let is_edit = matches!(
                    id,
                    ID_MAGNIFICATION
                        | ID_TRANSPARENCY
                        | ID_SPACING
                        | ID_OFFSETS
                        | ID_ORDER
                        | ID_DOCK_ORDER
                        | ID_ZONES
                        | ID_DATE_FORMAT
                );
                // For edit controls, only process if text actually changed (EN_CHANGE = 0x0300)
                if is_edit && code != 0x0300 {
                    return LRESULT(0);
                }
                match id {
                    ID_DARK => CONFIG.dark = !CONFIG.dark,
                    ID_DESKTOP_TOGGLE => {
                        CONFIG.desktop_enabled = !CONFIG.desktop_enabled;
                        if !DESKTOP.is_invalid() {
                            ShowWindow(
                                DESKTOP,
                                if CONFIG.desktop_enabled {
                                    SW_SHOW
                                } else {
                                    SW_HIDE
                                },
                            );
                        }
                    }
                    ID_SET_LUMA_SHELL => {
                        if is_luma_shell_active() {
                            if restore_explorer_shell() {
                                log_event("restored explorer shell via settings GUI");
                                std::process::exit(0);
                            }
                        } else if enable_luma_shell_replacement() {
                            log_event("shell replacement registered via settings GUI");
                            // Also kill explorer now so the replacement takes effect immediately
                            if is_explorer_running() {
                                kill_explorer();
                                // Show the desktop since explorer is now gone
                                if CONFIG.desktop_enabled && !DESKTOP.is_invalid() {
                                    ShowWindow(DESKTOP, SW_SHOW);
                                }
                            }
                        }
                    }
                    ID_CLOCK_CHECK => CONFIG.show_clock = !CONFIG.show_clock,
                    ID_NETWORK_CHECK => CONFIG.show_network = !CONFIG.show_network,
                    ID_VOLUME_CHECK => CONFIG.show_volume = !CONFIG.show_volume,
                    ID_BATTERY_CHECK => CONFIG.show_battery = !CONFIG.show_battery,
                    ID_NOTIFICATIONS => CONFIG.show_notifications = !CONFIG.show_notifications,
                    ID_BLUETOOTH => CONFIG.show_bluetooth = !CONFIG.show_bluetooth,
                    ID_MEDIA => CONFIG.show_media = !CONFIG.show_media,
                    ID_LEFT => CONFIG.position = DockPosition::Left,
                    ID_CENTER => CONFIG.position = DockPosition::Center,
                    ID_RIGHT => CONFIG.position = DockPosition::Right,
                    ID_RESET => {
                        CONFIG = default_config();
                        set_docks_visible(true);
                        sync_settings_edit_fields();
                    }
                    ID_RESTORE_EXPLORER => restore_explorer(),
                    ID_RESTORE_SETTINGS => {
                        if restore_config_backup() {
                            CONFIG = load_config();
                            sync_settings_edit_fields();
                        }
                    }
                    ID_OPEN_DIAGNOSTICS => {
                        log_event("diagnostics opened from settings");
                        let path = log_path().to_string_lossy().into_owned();
                        launch(&path);
                    }
                    ID_OPEN_WATCHDOG => {
                        log_event("watchdog diagnostics opened from settings");
                        let log = watchdog_log_path();
                        if let Some(parent) = log.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if !log.exists() {
                            let _ = fs::write(&log, "Watchdog has not run yet.\n");
                        }
                        launch(&log.to_string_lossy());
                    }
                    ID_STARTUP => {
                        let enabled = startup_enabled();
                        if !set_startup_enabled(!enabled) {
                            log_event("startup registration update failed");
                        }
                    }
                    ID_AUTO_HIDE => {
                        CONFIG.auto_hide = !CONFIG.auto_hide;
                        if !CONFIG.auto_hide {
                            set_docks_visible(true);
                        }
                    }
                    ID_ICON_SMALL => CONFIG.icon_size = 36,
                    ID_ICON_MEDIUM => CONFIG.icon_size = 44,
                    ID_ICON_LARGE => CONFIG.icon_size = 52,
                    ID_CLOCK_24H => CONFIG.clock_24h = !CONFIG.clock_24h,
                    ID_MAGNIFICATION => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_MAGNIFICATION as i32) {
                            CONFIG.magnification = read_control_text(h)
                                .trim()
                                .parse::<i32>()
                                .unwrap_or(CONFIG.magnification as i32)
                                .clamp(0, 100)
                                as u8;
                        }
                    }
                    ID_TRANSPARENCY => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_TRANSPARENCY as i32) {
                            CONFIG.transparency = read_control_text(h)
                                .trim()
                                .parse::<i32>()
                                .unwrap_or(CONFIG.transparency as i32)
                                .clamp(120, 255)
                                as u8;
                        }
                    }
                    ID_SPACING => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_SPACING as i32) {
                            CONFIG.spacing = read_control_text(h)
                                .trim()
                                .parse::<i32>()
                                .unwrap_or(CONFIG.spacing)
                                .clamp(72, 96);
                        }
                    }
                    ID_OFFSETS => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_OFFSETS as i32) {
                            let values = read_control_text(h);
                            let mut parts = values.split(',').map(str::trim);
                            CONFIG.offset_x = parts
                                .next()
                                .and_then(|value| value.parse::<i32>().ok())
                                .unwrap_or(CONFIG.offset_x)
                                .clamp(-200, 200);
                            CONFIG.offset_y = parts
                                .next()
                                .and_then(|value| value.parse::<i32>().ok())
                                .unwrap_or(CONFIG.offset_y)
                                .clamp(-100, 100);
                        }
                    }
                    ID_ORDER => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_ORDER as i32) {
                            CONFIG.applet_order = parse_order(&read_control_text(h));
                        }
                    }
                    ID_DOCK_ORDER => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_DOCK_ORDER as i32) {
                            CONFIG.dock_order = parse_dock_order(&read_control_text(h));
                        }
                    }
                    ID_ZONES => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_ZONES as i32) {
                            CONFIG.panel_zones = parse_zones(&read_control_text(h));
                        }
                    }
                    ID_DATE_FORMAT => {
                        if let Ok(h) = GetDlgItem(SETTINGS, ID_DATE_FORMAT as i32) {
                            CONFIG.date_format = match read_control_text(h).trim() {
                                "none" => 0,
                                "long" => 2,
                                _ => 1,
                            };
                        }
                    }
                    _ => {}
                }
                save_config();
                if !is_edit {
                    sync_settings_controls();
                    let _ = InvalidateRect(hwnd, None, false);
                }
                restyle_shell_windows();
                position_dock();
                let _ = InvalidateRect(PANEL, None, false);
                let _ = InvalidateRect(DOCK, None, false);
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
