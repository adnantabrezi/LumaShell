use super::*;

pub(crate) unsafe fn paint_audio_mixer(hwnd: HWND, screen_dc: HDC) {
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

    let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);
    let primary = if CONFIG.dark { 0xf7f8fa } else { 0x172033 };
    let secondary = if CONFIG.dark { 0x9da6b4 } else { 0x64748b };
    text(hdc, "Audio mixer", 22, 20, primary, 20);
    let (master, muted) = default_audio_state().unwrap_or((0, false));
    text(
        hdc,
        &format!(
            "Master  {}%  {}",
            master,
            if muted { "muted" } else { "active" }
        ),
        22,
        52,
        0x60a5fa,
        14,
    );
    text(
        hdc,
        "Click a row to mute · wheel to adjust",
        22,
        76,
        secondary,
        11,
    );
    let rows = audio_session_rows();
    let selected = AUDIO_MIXER_SELECTION.min(rows.len().min(10).saturating_sub(1));
    for (index, row) in rows.iter().take(10).enumerate() {
        let y = 112 + index as i32 * 30;
        if index == selected {
            let highlight = brush(if CONFIG.dark { 0x303846 } else { 0xe2e8f0 });
            let selected_row = RECT {
                left: 16,
                top: y - 6,
                right: r.right - 16,
                bottom: y + 24,
            };
            fill_round_rect(hdc, &selected_row, highlight, 8);
            let _ = DeleteObject(highlight);
        }
        if row.muted {
            icon_text(hdc, "\u{E74F}", 22, y, 0xf87171, 14);
        } else {
            icon_text(hdc, "\u{E767}", 22, y, 0x38bdf8, 14);
        }
        text(hdc, &row.label, 52, y, primary, 13);
        text(hdc, &format!("{}%", row.volume), 292, y, secondary, 12);
    }
    if rows.is_empty() {
        text(
            hdc,
            "No active application sessions",
            22,
            116,
            secondary,
            13,
        );
    }

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) unsafe fn show_audio_mixer() {
    if AUDIO_MIXER.0.is_null() {
        return;
    }
    if IsWindowVisible(AUDIO_MIXER).as_bool() {
        ShowWindow(AUDIO_MIXER, SW_HIDE);
        return;
    }
    if !NETWORK_POPUP.0.is_null() && IsWindowVisible(NETWORK_POPUP).as_bool() {
        ShowWindow(NETWORK_POPUP, SW_HIDE);
    }
    if !BLUETOOTH_POPUP.0.is_null() && IsWindowVisible(BLUETOOTH_POPUP).as_bool() {
        ShowWindow(BLUETOOTH_POPUP, SW_HIDE);
    }
    if !TRAY_POPUP.0.is_null() && IsWindowVisible(TRAY_POPUP).as_bool() {
        ShowWindow(TRAY_POPUP, SW_HIDE);
    }
    tiling_popup::hide_tiling_popup();
    AUDIO_MIXER_SELECTION = 0;
    let mut cursor = POINT::default();
    GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info);

    let x = (cursor.x - AUDIO_MIXER_WIDTH / 2)
        .max(info.rcWork.left + 8)
        .min(info.rcWork.right - AUDIO_MIXER_WIDTH - 8);
    let mut panel_r = RECT::default();
    let panel_bottom = if !PANEL.0.is_null() && GetWindowRect(PANEL, &mut panel_r).is_ok() {
        panel_r.bottom
    } else {
        info.rcWork.top
    };
    let y = if cursor.y < panel_bottom + 100 {
        panel_bottom + 4
    } else {
        (cursor.y - AUDIO_MIXER_HEIGHT - 8).max(info.rcWork.top + 8)
    };
    let _ = SetWindowPos(
        AUDIO_MIXER,
        HWND_TOPMOST,
        x,
        y,
        AUDIO_MIXER_WIDTH,
        AUDIO_MIXER_HEIGHT,
        SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(AUDIO_MIXER, None, false);
    ShowWindow(AUDIO_MIXER, SW_SHOW);
    SetForegroundWindow(AUDIO_MIXER);
    SetFocus(AUDIO_MIXER);
}

pub(crate) extern "system" fn audio_mixer_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_audio_mixer(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                if (90..112).contains(&y) {
                    if !toggle_audio_mute() {
                        launch("ms-settings:sound");
                    }
                } else if y >= 112 {
                    let index = ((y - 112) / 30) as usize;
                    AUDIO_MIXER_SELECTION = index.min(9);
                    notify_accessibility_focus(hwnd, AUDIO_MIXER_SELECTION as i32);
                    let _ = toggle_audio_session(AUDIO_MIXER_SELECTION);
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let mut point = POINT::default();
                GetCursorPos(&mut point);
                ScreenToClient(hwnd, &mut point);
                if point.y >= 112 {
                    let index = ((point.y - 112) / 30) as usize;
                    let delta = if ((w.0 >> 16) & 0xffff) as i16 > 0 {
                        0.05
                    } else {
                        -0.05
                    };
                    let _ = adjust_audio_session(index, delta);
                    let _ = InvalidateRect(hwnd, None, false);
                }
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let row_count = audio_session_rows().len().min(10);
                let max = row_count.saturating_sub(1);
                match w.0 as u32 {
                    value if value == VK_HOME.0 as u32 => {
                        if !toggle_audio_mute() {
                            launch("ms-settings:sound");
                        }
                    }
                    value if value == VK_UP.0 as u32 => {
                        AUDIO_MIXER_SELECTION = AUDIO_MIXER_SELECTION.saturating_sub(1);
                        notify_accessibility_focus(hwnd, AUDIO_MIXER_SELECTION as i32);
                    }
                    value if value == VK_DOWN.0 as u32 => {
                        AUDIO_MIXER_SELECTION = (AUDIO_MIXER_SELECTION + 1).min(max);
                        notify_accessibility_focus(hwnd, AUDIO_MIXER_SELECTION as i32);
                    }
                    value if value == VK_RETURN.0 as u32 && row_count > 0 => {
                        let _ = toggle_audio_session(AUDIO_MIXER_SELECTION);
                    }
                    value if value == VK_LEFT.0 as u32 && row_count > 0 => {
                        let _ = adjust_audio_session(AUDIO_MIXER_SELECTION, -0.05);
                    }
                    value if value == VK_RIGHT.0 as u32 && row_count > 0 => {
                        let _ = adjust_audio_session(AUDIO_MIXER_SELECTION, 0.05);
                    }
                    _ => return DefWindowProcW(hwnd, msg, w, l),
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_KILLFOCUS | WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

#[derive(Clone)]
pub(crate) struct WifiNetworkItem {
    ssid: String,
    signal: u32,
    connected: bool,
    security: bool,
    profile_exists: bool,
}

#[derive(Clone)]
pub(crate) struct BtDeviceItem {
    name: String,
    connected: bool,
    #[allow(dead_code)]
    remembered: bool,
    #[allow(dead_code)]
    authenticated: bool,
    #[allow(dead_code)]
    info: BLUETOOTH_DEVICE_INFO,
}

pub(crate) unsafe fn bluetooth_status_snapshot() -> (bool, bool) {
    if BLUETOOTH_TICKS < 4 {
        BLUETOOTH_TICKS += 1;
        return BLUETOOTH_STATUS;
    }
    BLUETOOTH_TICKS = 0;
    let present = {
        let params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut radio = HANDLE::default();
        let found = BluetoothFindFirstRadio(&params, &mut radio);
        if let Ok(handle) = found {
            let _ = BluetoothFindRadioClose(handle);
            let _ = CloseHandle(radio);
            true
        } else {
            false
        }
    };
    let mut any_connected = false;
    if present {
        let search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: BOOL(1),
            fReturnRemembered: BOOL(1),
            fReturnUnknown: BOOL(0),
            fReturnConnected: BOOL(1),
            fIssueInquiry: BOOL(0),
            cTimeoutMultiplier: 0,
            hRadio: HANDLE::default(),
        };
        let mut dev_info = BLUETOOTH_DEVICE_INFO {
            dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };
        if let Ok(find_handle) = BluetoothFindFirstDevice(&search_params, &mut dev_info) {
            loop {
                if dev_info.fConnected.as_bool() {
                    any_connected = true;
                    break;
                }
                dev_info = BLUETOOTH_DEVICE_INFO {
                    dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                    ..Default::default()
                };
                if BluetoothFindNextDevice(find_handle, &mut dev_info).is_err() {
                    break;
                }
            }
            let _ = BluetoothFindDeviceClose(find_handle);
        }
    }
    BLUETOOTH_STATUS = (present, any_connected);
    (present, any_connected)
}

pub(crate) unsafe fn refresh_wifi_networks() {
    let mut items: Vec<WifiNetworkItem> = Vec::new();
    let mut negotiated_version = 0u32;
    let mut client = HANDLE::default();
    if WlanOpenHandle(2, None, &mut negotiated_version, &mut client) == 0 {
        let mut if_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        if WlanEnumInterfaces(client, None, &mut if_list) == 0 && !if_list.is_null() {
            let list = &*if_list;
            for i in 0..list.dwNumberOfItems as usize {
                let info = &*list.InterfaceInfo.as_ptr().add(i);

                let mut profile_names = Vec::new();
                let mut prof_list: *mut WLAN_PROFILE_INFO_LIST = null_mut();
                if WlanGetProfileList(client, &info.InterfaceGuid, None, &mut prof_list) == 0
                    && !prof_list.is_null()
                {
                    let profiles = &*prof_list;
                    for p in 0..profiles.dwNumberOfItems as usize {
                        let prof = &*profiles.ProfileInfo.as_ptr().add(p);
                        let name = String::from_utf16_lossy(&prof.strProfileName);
                        profile_names.push(name.trim_matches('\0').to_string());
                    }
                    WlanFreeMemory(prof_list as *const _);
                }

                let mut current_connected_ssid: Option<String> = None;
                let mut data_size = 0u32;
                let mut data: *mut core::ffi::c_void = null_mut();
                let mut value_type = WLAN_OPCODE_VALUE_TYPE::default();
                if WlanQueryInterface(
                    client,
                    &info.InterfaceGuid,
                    wlan_intf_opcode_current_connection,
                    None,
                    &mut data_size,
                    &mut data,
                    Some(&mut value_type),
                ) == 0
                    && !data.is_null()
                    && data_size as usize >= size_of::<WLAN_CONNECTION_ATTRIBUTES>()
                {
                    let conn = &*(data as *const WLAN_CONNECTION_ATTRIBUTES);
                    let ssid = &conn.wlanAssociationAttributes.dot11Ssid;
                    let length = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
                    current_connected_ssid =
                        Some(String::from_utf8_lossy(&ssid.ucSSID[..length]).to_string());
                }
                if !data.is_null() {
                    WlanFreeMemory(data);
                }

                let mut net_list: *mut WLAN_AVAILABLE_NETWORK_LIST = null_mut();
                if WlanGetAvailableNetworkList(client, &info.InterfaceGuid, 0, None, &mut net_list)
                    == 0
                    && !net_list.is_null()
                {
                    let nets = &*net_list;
                    for n in 0..nets.dwNumberOfItems as usize {
                        let net = &*nets.Network.as_ptr().add(n);
                        let len =
                            (net.dot11Ssid.uSSIDLength as usize).min(net.dot11Ssid.ucSSID.len());
                        if len == 0 {
                            continue;
                        }
                        let ssid =
                            String::from_utf8_lossy(&net.dot11Ssid.ucSSID[..len]).to_string();
                        let signal = net.wlanSignalQuality.min(100);
                        let is_conn = (net.dwFlags & WLAN_AVAILABLE_NETWORK_CONNECTED) != 0
                            || current_connected_ssid
                                .as_ref()
                                .map(|s| s == &ssid)
                                .unwrap_or(false);
                        let security = net.bSecurityEnabled.as_bool();
                        let prof_exists = profile_names.iter().any(|p| p == &ssid);

                        if let Some(existing) = items.iter_mut().find(|it| it.ssid == ssid) {
                            if is_conn {
                                existing.connected = true;
                            }
                            if signal > existing.signal {
                                existing.signal = signal;
                            }
                        } else {
                            items.push(WifiNetworkItem {
                                ssid,
                                signal,
                                connected: is_conn,
                                security,
                                profile_exists: prof_exists,
                            });
                        }
                    }
                    WlanFreeMemory(net_list as *const _);
                }
            }
            WlanFreeMemory(if_list as *const _);
        }
        let _ = WlanCloseHandle(client, None);
    }

    items.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then_with(|| b.signal.cmp(&a.signal))
    });

    NETWORK_ITEMS = items;
    NETWORK_SCROLL_OFFSET = 0;
}

pub(crate) unsafe fn scan_wifi_networks() {
    let mut negotiated_version = 0u32;
    let mut client = HANDLE::default();
    if WlanOpenHandle(2, None, &mut negotiated_version, &mut client) == 0 {
        let mut if_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        if WlanEnumInterfaces(client, None, &mut if_list) == 0 && !if_list.is_null() {
            let list = &*if_list;
            for i in 0..list.dwNumberOfItems as usize {
                let info = &*list.InterfaceInfo.as_ptr().add(i);
                let _ = WlanScan(client, &info.InterfaceGuid, None, None, None);
            }
            WlanFreeMemory(if_list as *const _);
        }
        let _ = WlanCloseHandle(client, None);
    }
    refresh_wifi_networks();
}

pub(crate) unsafe fn connect_wifi_network(ssid: &str) {
    let is_connected = NETWORK_ITEMS
        .iter()
        .any(|it| it.ssid == ssid && it.connected);
    let profile_exists = NETWORK_ITEMS
        .iter()
        .any(|it| it.ssid == ssid && it.profile_exists);
    let mut negotiated_version = 0u32;
    let mut client = HANDLE::default();
    if WlanOpenHandle(2, None, &mut negotiated_version, &mut client) == 0 {
        let mut if_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        if WlanEnumInterfaces(client, None, &mut if_list) == 0 && !if_list.is_null() {
            let list = &*if_list;
            if list.dwNumberOfItems > 0 {
                let info = &*list.InterfaceInfo.as_ptr();

                if is_connected {
                    let _ = WlanDisconnect(client, &info.InterfaceGuid, None);
                } else if profile_exists {
                    let prof_name = wide(ssid);
                    let conn_params = WLAN_CONNECTION_PARAMETERS {
                        wlanConnectionMode: wlan_connection_mode_profile,
                        strProfile: PCWSTR(prof_name.as_ptr()),
                        pDot11Ssid: null_mut(),
                        pDesiredBssidList: null_mut(),
                        dot11BssType: dot11_BSS_type_any,
                        dwFlags: 0,
                    };
                    let res = WlanConnect(client, &info.InterfaceGuid, &conn_params, None);
                    if res != 0 {
                        launch("ms-availablenetworks:");
                    }
                } else {
                    launch("ms-availablenetworks:");
                }
            }
            WlanFreeMemory(if_list as *const _);
        }
        let _ = WlanCloseHandle(client, None);
    }
    NETWORK_TICKS = 4;
}

pub(crate) unsafe fn refresh_bluetooth_devices() {
    let mut items = Vec::new();
    let params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
        dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
        fReturnAuthenticated: BOOL(1),
        fReturnRemembered: BOOL(1),
        fReturnUnknown: BOOL(0),
        fReturnConnected: BOOL(1),
        fIssueInquiry: BOOL(0),
        cTimeoutMultiplier: 0,
        hRadio: HANDLE::default(),
    };
    let mut info = BLUETOOTH_DEVICE_INFO {
        dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
        ..Default::default()
    };
    if let Ok(find_handle) = BluetoothFindFirstDevice(&params, &mut info) {
        loop {
            let name = String::from_utf16_lossy(&info.szName);
            let clean_name = name.trim_matches('\0').to_string();
            if !clean_name.is_empty() {
                items.push(BtDeviceItem {
                    name: clean_name,
                    connected: info.fConnected.as_bool(),
                    remembered: info.fRemembered.as_bool(),
                    authenticated: info.fAuthenticated.as_bool(),
                    info,
                });
            }
            info = BLUETOOTH_DEVICE_INFO {
                dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };
            if BluetoothFindNextDevice(find_handle, &mut info).is_err() {
                break;
            }
        }
        let _ = BluetoothFindDeviceClose(find_handle);
    }
    items.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then_with(|| a.name.cmp(&b.name))
    });
    BLUETOOTH_ITEMS = items;
    BLUETOOTH_SCROLL_OFFSET = 0;
}

pub(crate) unsafe fn toggle_bluetooth_device(_device: &BtDeviceItem) {
    // Legacy Win32 BluetoothSetServiceState is incompatible with modern Bluetooth audio (A2DP/BLE/AVRCP),
    // causing driver deadlocks, kernel timeouts, and freezing the shell's input hooks.
    // Launch Windows Bluetooth settings to safely and reliably establish the audio/peripheral connection.
    launch("ms-settings:bluetooth");
    BLUETOOTH_TICKS = 4;
}

pub(crate) unsafe fn show_network_popup() {
    if NETWORK_POPUP.0.is_null() {
        return;
    }
    if IsWindowVisible(NETWORK_POPUP).as_bool() {
        ShowWindow(NETWORK_POPUP, SW_HIDE);
        return;
    }
    if !BLUETOOTH_POPUP.0.is_null() && IsWindowVisible(BLUETOOTH_POPUP).as_bool() {
        ShowWindow(BLUETOOTH_POPUP, SW_HIDE);
    }
    if !AUDIO_MIXER.0.is_null() && IsWindowVisible(AUDIO_MIXER).as_bool() {
        ShowWindow(AUDIO_MIXER, SW_HIDE);
    }
    if !TRAY_POPUP.0.is_null() && IsWindowVisible(TRAY_POPUP).as_bool() {
        ShowWindow(TRAY_POPUP, SW_HIDE);
    }
    tiling_popup::hide_tiling_popup();

    refresh_wifi_networks();
    NETWORK_POPUP_SELECTION = 0;

    let mut cursor = POINT::default();
    GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info);

    let x = (cursor.x - NETWORK_POPUP_WIDTH / 2)
        .max(info.rcWork.left + 8)
        .min(info.rcWork.right - NETWORK_POPUP_WIDTH - 8);
    let mut panel_r = RECT::default();
    let panel_bottom = if !PANEL.0.is_null() && GetWindowRect(PANEL, &mut panel_r).is_ok() {
        panel_r.bottom
    } else {
        info.rcWork.top
    };
    let y = if cursor.y < panel_bottom + 100 {
        panel_bottom + 4
    } else {
        (cursor.y - NETWORK_POPUP_HEIGHT - 8).max(info.rcWork.top + 8)
    };

    let _ = SetWindowPos(
        NETWORK_POPUP,
        HWND_TOPMOST,
        x,
        y,
        NETWORK_POPUP_WIDTH,
        NETWORK_POPUP_HEIGHT,
        SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(NETWORK_POPUP, None, false);
    ShowWindow(NETWORK_POPUP, SW_SHOW);
    SetForegroundWindow(NETWORK_POPUP);
    SetFocus(NETWORK_POPUP);
}

pub(crate) unsafe fn show_bluetooth_popup() {
    if BLUETOOTH_POPUP.0.is_null() {
        return;
    }
    if IsWindowVisible(BLUETOOTH_POPUP).as_bool() {
        ShowWindow(BLUETOOTH_POPUP, SW_HIDE);
        return;
    }
    if !NETWORK_POPUP.0.is_null() && IsWindowVisible(NETWORK_POPUP).as_bool() {
        ShowWindow(NETWORK_POPUP, SW_HIDE);
    }
    if !AUDIO_MIXER.0.is_null() && IsWindowVisible(AUDIO_MIXER).as_bool() {
        ShowWindow(AUDIO_MIXER, SW_HIDE);
    }
    if !TRAY_POPUP.0.is_null() && IsWindowVisible(TRAY_POPUP).as_bool() {
        ShowWindow(TRAY_POPUP, SW_HIDE);
    }
    tiling_popup::hide_tiling_popup();

    refresh_bluetooth_devices();
    BLUETOOTH_POPUP_SELECTION = 0;

    let mut cursor = POINT::default();
    GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info);

    let x = (cursor.x - BLUETOOTH_POPUP_WIDTH / 2)
        .max(info.rcWork.left + 8)
        .min(info.rcWork.right - BLUETOOTH_POPUP_WIDTH - 8);
    let mut panel_r = RECT::default();
    let panel_bottom = if !PANEL.0.is_null() && GetWindowRect(PANEL, &mut panel_r).is_ok() {
        panel_r.bottom
    } else {
        info.rcWork.top
    };
    let y = if cursor.y < panel_bottom + 100 {
        panel_bottom + 4
    } else {
        (cursor.y - BLUETOOTH_POPUP_HEIGHT - 8).max(info.rcWork.top + 8)
    };

    let _ = SetWindowPos(
        BLUETOOTH_POPUP,
        HWND_TOPMOST,
        x,
        y,
        BLUETOOTH_POPUP_WIDTH,
        BLUETOOTH_POPUP_HEIGHT,
        SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(BLUETOOTH_POPUP, None, false);
    ShowWindow(BLUETOOTH_POPUP, SW_SHOW);
    SetForegroundWindow(BLUETOOTH_POPUP);
    SetFocus(BLUETOOTH_POPUP);
}

pub(crate) const TRAY_POPUP_WIDTH: i32 = 280;

pub(crate) unsafe fn show_tray_popup() {
    if TRAY_POPUP.0.is_null() {
        return;
    }
    if IsWindowVisible(TRAY_POPUP).as_bool() {
        ShowWindow(TRAY_POPUP, SW_HIDE);
        return;
    }
    if !NETWORK_POPUP.0.is_null() && IsWindowVisible(NETWORK_POPUP).as_bool() {
        ShowWindow(NETWORK_POPUP, SW_HIDE);
    }
    if !BLUETOOTH_POPUP.0.is_null() && IsWindowVisible(BLUETOOTH_POPUP).as_bool() {
        ShowWindow(BLUETOOTH_POPUP, SW_HIDE);
    }
    if !AUDIO_MIXER.0.is_null() && IsWindowVisible(AUDIO_MIXER).as_bool() {
        ShowWindow(AUDIO_MIXER, SW_HIDE);
    }
    tiling_popup::hide_tiling_popup();

    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    let item_count = TRAY_ICONS.len();
    let popup_h = if item_count == 0 {
        120
    } else {
        (item_count as i32 * 36 + 75).clamp(120, 420)
    };

    let mut cursor = POINT::default();
    GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let _ = GetMonitorInfoW(monitor, &mut info);

    let x = (cursor.x - TRAY_POPUP_WIDTH / 2)
        .max(info.rcWork.left + 8)
        .min(info.rcWork.right - TRAY_POPUP_WIDTH - 8);
    let mut panel_r = RECT::default();
    let panel_bottom = if !PANEL.0.is_null() && GetWindowRect(PANEL, &mut panel_r).is_ok() {
        panel_r.bottom
    } else {
        info.rcWork.top
    };
    let y = panel_bottom + 4;

    let _ = SetWindowPos(
        TRAY_POPUP,
        HWND_TOPMOST,
        x,
        y,
        TRAY_POPUP_WIDTH,
        popup_h,
        SWP_NOACTIVATE,
    );
    let _ = InvalidateRect(TRAY_POPUP, None, false);
    ShowWindow(TRAY_POPUP, SW_SHOW);
    SetForegroundWindow(TRAY_POPUP);
    SetFocus(TRAY_POPUP);
}

pub(crate) unsafe fn paint_tray_popup(hwnd: HWND, screen_dc: HDC) {
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

    let dark = CONFIG.dark;
    let bg = brush(if dark { 0x1b1e24 } else { 0xffffff });
    let border = if dark { 0x334155 } else { 0xe2e8f0 };
    let primary = if dark { 0xf8fafc } else { 0x0f172a };
    let secondary = if dark { 0x94a3b8 } else { 0x64748b };

    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);
    stroke_round_rect(hdc, &r, border, 1, 12);

    text(hdc, "System Tray", 20, 16, primary, 15);
    text(hdc, "Active background applications", 20, 38, secondary, 11);

    // Refresh icon at top right
    icon_text(hdc, "\u{E72C}", w - 36, 16, secondary, 13);

    let sep = RECT {
        left: 16,
        top: 56,
        right: w - 16,
        bottom: 57,
    };
    let sep_brush = brush(border);
    FillRect(hdc, &sep, sep_brush);
    let _ = DeleteObject(sep_brush);

    TRAY_ICONS.retain(|it| IsWindow(it.hwnd).as_bool());
    if TRAY_ICONS.is_empty() {
        text(hdc, "No tray applications active", 20, 72, secondary, 12);
    } else {
        for (i, item) in TRAY_ICONS.iter().enumerate() {
            let row_y = 66 + (i as i32 * 36);
            if row_y + 32 > h {
                break;
            }
            if !item.icon.is_invalid() {
                let _ = DrawIconEx(hdc, 20, row_y + 4, item.icon, 20, 20, 0, None, DI_NORMAL);
            }
            let label = if !item.tip.is_empty() {
                if item.tip.len() > 28 {
                    format!("{}...", &item.tip[..28])
                } else {
                    item.tip.clone()
                }
            } else {
                format!("App #{}", item.uid)
            };
            text(hdc, &label, 50, row_y + 6, primary, 12);
        }
    }

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) extern "system" fn tray_popup_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_tray_popup(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let w_client = r.right - r.left;

                if y < 45 && x >= w_client - 45 {
                    broadcast_taskbar_created();
                    let _ = InvalidateRect(hwnd, None, false);
                } else if y >= 66 {
                    let idx = ((y - 66) / 36) as usize;
                    if let Some(item) = TRAY_ICONS.get(idx) {
                        ShowWindow(hwnd, SW_HIDE);
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(WM_LBUTTONDOWN as isize),
                        );
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(WM_LBUTTONUP as isize),
                        );
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(0x0402 as isize),
                        );
                    }
                }
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                if y >= 66 {
                    let idx = ((y - 66) / 36) as usize;
                    if let Some(item) = TRAY_ICONS.get(idx) {
                        ShowWindow(hwnd, SW_HIDE);
                        SetForegroundWindow(item.hwnd);
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(WM_RBUTTONDOWN as isize),
                        );
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(WM_RBUTTONUP as isize),
                        );
                        PostMessageW(
                            item.hwnd,
                            item.callback_msg,
                            WPARAM(item.uid as usize),
                            LPARAM(0x0404 as isize),
                        );
                    }
                }
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_KILLFOCUS | WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

pub(crate) unsafe fn paint_network_popup(hwnd: HWND, screen_dc: HDC) {
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

    let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);
    let primary = if CONFIG.dark { 0xf7f8fa } else { 0x172033 };
    let secondary = if CONFIG.dark { 0x9da6b4 } else { 0x64748b };
    let border = if CONFIG.dark { 0x2e3542 } else { 0xd5dce6 };

    text(hdc, "Wi-Fi Networks", 22, 18, primary, 20);

    let (connected, _wifi, details) = network_snapshot();
    let status_str = if let Some((ssid, quality)) = &details {
        format!("Connected: {} ({}%)", ssid, quality)
    } else if connected {
        "Connected (Ethernet)".to_string()
    } else {
        "Not connected".to_string()
    };
    text(
        hdc,
        &status_str,
        22,
        48,
        if connected { 0x38bdf8 } else { 0xf87171 },
        13,
    );
    text(
        hdc,
        "Click to manage in Settings · Esc to close",
        22,
        70,
        secondary,
        11,
    );

    icon_text(hdc, "\u{E72C}", w - 38, 20, secondary, 14);

    let sep_brush = brush(border);
    let sep_rect = RECT {
        left: 20,
        top: 92,
        right: w - 20,
        bottom: 93,
    };
    FillRect(hdc, &sep_rect, sep_brush);
    let _ = DeleteObject(sep_brush);

    let total = NETWORK_ITEMS.len();
    let visible_count = total.saturating_sub(NETWORK_SCROLL_OFFSET).min(8);
    for idx in 0..visible_count {
        let item_idx = NETWORK_SCROLL_OFFSET + idx;
        let item = &NETWORK_ITEMS[item_idx];
        let y = 100 + idx as i32 * 34;

        if item_idx == NETWORK_POPUP_SELECTION {
            let highlight = brush(if CONFIG.dark { 0x2c3340 } else { 0xe2e8f0 });
            let selected_row = RECT {
                left: 14,
                top: y - 4,
                right: w - 14,
                bottom: y + 28,
            };
            fill_round_rect(hdc, &selected_row, highlight, 6);
            let _ = DeleteObject(highlight);
        }

        let icon = if item.signal >= 75 {
            "\u{E701}"
        } else if item.signal >= 50 {
            "\u{E874}"
        } else if item.signal >= 25 {
            "\u{E873}"
        } else {
            "\u{E872}"
        };
        let icon_color = if item.connected { 0x38bdf8 } else { secondary };
        icon_text(hdc, icon, 24, y + 2, icon_color, 14);

        if item.security {
            icon_text(hdc, "\u{E72E}", 48, y + 4, secondary, 11);
        }

        let name_x = if item.security { 66 } else { 52 };
        let display_name = if item.ssid.len() > 20 {
            format!("{}...", &item.ssid[..18])
        } else {
            item.ssid.clone()
        };
        text(
            hdc,
            &display_name,
            name_x,
            y + 2,
            if item.connected { 0x38bdf8 } else { primary },
            13,
        );

        if item.connected {
            text(hdc, "Connected", w - 110, y + 2, 0x38bdf8, 12);
        }
        text(
            hdc,
            &format!("{}%", item.signal),
            w - 42,
            y + 2,
            secondary,
            11,
        );
    }

    if total == 0 {
        text(hdc, "No Wi-Fi networks found", 24, 120, secondary, 13);
    }

    let footer_y = h - 38;
    let footer_sep = RECT {
        left: 20,
        top: footer_y - 8,
        right: w - 20,
        bottom: footer_y - 7,
    };
    let sep_brush2 = brush(border);
    FillRect(hdc, &footer_sep, sep_brush2);
    let _ = DeleteObject(sep_brush2);

    icon_text(hdc, "\u{E713}", 24, footer_y, secondary, 13);
    text(hdc, "More network settings", 46, footer_y, secondary, 12);

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) extern "system" fn network_popup_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_network_popup(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let w_client = r.right - r.left;
                let h_client = r.bottom - r.top;

                if y < 45 && x >= w_client - 50 {
                    scan_wifi_networks();
                } else if y >= h_client - 45 {
                    ShowWindow(hwnd, SW_HIDE);
                    launch("ms-settings:network-wifi");
                } else if (98..h_client - 45).contains(&y) {
                    let idx = ((y - 98) / 34) as usize;
                    let item_idx = NETWORK_SCROLL_OFFSET + idx;
                    if item_idx < NETWORK_ITEMS.len() {
                        NETWORK_POPUP_SELECTION = item_idx;
                        notify_accessibility_focus(hwnd, NETWORK_POPUP_SELECTION as i32);
                        let ssid = NETWORK_ITEMS[item_idx].ssid.clone();
                        connect_wifi_network(&ssid);
                        refresh_wifi_networks();
                    }
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((w.0 >> 16) & 0xffff) as i16;
                let total = NETWORK_ITEMS.len();
                if delta > 0 {
                    NETWORK_SCROLL_OFFSET = NETWORK_SCROLL_OFFSET.saturating_sub(1);
                } else if delta < 0 && NETWORK_SCROLL_OFFSET + 8 < total {
                    NETWORK_SCROLL_OFFSET += 1;
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let total = NETWORK_ITEMS.len();
                match w.0 as u32 {
                    value if value == VK_UP.0 as u32 => {
                        NETWORK_POPUP_SELECTION = NETWORK_POPUP_SELECTION.saturating_sub(1);
                        if NETWORK_POPUP_SELECTION < NETWORK_SCROLL_OFFSET {
                            NETWORK_SCROLL_OFFSET = NETWORK_POPUP_SELECTION;
                        }
                        notify_accessibility_focus(hwnd, NETWORK_POPUP_SELECTION as i32);
                    }
                    value if value == VK_DOWN.0 as u32 => {
                        if total > 0 {
                            NETWORK_POPUP_SELECTION = (NETWORK_POPUP_SELECTION + 1).min(total - 1);
                            if NETWORK_POPUP_SELECTION >= NETWORK_SCROLL_OFFSET + 8 {
                                NETWORK_SCROLL_OFFSET = NETWORK_POPUP_SELECTION - 7;
                            }
                            notify_accessibility_focus(hwnd, NETWORK_POPUP_SELECTION as i32);
                        }
                    }
                    value if value == VK_RETURN.0 as u32 => {
                        if NETWORK_POPUP_SELECTION < total {
                            let ssid = NETWORK_ITEMS[NETWORK_POPUP_SELECTION].ssid.clone();
                            connect_wifi_network(&ssid);
                            refresh_wifi_networks();
                        }
                    }
                    value if value == VK_F5.0 as u32 => {
                        scan_wifi_networks();
                    }
                    _ => return DefWindowProcW(hwnd, msg, w, l),
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_KILLFOCUS | WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

pub(crate) unsafe fn paint_bluetooth_popup(hwnd: HWND, screen_dc: HDC) {
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

    let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
    FillRect(hdc, &r, bg);
    let _ = DeleteObject(bg);
    let primary = if CONFIG.dark { 0xf7f8fa } else { 0x172033 };
    let secondary = if CONFIG.dark { 0x9da6b4 } else { 0x64748b };
    let border = if CONFIG.dark { 0x2e3542 } else { 0xd5dce6 };

    text(hdc, "Bluetooth Devices", 22, 18, primary, 20);

    let (present, any_connected) = bluetooth_status_snapshot();
    let status_str = if any_connected {
        "Connected to device(s)".to_string()
    } else if present {
        "Bluetooth is on".to_string()
    } else {
        "Bluetooth is off / not found".to_string()
    };
    text(
        hdc,
        &status_str,
        22,
        48,
        if any_connected { 0x38bdf8 } else { secondary },
        13,
    );
    text(
        hdc,
        "Click to manage in Settings · Esc to close",
        22,
        70,
        secondary,
        11,
    );

    icon_text(hdc, "\u{E72C}", w - 38, 20, secondary, 14);

    let sep_brush = brush(border);
    let sep_rect = RECT {
        left: 20,
        top: 92,
        right: w - 20,
        bottom: 93,
    };
    FillRect(hdc, &sep_rect, sep_brush);
    let _ = DeleteObject(sep_brush);

    let total = BLUETOOTH_ITEMS.len();
    let visible_count = total.saturating_sub(BLUETOOTH_SCROLL_OFFSET).min(8);
    for idx in 0..visible_count {
        let item_idx = BLUETOOTH_SCROLL_OFFSET + idx;
        let item = &BLUETOOTH_ITEMS[item_idx];
        let y = 100 + idx as i32 * 34;

        if item_idx == BLUETOOTH_POPUP_SELECTION {
            let highlight = brush(if CONFIG.dark { 0x2c3340 } else { 0xe2e8f0 });
            let selected_row = RECT {
                left: 14,
                top: y - 4,
                right: w - 14,
                bottom: y + 28,
            };
            fill_round_rect(hdc, &selected_row, highlight, 6);
            let _ = DeleteObject(highlight);
        }

        let lower = item.name.to_lowercase();
        let icon = if lower.contains("bud")
            || lower.contains("head")
            || lower.contains("ear")
            || lower.contains("airpod")
        {
            "\u{E7F5}"
        } else if lower.contains("speaker") || lower.contains("sound") {
            "\u{E767}"
        } else if lower.contains("mouse") || lower.contains("keyboard") {
            "\u{E962}"
        } else {
            "\u{E702}"
        };
        let icon_color = if item.connected { 0x38bdf8 } else { secondary };
        icon_text(hdc, icon, 24, y + 2, icon_color, 14);

        let display_name = if item.name.len() > 22 {
            format!("{}...", &item.name[..20])
        } else {
            item.name.clone()
        };
        text(
            hdc,
            &display_name,
            50,
            y + 2,
            if item.connected { 0x38bdf8 } else { primary },
            13,
        );

        if item.connected {
            text(hdc, "Connected", w - 90, y + 2, 0x38bdf8, 12);
        } else {
            text(hdc, "Paired", w - 65, y + 2, secondary, 11);
        }
    }

    if total == 0 {
        text(hdc, "No paired Bluetooth devices", 24, 120, secondary, 13);
    }

    let footer_y = h - 38;
    let footer_sep = RECT {
        left: 20,
        top: footer_y - 8,
        right: w - 20,
        bottom: footer_y - 7,
    };
    let sep_brush2 = brush(border);
    FillRect(hdc, &footer_sep, sep_brush2);
    let _ = DeleteObject(sep_brush2);

    icon_text(hdc, "\u{E713}", 24, footer_y, secondary, 13);
    text(
        hdc,
        "Add device / Bluetooth settings",
        46,
        footer_y,
        secondary,
        12,
    );

    let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
    SelectObject(hdc, old_bmp);
    let _ = DeleteObject(mem_bmp);
    let _ = DeleteDC(hdc);
}

pub(crate) extern "system" fn bluetooth_popup_proc(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let h = BeginPaint(hwnd, &mut ps);
                paint_bluetooth_popup(hwnd, h);
                EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                let x = (l.0 as u32 & 0xffff) as i32;
                let y = ((l.0 as u32 >> 16) & 0xffff) as i32;
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let w_client = r.right - r.left;
                let h_client = r.bottom - r.top;

                if y < 45 && x >= w_client - 50 {
                    refresh_bluetooth_devices();
                } else if y >= h_client - 45 {
                    ShowWindow(hwnd, SW_HIDE);
                    launch("ms-settings:bluetooth");
                } else if (98..h_client - 45).contains(&y) {
                    let idx = ((y - 98) / 34) as usize;
                    let item_idx = BLUETOOTH_SCROLL_OFFSET + idx;
                    if item_idx < BLUETOOTH_ITEMS.len() {
                        BLUETOOTH_POPUP_SELECTION = item_idx;
                        notify_accessibility_focus(hwnd, BLUETOOTH_POPUP_SELECTION as i32);
                        let dev = BLUETOOTH_ITEMS[item_idx].clone();
                        ShowWindow(hwnd, SW_HIDE);
                        toggle_bluetooth_device(&dev);
                    }
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((w.0 >> 16) & 0xffff) as i16;
                let total = BLUETOOTH_ITEMS.len();
                if delta > 0 {
                    BLUETOOTH_SCROLL_OFFSET = BLUETOOTH_SCROLL_OFFSET.saturating_sub(1);
                } else if delta < 0 && BLUETOOTH_SCROLL_OFFSET + 8 < total {
                    BLUETOOTH_SCROLL_OFFSET += 1;
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_KEYDOWN if w.0 as u32 == VK_ESCAPE.0 as u32 => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let total = BLUETOOTH_ITEMS.len();
                match w.0 as u32 {
                    value if value == VK_UP.0 as u32 => {
                        BLUETOOTH_POPUP_SELECTION = BLUETOOTH_POPUP_SELECTION.saturating_sub(1);
                        if BLUETOOTH_POPUP_SELECTION < BLUETOOTH_SCROLL_OFFSET {
                            BLUETOOTH_SCROLL_OFFSET = BLUETOOTH_POPUP_SELECTION;
                        }
                        notify_accessibility_focus(hwnd, BLUETOOTH_POPUP_SELECTION as i32);
                    }
                    value if value == VK_DOWN.0 as u32 => {
                        if total > 0 {
                            BLUETOOTH_POPUP_SELECTION =
                                (BLUETOOTH_POPUP_SELECTION + 1).min(total - 1);
                            if BLUETOOTH_POPUP_SELECTION >= BLUETOOTH_SCROLL_OFFSET + 8 {
                                BLUETOOTH_SCROLL_OFFSET = BLUETOOTH_POPUP_SELECTION - 7;
                            }
                            notify_accessibility_focus(hwnd, BLUETOOTH_POPUP_SELECTION as i32);
                        }
                    }
                    value if value == VK_RETURN.0 as u32 => {
                        if BLUETOOTH_POPUP_SELECTION < total {
                            let dev = BLUETOOTH_ITEMS[BLUETOOTH_POPUP_SELECTION].clone();
                            ShowWindow(hwnd, SW_HIDE);
                            toggle_bluetooth_device(&dev);
                        }
                    }
                    value if value == VK_F5.0 as u32 => {
                        refresh_bluetooth_devices();
                    }
                    _ => return DefWindowProcW(hwnd, msg, w, l),
                }
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_KILLFOCUS | WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
