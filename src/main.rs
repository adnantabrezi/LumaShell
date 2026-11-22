#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn, unused_must_use, static_mut_refs)]

#[cfg(target_os = "windows")]
mod shell {
    use std::{
        env,
        ffi::OsStr,
        fs,
        io::Write,
        mem::size_of,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::null_mut,
    };
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };
    use windows::Win32::Devices::Bluetooth::{
        BLUETOOTH_FIND_RADIO_PARAMS, BluetoothFindFirstRadio, BluetoothFindRadioClose,
    };
    use windows::Win32::Foundation::{
        BOOL, COLORREF, CloseHandle, ERROR_ALREADY_EXISTS, ERROR_BUFFER_OVERFLOW, GetLastError,
        HANDLE, HINSTANCE, HLOCAL, HWND, LPARAM, LRESULT, LocalFree, POINT, RECT, SYSTEMTIME,
        WIN32_ERROR, WPARAM,
    };
    use windows::Win32::Graphics::Dwm::{
        DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_VISIBLE,
        DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmRegisterThumbnail, DwmSetWindowAttribute,
        DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
    };
    use windows::Win32::Graphics::Gdi::{
        BITMAP, BeginPaint, BitBlt, CLEARTYPE_QUALITY, ClientToScreen, CreateCompatibleBitmap,
        CreateCompatibleDC, CreateFontW, CreatePen, CreateSolidBrush, DEFAULT_CHARSET, DeleteDC,
        DeleteObject, EndPaint, EnumDisplayMonitors, FillRect, GetMonitorInfoW, GetObjectW,
        HBITMAP, HBRUSH, HDC, HMONITOR, InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITORINFO,
        MonitorFromPoint, MonitorFromWindow, PAINTSTRUCT, PS_NULL, PS_SOLID, RoundRect, SRCCOPY,
        ScreenToClient, SelectObject, SetBkMode, SetTextColor, StretchBlt, TRANSPARENT, TextOutW,
    };
    use windows::Win32::Media::Audio::Endpoints::{
        IAudioEndpointVolume, IAudioEndpointVolumeCallback,
    };
    use windows::Win32::Media::Audio::{
        AudioSessionStateActive, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
        ISimpleAudioVolume, eMultimedia, eRender,
    };
    use windows::Win32::NetworkManagement::IpHelper::{
        GET_ADAPTERS_ADDRESSES_FLAGS, GetAdaptersAddresses, IF_TYPE_IEEE80211,
        IF_TYPE_SOFTWARE_LOOPBACK, IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
    use windows::Win32::NetworkManagement::WiFi::{
        WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST, WLAN_OPCODE_VALUE_TYPE,
        WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface,
        wlan_interface_state_connected, wlan_intf_opcode_current_connection,
    };
    use windows::Win32::Networking::WinSock::AF_UNSPEC;
    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Power::{
        GetSystemPowerStatus, HPOWERNOTIFY, POWERBROADCAST_SETTING, PowerGetActiveScheme,
        PowerReadACValueIndex, PowerSetActiveScheme, PowerWriteACValueIndex,
        RegisterPowerSettingNotification, SYSTEM_POWER_STATUS, UnregisterPowerSettingNotification,
    };
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, KEY_WRITE, REG_DWORD, REG_OPEN_CREATE_OPTIONS,
        REG_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW,
    };
    use windows::Win32::System::Shutdown::{
        EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, ExitWindowsEx, LockWorkStation, SHUTDOWN_REASON,
    };
    use windows::Win32::System::SystemInformation::GetLocalTime;
    use windows::Win32::System::Threading::{
        CreateMutexW, OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess,
    };
    use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize};
    use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW, NotifyWinEvent};
    use windows::Win32::UI::HiDpi::{
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, SetProcessDpiAwarenessContext,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyState, KEYEVENTF_KEYUP, MOD_ALT, MOD_SHIFT, MOD_WIN, RegisterHotKey, ReleaseCapture,
        SetCapture, SetFocus, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, UnregisterHotKey,
        VIRTUAL_KEY, VK_DOWN, VK_END, VK_ESCAPE, VK_F5, VK_F6, VK_HOME, VK_LCONTROL, VK_LEFT,
        VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RWIN, VK_SPACE,
        VK_TAB, VK_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, keybd_event,
    };
    use windows::Win32::UI::Shell::{
        ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABN_FULLSCREENAPP, ABN_POSCHANGED, APPBARDATA,
        ExtractIconExW, IVirtualDesktopManager, QUNS_ACCEPTS_NOTIFICATIONS, QUNS_QUIET_TIME,
        SHAppBarMessage, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON, SHGetFileInfoW,
        SHQueryUserNotificationState, ShellExecuteW, VirtualDesktopManager,
    };
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::{GUID, HRESULT, Interface, PCWSTR, PWSTR};

    const PANEL_HEIGHT: i32 = 38;
    const DOCK_WIDTH: i32 = 486;
    const DOCK_HEIGHT: i32 = 82;
    const SETTINGS_WIDTH: i32 = 760;
    const SETTINGS_HEIGHT: i32 = 530;
    const LAUNCHER_WIDTH: i32 = 520;
    const LAUNCHER_HEIGHT: i32 = 520;
    const SWITCHER_WIDTH: i32 = 620;
    const SWITCHER_HEIGHT: i32 = 430;
    const AUDIO_MIXER_WIDTH: i32 = 360;
    const AUDIO_MIXER_HEIGHT: i32 = 420;
    const HOTKEY_DOCK: i32 = 10;
    const HOTKEY_SWITCHER: i32 = 11;
    const HOTKEY_WORKSPACE_PREVIOUS: i32 = 12;
    const HOTKEY_WORKSPACE_NEXT: i32 = 13;
    const HOTKEY_MOVE_WINDOW_PREVIOUS: i32 = 14;
    const HOTKEY_MOVE_WINDOW_NEXT: i32 = 15;
    const HOTKEY_LAUNCHER: i32 = 16;
    const OSD_WIDTH: i32 = 260;
    const OSD_HEIGHT: i32 = 54;
    const ID_OSD_TIMER: usize = 701;
    const WM_APP_AUDIO_CHANGE: u32 = WM_APP + 101;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum OsdKind {
        Volume,
        Brightness,
    }
    const WM_MOUSE_LEAVE: u32 = 0x02A3;
    const EM_SETCUEBANNER: u32 = 0x1501;
    const ID_SEARCH: usize = 301;
    const ID_DARK: usize = 201;
    const ID_CLOCK_CHECK: usize = 202;
    const ID_NETWORK_CHECK: usize = 203;
    const ID_VOLUME_CHECK: usize = 204;
    const ID_BATTERY_CHECK: usize = 205;
    const ID_LEFT: usize = 206;
    const ID_CENTER: usize = 207;
    const ID_RIGHT: usize = 208;
    const ID_RESET: usize = 209;
    const ID_RESTORE_EXPLORER: usize = 210;
    const ID_AUTO_HIDE: usize = 211;
    const ID_ICON_SMALL: usize = 212;
    const ID_ICON_MEDIUM: usize = 213;
    const ID_ICON_LARGE: usize = 214;
    const ID_CLOCK_24H: usize = 215;
    const ID_NOTIFICATIONS: usize = 216;
    const ID_ORDER: usize = 217;
    const ID_DOCK_ORDER: usize = 218;
    const ID_BLUETOOTH: usize = 219;
    const ID_MEDIA: usize = 220;
    const ID_DATE_FORMAT: usize = 221;
    const ID_OPEN_DIAGNOSTICS: usize = 222;
    const ID_STARTUP: usize = 223;
    const ID_ZONES: usize = 224;
    const ID_RESTORE_SETTINGS: usize = 225;
    const ID_MAGNIFICATION: usize = 226;
    const ID_TRANSPARENCY: usize = 227;
    const ID_SPACING: usize = 228;
    const ID_OFFSETS: usize = 229;
    const ID_OPEN_WATCHDOG: usize = 234;
    const ID_SET_LUMA_SHELL: usize = 236;
    const ID_DESKTOP_TOGGLE: usize = 237;

    const ID_MENU_LOCK: usize = 1001;
    const ID_MENU_LOGOFF: usize = 1002;
    const ID_MENU_SLEEP: usize = 1003;
    const ID_MENU_RESTART: usize = 1004;
    const ID_MENU_SHUTDOWN: usize = 1005;
    const ID_MENU_SETTINGS: usize = 1006;
    const ID_MENU_RESTORE_EXPLORER: usize = 1007;

    const ID_DESKTOP_TERMINAL: usize = 1010;
    const ID_DESKTOP_FILES: usize = 1011;
    const ID_DESKTOP_LAUNCHER: usize = 1012;
    const ID_DESKTOP_SWITCHER: usize = 1013;
    const ID_DESKTOP_DISPLAY: usize = 1014;
    const ID_DESKTOP_PERSONALIZE: usize = 1015;

    const ID_PAGE_GENERAL: usize = 230;
    const ID_PAGE_DOCK: usize = 231;
    const ID_PAGE_APPLETS: usize = 232;
    const ID_PAGE_RECOVERY: usize = 233;
    const CURRENT_CONFIG_VERSION: u32 = 7;

    static mut PANEL: HWND = HWND(null_mut());
    static mut DOCK: HWND = HWND(null_mut());
    static mut DESKTOP: HWND = HWND(null_mut());
    static mut SHELLHOOK_MSG: u32 = 0;
    static mut KEYBOARD_HOOK: HHOOK = HHOOK(null_mut());
    static mut WIN_KEY_DOWN: bool = false;
    static mut WIN_COMBO_USED: bool = false;
    static mut SECONDARY_PANELS: [HWND; 8] = [HWND(null_mut()); 8];
    static mut SECONDARY_DOCKS: [HWND; 8] = [HWND(null_mut()); 8];
    static mut SECONDARY_APPBARS: [Option<APPBARDATA>; 8] = [None; 8];
    static mut SECONDARY_COUNT: usize = 0;
    static mut PRIMARY_APPBAR: Option<APPBARDATA> = None;
    static mut INSTANCE: HINSTANCE = HINSTANCE(null_mut());
    static mut SETTINGS: HWND = HWND(null_mut());
    static mut SETTINGS_INITIALIZED: bool = false;
    static mut SYNCING_SETTINGS: bool = false;
    static mut LAUNCHER: HWND = HWND(null_mut());
    static mut LAUNCHER_EDIT: HWND = HWND(null_mut());
    static mut SWITCHER: HWND = HWND(null_mut());
    static mut AUDIO_MIXER: HWND = HWND(null_mut());
    static mut AUDIO_MIXER_SELECTION: usize = 0;
    static mut OSD_WINDOW: HWND = HWND(null_mut());
    static mut OSD_KIND: OsdKind = OsdKind::Volume;
    static mut OSD_LEVEL: u8 = 0;
    static mut OSD_MUTED: bool = false;
    static mut CURRENT_BRIGHTNESS: u8 = 100;
    static mut POWER_NOTIFY: HPOWERNOTIFY = HPOWERNOTIFY(0);
    static mut REGISTERED_ENDPOINT: Option<IAudioEndpointVolume> = None;
    static mut DOCK_VISIBLE: bool = true;
    static mut HOVERED_DOCK_ICON: i32 = -1;
    static mut DOCK_POINTER_X: i32 = -1;
    static mut DOCK_MAGNIFICATION: f32 = 0.0;
    static mut DRAGGED_DOCK_ICON: i32 = -1;
    static mut DOCK_DRAG_MOVED: bool = false;
    static mut RUNNING_WINDOWS: usize = 0;
    static mut WINDOW_LIST: Vec<HWND> = Vec::new();
    static mut APP_INDEX: Option<Vec<(String, String)>> = None;
    static mut APP_INDEX_TICKS: u16 = 0;
    static mut NETWORK_TICKS: u8 = 4;
    static mut NETWORK_CONNECTED: bool = false;
    static mut NETWORK_WIFI: bool = false;
    static mut NETWORK_WIFI_DETAILS: Option<(String, u32)> = None;
    static mut LAUNCHER_SELECTION: i32 = 0;
    static mut MEDIA_MANAGER: Option<GlobalSystemMediaTransportControlsSessionManager> = None;
    static mut SWITCHER_SELECTION: usize = 0;
    static mut SWITCHER_THUMBNAIL: isize = 0;
    static mut SWITCHER_THUMBNAIL_SOURCE: HWND = HWND(null_mut());
    static mut DOCK_ICON_CACHE: [HICON; 5] = [HICON(null_mut()); 5];
    static mut SETTINGS_PAGE: usize = 0;
    static mut CONFIG: Config = Config {
        dark: true,
        show_clock: true,
        show_network: true,
        show_volume: true,
        show_battery: true,
        show_notifications: true,
        show_bluetooth: true,
        show_media: true,
        position: DockPosition::Center,
        auto_hide: false,
        icon_size: 25,
        magnification: 45,
        transparency: 238,
        spacing: 84,
        offset_x: 0,
        offset_y: 0,
        clock_24h: false,
        date_format: 1,
        desktop_enabled: true,
        applet_order: [0, 1, 2, 3, 4, 5, 6],
        panel_zones: [2, 2, 2, 2, 2, 2, 2],
        dock_order: [0, 1, 2, 3, 4],
    };

    #[derive(Clone, Copy, PartialEq)]
    enum DockPosition {
        Left,
        Center,
        Right,
    }
    #[derive(Clone, Copy)]
    struct Config {
        dark: bool,
        show_clock: bool,
        show_network: bool,
        show_volume: bool,
        show_battery: bool,
        show_notifications: bool,
        show_bluetooth: bool,
        show_media: bool,
        position: DockPosition,
        auto_hide: bool,
        icon_size: i32,
        magnification: u8,
        transparency: u8,
        spacing: i32,
        offset_x: i32,
        offset_y: i32,
        clock_24h: bool,
        date_format: u8,
        desktop_enabled: bool,
        applet_order: [u8; 7],
        panel_zones: [u8; 7],
        dock_order: [u8; 5],
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }
    fn brush(c: u32) -> HBRUSH {
        unsafe { CreateSolidBrush(COLORREF(c)) }
    }
    unsafe fn scaled(hwnd: HWND, value: i32) -> i32 {
        let dpi = GetDpiForWindow(hwnd).max(96);
        value.saturating_mul(dpi as i32) / 96
    }
    unsafe fn apply_dpi_suggested_rect(hwnd: HWND, l: LPARAM) {
        if l.0 == 0 {
            return;
        }
        let suggested = *(l.0 as *const RECT);
        let _ = SetWindowPos(
            hwnd,
            HWND_TOP,
            suggested.left,
            suggested.top,
            suggested.right - suggested.left,
            suggested.bottom - suggested.top,
            SWP_NOACTIVATE,
        );
    }
    fn high_contrast() -> bool {
        unsafe {
            let mut settings = HIGHCONTRASTW {
                cbSize: size_of::<HIGHCONTRASTW>() as u32,
                ..Default::default()
            };
            SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                settings.cbSize,
                Some(&mut settings as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .is_ok()
                && (settings.dwFlags.0 & HCF_HIGHCONTRASTON.0) != 0
        }
    }
    fn animations_enabled() -> bool {
        unsafe {
            let mut enabled = BOOL(1);
            SystemParametersInfoW(
                SPI_GETCLIENTAREAANIMATION,
                0,
                Some(&mut enabled as *mut BOOL as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .is_ok()
                && enabled.as_bool()
        }
    }
    fn config_path() -> PathBuf {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("LumaShell")
            .join("settings.ini")
    }
    fn config_backup_path() -> PathBuf {
        config_path().with_file_name("settings.ini.bak")
    }
    fn log_path() -> PathBuf {
        config_path().with_file_name("luma.log")
    }
    fn watchdog_log_path() -> PathBuf {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("LumaShell")
            .join("watchdog.log")
    }
    fn log_event(message: &str) {
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("D:\\project\\luma.log")
        {
            let _ = writeln!(f, "{:?} {}", std::time::SystemTime::now(), message);
        }
        if let Some(parent) = log_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::metadata(log_path())
            .map(|metadata| metadata.len() > 1_048_576)
            .unwrap_or(false)
        {
            let rotated = log_path().with_file_name("luma.log.1");
            let _ = fs::remove_file(&rotated);
            let _ = fs::rename(log_path(), rotated);
        }
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path())
        {
            let _ = writeln!(file, "{:?} {}", std::time::SystemTime::now(), message);
        }
    }
    fn startup_enabled() -> bool {
        let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let value_name = wide("LumaShell");
        let mut key = HKEY::default();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            )
        };
        if result.0 != 0 {
            return false;
        }
        let mut size = 0u32;
        let query = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                None,
                Some(&mut size),
            )
        };
        let enabled = query.0 == 0 && size > 0;
        unsafe {
            RegCloseKey(key);
        }
        enabled
    }
    fn set_startup_enabled(enabled: bool) -> bool {
        let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        let value_name = wide("LumaShell");
        let mut key = HKEY::default();
        if enabled {
            let result = unsafe {
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey.as_ptr()),
                    0,
                    PCWSTR::null(),
                    REG_OPEN_CREATE_OPTIONS(0),
                    KEY_WRITE,
                    None,
                    &mut key,
                    None,
                )
            };
            if result.0 != 0 {
                return false;
            }
            let Ok(exe) = env::current_exe() else {
                unsafe { RegCloseKey(key) };
                return false;
            };
            let command = wide(&format!("\"{}\"", exe.display()));
            let bytes = unsafe {
                std::slice::from_raw_parts(command.as_ptr() as *const u8, command.len() * 2)
            };
            let result =
                unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes)) };
            unsafe { RegCloseKey(key) };
            result.0 == 0
        } else {
            let result = unsafe {
                RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(subkey.as_ptr()),
                    0,
                    KEY_SET_VALUE,
                    &mut key,
                )
            };
            if result.0 != 0 {
                return true;
            }
            let result = unsafe { RegDeleteValueW(key, PCWSTR(value_name.as_ptr())) };
            unsafe { RegCloseKey(key) };
            result.0 == 0
        }
    }
    fn parse_order(raw: &str) -> [u8; 7] {
        let mut result = [0u8; 7];
        let mut used = [false; 7];
        let mut count = 0usize;
        for token in raw.split(',') {
            let id = match token.trim() {
                "clock" => 0,
                "network" => 1,
                "volume" => 2,
                "bluetooth" => 3,
                "battery" => 4,
                "notifications" => 5,
                "media" => 6,
                _ => continue,
            };
            if !used[id] {
                result[count] = id as u8;
                used[id] = true;
                count += 1;
            }
            if count == 7 {
                break;
            }
        }
        for (id, &is_used) in used.iter().enumerate() {
            if !is_used {
                result[count] = id as u8;
                count += 1;
            }
        }
        result
    }
    fn order_text(order: [u8; 7]) -> String {
        order
            .iter()
            .map(|id| match id {
                0 => "clock",
                1 => "network",
                2 => "volume",
                3 => "bluetooth",
                4 => "battery",
                5 => "notifications",
                _ => "media",
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    fn parse_zones(raw: &str) -> [u8; 7] {
        let mut result = [2u8; 7];
        for token in raw.split(',') {
            let mut pair = token.trim().splitn(2, [':', '=']);
            let Some(name) = pair.next() else { continue };
            let Some(zone) = pair.next() else { continue };
            let id = match name.trim() {
                "clock" => 0,
                "network" => 1,
                "volume" => 2,
                "bluetooth" => 3,
                "battery" => 4,
                "notifications" => 5,
                "media" => 6,
                _ => continue,
            };
            result[id] = match zone.trim() {
                "left" => 0,
                "center" => 1,
                _ => 2,
            };
        }
        result
    }
    fn zones_text(zones: [u8; 7]) -> String {
        (0..7)
            .map(|id| {
                let name = match id {
                    0 => "clock",
                    1 => "network",
                    2 => "volume",
                    3 => "bluetooth",
                    4 => "battery",
                    5 => "notifications",
                    _ => "media",
                };
                let zone = match zones[id] {
                    0 => "left",
                    1 => "center",
                    _ => "right",
                };
                format!("{}:{}", name, zone)
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    fn parse_dock_order(raw: &str) -> [u8; 5] {
        let mut result = [0u8; 5];
        let mut used = [false; 5];
        let mut count = 0usize;
        for token in raw.split(',') {
            let id = match token.trim() {
                "home" => 0,
                "files" => 1,
                "terminal" => 2,
                "browser" => 3,
                "settings" => 4,
                _ => continue,
            };
            if !used[id] {
                result[count] = id as u8;
                used[id] = true;
                count += 1;
            }
            if count == 5 {
                break;
            }
        }
        for (id, &is_used) in used.iter().enumerate() {
            if !is_used {
                result[count] = id as u8;
                count += 1;
            }
        }
        result
    }
    fn dock_order_text(order: [u8; 5]) -> String {
        order
            .iter()
            .map(|id| match id {
                0 => "home",
                1 => "files",
                2 => "terminal",
                3 => "browser",
                _ => "settings",
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    fn load_config() -> Config {
        let mut c = unsafe { CONFIG };
        let mut version = 1u32;
        let raw =
            fs::read_to_string(config_path()).or_else(|_| fs::read_to_string(config_backup_path()));
        if let Ok(raw) = raw {
            for line in raw.lines() {
                let mut it = line.splitn(2, '=');
                let k = it.next().unwrap_or("");
                let v = it.next().unwrap_or("");
                match k {
                    "version" => version = v.parse().unwrap_or(1),
                    "dark" => c.dark = v == "true",
                    "clock" => c.show_clock = v == "true",
                    "network" => c.show_network = v == "true",
                    "volume" => c.show_volume = v == "true",
                    "battery" => c.show_battery = v == "true",
                    "notifications" => c.show_notifications = v == "true",
                    "bluetooth" => c.show_bluetooth = v == "true",
                    "media" => c.show_media = v == "true",
                    "position" => {
                        c.position = match v {
                            "left" => DockPosition::Left,
                            "right" => DockPosition::Right,
                            _ => DockPosition::Center,
                        }
                    }
                    "auto_hide" => c.auto_hide = v == "true",
                    "icon_size" => c.icon_size = v.parse().unwrap_or(25).clamp(20, 32),
                    "magnification" => {
                        c.magnification = v.parse().unwrap_or(45).clamp(0, 100) as u8
                    }
                    "transparency" => {
                        c.transparency = v.parse().unwrap_or(238).clamp(120, 255) as u8
                    }
                    "spacing" => c.spacing = v.parse().unwrap_or(84).clamp(72, 96),
                    "offset_x" => c.offset_x = v.parse().unwrap_or(0).clamp(-200, 200),
                    "offset_y" => c.offset_y = v.parse().unwrap_or(0).clamp(-100, 100),
                    "clock_24h" => c.clock_24h = v == "true",
                    "desktop" => c.desktop_enabled = v == "true",
                    "date_format" => {
                        c.date_format = match v {
                            "none" => 0,
                            "long" => 2,
                            _ => 1,
                        }
                    }
                    "order" => c.applet_order = parse_order(v),
                    "zones" => c.panel_zones = parse_zones(v),
                    "dock_order" => c.dock_order = parse_dock_order(v),
                    _ => {}
                }
            }
            if version > CURRENT_CONFIG_VERSION {
                log_event("unsupported future configuration version; defaults loaded");
                return default_config();
            }
        }
        c
    }
    fn save_config() {
        let c = unsafe { CONFIG };
        let position = match c.position {
            DockPosition::Left => "left",
            DockPosition::Right => "right",
            DockPosition::Center => "center",
        };
        let text = format!(
            "version={}\ndark={}\ndesktop={}\nclock={}\nnetwork={}\nvolume={}\nbattery={}\nnotifications={}\nbluetooth={}\nmedia={}\nposition={}\nauto_hide={}\nicon_size={}\nmagnification={}\ntransparency={}\nspacing={}\noffset_x={}\noffset_y={}\nclock_24h={}\ndate_format={}\norder={}\nzones={}\ndock_order={}\n",
            CURRENT_CONFIG_VERSION,
            c.dark,
            c.desktop_enabled,
            c.show_clock,
            c.show_network,
            c.show_volume,
            c.show_battery,
            c.show_notifications,
            c.show_bluetooth,
            c.show_media,
            position,
            c.auto_hide,
            c.icon_size,
            c.magnification,
            c.transparency,
            c.spacing,
            c.offset_x,
            c.offset_y,
            c.clock_24h,
            match c.date_format {
                0 => "none",
                2 => "long",
                _ => "short",
            },
            order_text(c.applet_order),
            zones_text(c.panel_zones),
            dock_order_text(c.dock_order)
        );
        if let Some(parent) = config_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        let path = config_path();
        let temp = path.with_file_name("settings.ini.tmp");
        let backup = config_backup_path();
        if fs::write(&temp, text).is_err() {
            log_event("configuration write failed");
            return;
        }
        if path.exists() {
            let _ = fs::copy(&path, &backup);
        }
        let _ = fs::remove_file(&path);
        if fs::rename(&temp, &path).is_err() {
            if backup.exists() {
                let _ = fs::copy(&backup, &path);
            }
            let _ = fs::remove_file(&temp);
            log_event("configuration commit failed; backup restored");
            return;
        }
        log_event("configuration saved with backup");
    }
    fn restore_config_backup() -> bool {
        let path = config_path();
        let backup = config_backup_path();
        if !backup.exists() {
            log_event("configuration backup restore requested but no backup exists");
            return false;
        }
        let temp = path.with_file_name("settings.ini.restore.tmp");
        if fs::copy(&backup, &temp).is_err() {
            log_event("configuration backup staging failed");
            return false;
        }
        let previous = path.with_file_name("settings.ini.restore-old");
        if path.exists() && fs::copy(&path, &previous).is_err() {
            let _ = fs::remove_file(&temp);
            log_event("configuration pre-restore snapshot failed");
            return false;
        }
        let _ = fs::remove_file(&path);
        if fs::rename(&temp, &path).is_err() {
            if previous.exists() {
                let _ = fs::copy(&previous, &path);
            }
            let _ = fs::remove_file(&temp);
            let _ = fs::remove_file(&previous);
            log_event("configuration backup restore commit failed");
            return false;
        }
        let _ = fs::remove_file(&previous);
        log_event("configuration backup restored");
        true
    }
    fn default_config() -> Config {
        Config {
            dark: true,
            show_clock: true,
            show_network: true,
            show_volume: true,
            show_battery: true,
            show_notifications: true,
            show_bluetooth: true,
            show_media: true,
            position: DockPosition::Center,
            auto_hide: false,
            icon_size: 25,
            magnification: 45,
            transparency: 238,
            spacing: 84,
            offset_x: 0,
            offset_y: 0,
            clock_24h: false,
            date_format: 1,
            desktop_enabled: true,
            applet_order: [0, 1, 2, 3, 4, 5, 6],
            panel_zones: [2, 2, 2, 2, 2, 2, 2],
            dock_order: [0, 1, 2, 3, 4],
        }
    }
    static mut DESKTOP_SHOWING: bool = false;
    static mut MINIMIZED_WINDOWS: Vec<HWND> = Vec::new();

    unsafe fn toggle_show_desktop() {
        if DESKTOP_SHOWING {
            for &hwnd in &MINIMIZED_WINDOWS {
                if IsWindow(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
            }
            MINIMIZED_WINDOWS.clear();
            DESKTOP_SHOWING = false;
        } else {
            MINIMIZED_WINDOWS.clear();
            unsafe extern "system" fn enum_proc(hwnd: HWND, _: LPARAM) -> BOOL {
                if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
                    if hwnd != PANEL
                        && hwnd != DOCK
                        && hwnd != DESKTOP
                        && hwnd != SETTINGS
                        && hwnd != LAUNCHER
                        && hwnd != SWITCHER
                        && hwnd != AUDIO_MIXER
                    {
                        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
                        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                        if (style & WS_VISIBLE.0 != 0) && (ex_style & WS_EX_TOOLWINDOW.0 == 0) {
                            MINIMIZED_WINDOWS.push(hwnd);
                            let _ = ShowWindow(hwnd, SW_MINIMIZE);
                        }
                    }
                }
                BOOL(1)
            }
            let _ = EnumWindows(Some(enum_proc), LPARAM(0));
            DESKTOP_SHOWING = true;
        }
    }

    unsafe fn hide_windows_shell_windows() {
        let class = wide("Shell_TrayWnd");
        if let Ok(tray) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            if !tray.is_invalid() && IsWindowVisible(tray).as_bool() {
                let _ = ShowWindow(tray, SW_HIDE);
            }
        }
        let class_sec = wide("Shell_SecondaryTrayWnd");
        let mut prev = HWND(null_mut());
        loop {
            match FindWindowExW(HWND(null_mut()), prev, PCWSTR(class_sec.as_ptr()), PCWSTR::null()) {
                Ok(sec) if !sec.is_invalid() => {
                    if IsWindowVisible(sec).as_bool() {
                        let _ = ShowWindow(sec, SW_HIDE);
                    }
                    prev = sec;
                }
                _ => break,
            }
        }
        let progman = wide("Progman");
        if let Ok(prog) = FindWindowW(PCWSTR(progman.as_ptr()), PCWSTR::null()) {
            if !prog.is_invalid() && IsWindowVisible(prog).as_bool() {
                let _ = ShowWindow(prog, SW_HIDE);
            }
        }
    }

    unsafe fn show_windows_shell_windows() {
        let class = wide("Shell_TrayWnd");
        if let Ok(tray) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            if !tray.is_invalid() {
                let _ = ShowWindow(tray, SW_SHOW);
            }
        }
        let class_sec = wide("Shell_SecondaryTrayWnd");
        let mut prev = HWND(null_mut());
        loop {
            match FindWindowExW(HWND(null_mut()), prev, PCWSTR(class_sec.as_ptr()), PCWSTR::null()) {
                Ok(sec) if !sec.is_invalid() => {
                    let _ = ShowWindow(sec, SW_SHOW);
                    prev = sec;
                }
                _ => break,
            }
        }
        let progman = wide("Progman");
        if let Ok(prog) = FindWindowW(PCWSTR(progman.as_ptr()), PCWSTR::null()) {
            if !prog.is_invalid() {
                let _ = ShowWindow(prog, SW_SHOW);
            }
        }
    }

    fn is_luma_shell_active() -> bool {
        let subkey = wide("Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon");
        let mut key = HKEY::default();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                KEY_READ,
                &mut key,
            )
        };
        if result.0 != 0 {
            return false;
        }
        let value_name = wide("Shell");
        let mut buf = [0u16; 512];
        let mut size = (buf.len() * 2) as u32;
        let mut kind = REG_SZ;
        let query = unsafe {
            RegQueryValueExW(
                key,
                PCWSTR(value_name.as_ptr()),
                None,
                Some(&mut kind),
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut size),
            )
        };
        unsafe { RegCloseKey(key) };
        if query.0 != 0 || size == 0 {
            return false;
        }
        let len = (size as usize / 2).saturating_sub(1);
        let shell_str = String::from_utf16_lossy(&buf[..len]).trim().to_lowercase();
        if shell_str == "explorer.exe" || shell_str.is_empty() {
            return false;
        }
        if let Ok(current_exe) = env::current_exe() {
            let cp = current_exe.to_string_lossy().to_lowercase();
            if shell_str.contains(&cp) {
                return true;
            }
        }
        let raw_clean = shell_str.trim_matches('"');
        if Path::new(raw_clean).exists() {
            return true;
        }
        shell_str.contains("watchdog.ps1")
    }

    unsafe fn is_explorer_running() -> bool {
        let class = wide("Shell_TrayWnd");
        if let Ok(hwnd) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            if !hwnd.is_invalid() {
                return true;
            }
        }
        false
    }

    unsafe fn kill_explorer() {
        log_event("terminating explorer.exe");
        let class = wide("Shell_TrayWnd");
        if let Ok(hwnd) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
            if !hwnd.is_invalid() {
                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid != 0 {
                    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                        let _ = TerminateProcess(handle, 1);
                        let _ = CloseHandle(handle);
                        log_event("terminated explorer.exe via Shell_TrayWnd");
                    }
                }
            }
        }
        hide_windows_shell_windows();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    fn enable_luma_shell_replacement() -> bool {
        let Ok(exe) = env::current_exe() else {
            return false;
        };
        let local_app_data = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir);
        let root = local_app_data.join("LumaShell");
        let _ = fs::create_dir_all(&root);
        let backup_path = root.join("explorer-shell.backup.json");
        if !backup_path.exists() {
            let backup_content = r#"{"Version":1,"Key":"HKCU:\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon","Shell":"explorer.exe"}"#;
            let _ = fs::write(&backup_path, backup_content);
        }

        let watchdog = root.join("watchdog.ps1");
        let recovery = root.join("restore-explorer.ps1");
        let shell_cmd = if watchdog.exists() && recovery.exists() {
            format!(
                "powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File \"{}\" -BinaryPath \"{}\" -RecoveryScriptPath \"{}\" -ShellBackupPath \"{}\" -RestoreShellOnExit",
                watchdog.display(),
                exe.display(),
                recovery.display(),
                backup_path.display()
            )
        } else {
            format!("\"{}\"", exe.display())
        };

        let subkey = wide("Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon");
        let mut key = HKEY::default();
        let result = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                PCWSTR::null(),
                REG_OPEN_CREATE_OPTIONS(0),
                KEY_WRITE,
                None,
                &mut key,
                None,
            )
        };
        if result.0 != 0 {
            return false;
        }
        let value_name = wide("Shell");
        let cmd_wide = wide(&shell_cmd);
        let bytes = unsafe {
            std::slice::from_raw_parts(cmd_wide.as_ptr() as *const u8, cmd_wide.len() * 2)
        };
        let set_res =
            unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes)) };
        let auto_restart_name = wide("AutoRestartShell");
        let zero: u32 = 0;
        let zero_bytes = unsafe {
            std::slice::from_raw_parts(&zero as *const u32 as *const u8, 4)
        };
        let _ = unsafe {
            RegSetValueExW(
                key,
                PCWSTR(auto_restart_name.as_ptr()),
                0,
                REG_DWORD,
                Some(zero_bytes),
            )
        };
        unsafe { RegCloseKey(key) };
        log_event(&format!("shell replacement registered: {}", shell_cmd));
        set_res.0 == 0
    }

    fn restore_explorer_shell() -> bool {
        let subkey = wide("Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon");
        let mut key = HKEY::default();
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                0,
                KEY_SET_VALUE,
                &mut key,
            )
        };
        if result.0 == 0 {
            let value_name = wide("Shell");
            let cmd_wide = wide("explorer.exe");
            let bytes = unsafe {
                std::slice::from_raw_parts(cmd_wide.as_ptr() as *const u8, cmd_wide.len() * 2)
            };
            let _ =
                unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes)) };
            let auto_restart_name = wide("AutoRestartShell");
            let _ = unsafe { RegDeleteValueW(key, PCWSTR(auto_restart_name.as_ptr())) };
            unsafe { RegCloseKey(key) };
        }
        log_event("explorer shell restored in registry");
        unsafe { show_windows_shell_windows(); }
        launch("explorer.exe");
        true
    }

    fn launch(target: &str) {
        record_recent(target);
        if target == "wt.exe" {
            if let Some(local_app) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
                let wt_path = local_app.join("Microsoft\\WindowsApps\\wt.exe");
                if wt_path.exists() {
                    let v = wide(&wt_path.to_string_lossy());
                    unsafe {
                        let _ = ShellExecuteW(
                            None,
                            PCWSTR::null(),
                            PCWSTR(v.as_ptr()),
                            PCWSTR::null(),
                            PCWSTR::null(),
                            SW_SHOWNORMAL,
                        );
                    }
                    return;
                }
            }
            let v = wide("powershell.exe");
            unsafe {
                let _ = ShellExecuteW(
                    None,
                    PCWSTR::null(),
                    PCWSTR(v.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );
            }
            return;
        }
        if target == "shell:Home" {
            let user_dir = env::var_os("USERPROFILE").unwrap_or_default();
            let v = wide(&user_dir.to_string_lossy());
            unsafe {
                let _ = ShellExecuteW(
                    None,
                    PCWSTR::null(),
                    PCWSTR(v.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );
            }
            return;
        }
        let v = wide(target);
        unsafe {
            let _ = ShellExecuteW(
                None,
                PCWSTR::null(),
                PCWSTR(v.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
    }
    fn restore_explorer() {
        let _ = restore_explorer_shell();
        let Some(local_app_data) = env::var_os("LOCALAPPDATA") else {
            return;
        };
        let root = PathBuf::from(local_app_data).join("LumaShell");
        let script = root.join("restore-explorer.ps1");
        let backup = root.join("explorer-shell.backup.json");
        if !script.exists() {
            return;
        }
        let args = format!(
            "-NoProfile -ExecutionPolicy Bypass -File \"{}\" -ShellBackupPath \"{}\"",
            script.display(),
            backup.display()
        );
        let verb = wide("open");
        let file = wide("powershell.exe");
        let parameters = wide(&args);
        unsafe {
            let _ = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(file.as_ptr()),
                PCWSTR(parameters.as_ptr()),
                PCWSTR::null(),
                SW_HIDE,
            );
        }
    }

    fn collect_apps(dir: PathBuf, items: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_apps(path, items);
            } else if matches!(
                path.extension().and_then(|x| x.to_str()),
                Some("lnk") | Some("url") | Some("exe")
            ) {
                let name = path
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .unwrap_or("App")
                    .replace('_', " ");
                items.push((name, path.to_string_lossy().into_owned()));
            }
        }
    }

    fn build_app_index() -> Vec<(String, String)> {
        let mut items = Vec::new();
        if let Some(appdata) = env::var_os("APPDATA") {
            collect_apps(
                PathBuf::from(appdata).join("Microsoft/Windows/Start Menu/Programs"),
                &mut items,
            );
        }
        if let Some(program_data) = env::var_os("ProgramData") {
            collect_apps(
                PathBuf::from(program_data).join("Microsoft/Windows/Start Menu/Programs"),
                &mut items,
            );
        }
        for (name, target) in [
            ("Wi-Fi settings", "ms-settings:network-wifi"),
            ("Sound settings", "ms-settings:sound"),
            ("Display settings", "ms-settings:display"),
            ("Bluetooth settings", "ms-settings:bluetooth"),
            ("Notifications settings", "ms-settings:notifications"),
            ("Personalization settings", "ms-settings:personalization"),
            ("Windows Update", "ms-settings:windowsupdate"),
        ] {
            items.push((name.to_string(), target.to_string()));
        }
        items.sort_by_key(|(name, _)| name.to_lowercase());
        items.dedup_by(|a, b| a.0.eq_ignore_ascii_case(&b.0));
        items
    }

    fn collect_matching_files(
        dir: &Path,
        query: &str,
        depth: u8,
        budget: &mut usize,
        items: &mut Vec<(String, String)>,
    ) {
        if depth > 3 || *budget == 0 {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            if *budget == 0 {
                break;
            }
            *budget -= 1;
            let path = entry.path();
            if path.is_dir() {
                collect_matching_files(&path, query, depth + 1, budget, items);
            } else if path
                .file_name()
                .and_then(|value| value.to_str())
                .map(|name| name.to_lowercase().contains(query))
                .unwrap_or(false)
            {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("File");
                items.push((
                    format!("File · {}", name),
                    path.to_string_lossy().into_owned(),
                ));
                if items.len() >= 20 {
                    *budget = 0;
                }
            }
        }
    }

    fn search_user_files(query: &str) -> Vec<(String, String)> {
        let Some(profile) = env::var_os("USERPROFILE") else {
            return Vec::new();
        };
        let profile = PathBuf::from(profile);
        let mut items = Vec::new();
        let mut budget = 1500usize;
        for folder in ["Desktop", "Documents", "Downloads"] {
            collect_matching_files(&profile.join(folder), query, 0, &mut budget, &mut items);
            if budget == 0 {
                break;
            }
        }
        items
    }

    fn recent_path() -> PathBuf {
        config_path().with_file_name("recent.txt")
    }
    fn favorites_path() -> PathBuf {
        config_path().with_file_name("favorites.txt")
    }
    fn favorite_targets() -> Vec<String> {
        fs::read_to_string(favorites_path())
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect()
    }
    fn is_favorite(target: &str) -> bool {
        favorite_targets()
            .iter()
            .any(|item| item.eq_ignore_ascii_case(target))
    }
    fn toggle_favorite(target: &str) {
        let mut items = favorite_targets();
        if let Some(index) = items
            .iter()
            .position(|item| item.eq_ignore_ascii_case(target))
        {
            items.remove(index);
        } else {
            items.insert(0, target.to_string());
            items.truncate(32);
        }
        if let Some(parent) = favorites_path().parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(favorites_path(), items.join("\n"));
    }
    fn record_recent(target: &str) {
        let path = recent_path();
        let mut items = fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|item| !item.is_empty() && !item.eq_ignore_ascii_case(target))
            .map(str::to_string)
            .collect::<Vec<_>>();
        items.insert(0, target.to_string());
        items.truncate(12);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, items.join("\n"));
    }

    unsafe fn launcher_query() -> String {
        let len = GetWindowTextLengthW(LAUNCHER_EDIT);
        if len <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(LAUNCHER_EDIT, &mut buffer);
        String::from_utf16_lossy(&buffer[..copied as usize]).to_lowercase()
    }

    unsafe fn launcher_entries(query: &str) -> Vec<(String, String)> {
        let Some(index) = &APP_INDEX else {
            return Vec::new();
        };
        let mut result = Vec::new();
        if query.is_empty() {
            for target in favorite_targets() {
                let found = index
                    .iter()
                    .find(|item| item.1.eq_ignore_ascii_case(&target));
                if let Some(item) = found.filter(|item| {
                    !result
                        .iter()
                        .any(|existing: &(String, String)| existing.1 == item.1)
                }) {
                    result.push(item.clone());
                }
            }
            for target in fs::read_to_string(recent_path())
                .unwrap_or_default()
                .lines()
                .map(str::trim)
            {
                let found = index
                    .iter()
                    .find(|item| item.1.eq_ignore_ascii_case(target));
                if let Some(item) = found.filter(|item| {
                    !result
                        .iter()
                        .any(|existing: &(String, String)| existing.1 == item.1)
                }) {
                    result.push(item.clone());
                }
            }
        }
        for item in index
            .iter()
            .filter(|item| query.is_empty() || item.0.to_lowercase().contains(query))
        {
            if !result
                .iter()
                .any(|existing: &(String, String)| existing.1 == item.1)
            {
                result.push(item.clone());
            }
        }
        if query.len() >= 2 {
            for item in search_user_files(query) {
                if !result
                    .iter()
                    .any(|existing: &(String, String)| existing.1 == item.1)
                {
                    result.push(item);
                }
            }
        }
        result
    }

    unsafe fn acquire_instance() -> Option<HANDLE> {
        let name = wide("Local\\LumaShell.SingleInstance");
        let handle = CreateMutexW(None, true, PCWSTR(name.as_ptr())).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            log_event("duplicate instance prevented");
            let class_name = wide("LumaPanel");
            if let Ok(panel) = FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) {
                ShowWindow(panel, SW_SHOW);
                SetForegroundWindow(panel);
            }
            let dock_name = wide("LumaDock");
            if let Ok(dock) = FindWindowW(PCWSTR(dock_name.as_ptr()), PCWSTR::null()) {
                ShowWindow(dock, SW_SHOW);
            }
            None
        } else {
            Some(handle)
        }
    }

    unsafe fn style_window(hwnd: HWND) {
        let preference = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const _,
            size_of::<i32>() as u32,
        );
        let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, current_style | WS_EX_LAYERED.0 as isize);
        let alpha = if high_contrast() {
            255
        } else {
            CONFIG.transparency
        };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
        let backdrop = DWMSBT_TRANSIENTWINDOW;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const _ as *const _,
            size_of::<i32>() as u32,
        );
        let dark_mode = if CONFIG.dark { 1i32 } else { 0i32 };
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as *const _,
            size_of::<i32>() as u32,
        );
    }
    unsafe fn restyle_shell_windows() {
        style_window(PANEL);
        style_window(DOCK);
        let dark: BOOL = CONFIG.dark.into();
        let _ = DwmSetWindowAttribute(
            SETTINGS,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const _ as *const _,
            size_of::<BOOL>() as u32,
        );
        style_window(LAUNCHER);
        style_window(SWITCHER);
        style_window(AUDIO_MIXER);
        if !OSD_WINDOW.is_invalid() {
            style_window(OSD_WINDOW);
        }
        for index in 0..SECONDARY_COUNT {
            style_window(SECONDARY_PANELS[index]);
            style_window(SECONDARY_DOCKS[index]);
        }
    }

    unsafe fn register_panel_appbar_for_rect(hwnd: HWND, monitor: RECT) -> APPBARDATA {
        let panel_height = scaled(hwnd, PANEL_HEIGHT);
        let mut data = APPBARDATA {
            cbSize: size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: WM_APP + 40,
            uEdge: ABE_TOP,
            rc: RECT {
                left: monitor.left,
                top: monitor.top,
                right: monitor.right,
                bottom: monitor.top + panel_height,
            },
            lParam: LPARAM(0),
        };
        let _ = SHAppBarMessage(ABM_NEW, &mut data);
        let _ = SHAppBarMessage(ABM_QUERYPOS, &mut data);
        data.rc.top = monitor.top;
        data.rc.bottom = monitor.top + panel_height;
        data.rc.left = monitor.left;
        data.rc.right = monitor.right;
        let _ = SHAppBarMessage(ABM_SETPOS, &mut data);
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            data.rc.left,
            data.rc.top,
            data.rc.right - data.rc.left,
            data.rc.bottom - data.rc.top,
            SWP_SHOWWINDOW,
        );
        let mut work_area = RECT {
            left: monitor.left,
            top: monitor.top + panel_height,
            right: monitor.right,
            bottom: monitor.bottom,
        };
        let mut cur = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut cur as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        if cur.top != work_area.top
            || cur.bottom != work_area.bottom
            || cur.left != work_area.left
            || cur.right != work_area.right
        {
            let _ = SystemParametersInfoW(
                SPI_SETWORKAREA,
                0,
                Some(&mut work_area as *mut RECT as *mut _),
                SPIF_SENDCHANGE,
            );
        }
        data
    }

    unsafe fn unregister_panel_appbar(data: &mut APPBARDATA) {
        let _ = SHAppBarMessage(ABM_REMOVE, data);
        let monitor = MonitorFromWindow(data.hWnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let mut full_work_area = info.rcMonitor;
            let mut cur = RECT::default();
            let _ = SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut cur as *mut RECT as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
            if cur.top != full_work_area.top
                || cur.bottom != full_work_area.bottom
                || cur.left != full_work_area.left
                || cur.right != full_work_area.right
            {
                let _ = SystemParametersInfoW(
                    SPI_SETWORKAREA,
                    0,
                    Some(&mut full_work_area as *mut RECT as *mut _),
                    SPIF_SENDCHANGE,
                );
            }
        }
    }

    static mut REPOSITIONING_PRIMARY: bool = false;
    unsafe fn reposition_primary_panel() {
        if REPOSITIONING_PRIMARY {
            return;
        }
        REPOSITIONING_PRIMARY = true;
        let Some(data) = PRIMARY_APPBAR.as_mut() else {
            REPOSITIONING_PRIMARY = false;
            return;
        };
        let monitor = MonitorFromWindow(PANEL, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            REPOSITIONING_PRIMARY = false;
            return;
        }
        let panel_height = scaled(PANEL, PANEL_HEIGHT);
        data.rc = RECT {
            left: info.rcMonitor.left,
            top: info.rcMonitor.top,
            right: info.rcMonitor.right,
            bottom: info.rcMonitor.top + panel_height,
        };
        let _ = SHAppBarMessage(ABM_QUERYPOS, data);
        data.rc.top = info.rcMonitor.top;
        data.rc.bottom = info.rcMonitor.top + panel_height;
        data.rc.left = info.rcMonitor.left;
        data.rc.right = info.rcMonitor.right;
        let _ = SHAppBarMessage(ABM_SETPOS, data);
        let _ = SetWindowPos(
            PANEL,
            HWND_TOPMOST,
            data.rc.left,
            data.rc.top,
            data.rc.right - data.rc.left,
            data.rc.bottom - data.rc.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        let mut work_area = RECT {
            left: info.rcMonitor.left,
            top: info.rcMonitor.top + panel_height,
            right: info.rcMonitor.right,
            bottom: info.rcMonitor.bottom,
        };
        let mut cur = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut cur as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        if cur.top != work_area.top
            || cur.bottom != work_area.bottom
            || cur.left != work_area.left
            || cur.right != work_area.right
        {
            let _ = SystemParametersInfoW(
                SPI_SETWORKAREA,
                0,
                Some(&mut work_area as *mut RECT as *mut _),
                SPIF_SENDCHANGE,
            );
        }
        REPOSITIONING_PRIMARY = false;
    }

    unsafe fn text_with_face(
        hdc: HDC,
        value: &str,
        x: i32,
        y: i32,
        color: u32,
        size: i32,
        face: &str,
    ) {
        let font = CreateFontW(
            size,
            0,
            0,
            0,
            500,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            0,
            0,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            PCWSTR(wide(face).as_ptr()),
        );
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(color));
        let v = wide(value);
        let _ = TextOutW(hdc, x, y, &v[..v.len() - 1]);
        SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }

    unsafe fn text(hdc: HDC, value: &str, x: i32, y: i32, color: u32, size: i32) {
        text_with_face(hdc, value, x, y, color, size, "Segoe UI");
    }

    unsafe fn icon_text(hdc: HDC, value: &str, x: i32, y: i32, color: u32, size: i32) {
        text_with_face(hdc, value, x, y, color, size, "Segoe MDL2 Assets");
    }

    unsafe fn default_audio_endpoint() -> Option<IAudioEndpointVolume> {
        let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&class_id, None, CLSCTX_ALL).ok()?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .ok()?;
        device.Activate(CLSCTX_ALL, None).ok()
    }

    unsafe fn active_audio_session_count() -> u32 {
        let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&class_id, None, CLSCTX_ALL) {
            Ok(value) => value,
            Err(_) => return 0,
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
            return 0;
        };
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
            return 0;
        };
        let Ok(sessions) = manager.GetSessionEnumerator() else {
            return 0;
        };
        let Ok(count) = sessions.GetCount() else {
            return 0;
        };
        (0..count)
            .filter_map(|index| sessions.GetSession(index).ok())
            .filter_map(|session| session.GetState().ok())
            .filter(|state| *state == AudioSessionStateActive)
            .count() as u32
    }

    struct AudioSessionRow {
        label: String,
        volume: u8,
        muted: bool,
    }

    unsafe fn process_label(pid: u32) -> String {
        if pid == 0 {
            return "System audio".to_string();
        }
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return format!("Process {}", pid);
        };
        let mut buffer = vec![0u16; 512];
        let mut length = buffer.len() as u32;
        let label = if QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
        .is_ok()
        {
            PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]))
                .file_stem()
                .and_then(|value| value.to_str())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        } else {
            None
        };
        let _ = CloseHandle(process);
        label.unwrap_or_else(|| format!("Process {}", pid))
    }

    unsafe fn audio_session_controls() -> Vec<(IAudioSessionControl2, ISimpleAudioVolume)> {
        let class_id = GUID::from_u128(0xbcde0395_e52f_467c_8e3d_c4579291692e);
        let Ok(enumerator): Result<IMMDeviceEnumerator, _> =
            CoCreateInstance(&class_id, None, CLSCTX_ALL)
        else {
            return Vec::new();
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) else {
            return Vec::new();
        };
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
            return Vec::new();
        };
        let Ok(sessions) = manager.GetSessionEnumerator() else {
            return Vec::new();
        };
        let Ok(count) = sessions.GetCount() else {
            return Vec::new();
        };
        (0..count)
            .filter_map(|index| sessions.GetSession(index).ok())
            .filter_map(|session| {
                if session.GetState().ok()? != AudioSessionStateActive {
                    return None;
                }
                let control = session.cast::<IAudioSessionControl2>().ok()?;
                let volume = session.cast::<ISimpleAudioVolume>().ok()?;
                Some((control, volume))
            })
            .collect()
    }

    unsafe fn audio_session_rows() -> Vec<AudioSessionRow> {
        audio_session_controls()
            .into_iter()
            .map(|(control, volume)| {
                let pid = control.GetProcessId().unwrap_or(0);
                let level = (volume.GetMasterVolume().unwrap_or(0.0) * 100.0)
                    .round()
                    .clamp(0.0, 100.0) as u8;
                let muted = volume
                    .GetMute()
                    .map(|value| value.as_bool())
                    .unwrap_or(false);
                AudioSessionRow {
                    label: process_label(pid),
                    volume: level,
                    muted,
                }
            })
            .collect()
    }

    unsafe fn toggle_audio_session(index: usize) -> bool {
        let Some((_, volume)) = audio_session_controls().into_iter().nth(index) else {
            return false;
        };
        let Ok(muted) = volume.GetMute() else {
            return false;
        };
        let context = GUID::from_u128(0);
        volume.SetMute(!muted.as_bool(), &context).is_ok()
    }

    unsafe fn adjust_audio_session(index: usize, delta: f32) -> bool {
        let Some((_, volume)) = audio_session_controls().into_iter().nth(index) else {
            return false;
        };
        let Ok(current) = volume.GetMasterVolume() else {
            return false;
        };
        let context = GUID::from_u128(0);
        volume
            .SetMasterVolume((current + delta).clamp(0.0, 1.0), &context)
            .is_ok()
    }

    unsafe fn default_audio_state() -> Option<(u8, bool)> {
        let endpoint = default_audio_endpoint()?;
        let level = (endpoint.GetMasterVolumeLevelScalar().ok()? * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8;
        let muted = endpoint.GetMute().ok()?.as_bool();
        Some((level, muted))
    }

    unsafe fn toggle_audio_mute() -> bool {
        let Some(endpoint) = default_audio_endpoint() else {
            return false;
        };
        let Ok(muted) = endpoint.GetMute() else {
            return false;
        };
        let context = GUID::from_u128(0);
        endpoint.SetMute(!muted.as_bool(), &context).is_ok()
    }

    unsafe fn adjust_master_volume(delta: f32) -> bool {
        let Some(endpoint) = default_audio_endpoint() else {
            return false;
        };
        let Ok(current) = endpoint.GetMasterVolumeLevelScalar() else {
            return false;
        };
        let target = (current + delta).clamp(0.0, 1.0);
        let context = GUID::from_u128(0);
        endpoint.SetMasterVolumeLevelScalar(target, &context).is_ok()
    }

    #[repr(C)]
    struct AudioVolumeNotificationData {
        guid_event_context: GUID,
        muted: BOOL,
        master_volume: f32,
        channels: u32,
        channel_volumes: [f32; 1],
    }

    #[repr(C)]
    struct CallbackVtbl {
        query_interface: unsafe extern "system" fn(*mut std::ffi::c_void, *const GUID, *mut *mut std::ffi::c_void) -> HRESULT,
        add_ref: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        release: unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        on_notify: unsafe extern "system" fn(*mut std::ffi::c_void, *const AudioVolumeNotificationData) -> HRESULT,
    }

    unsafe impl Sync for CallbackVtbl {}
    unsafe impl Send for CallbackVtbl {}

    #[repr(C)]
    struct VolumeCallbackImpl {
        vtbl: *const CallbackVtbl,
    }

    unsafe impl Sync for VolumeCallbackImpl {}
    unsafe impl Send for VolumeCallbackImpl {}

    unsafe extern "system" fn volume_query_interface(
        this: *mut std::ffi::c_void,
        riid: *const GUID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> HRESULT {
        if ppv.is_null() {
            return windows::Win32::Foundation::E_POINTER;
        }
        let iid_unk = GUID::from_u128(0x00000000_0000_0000_c000_000000000046);
        let iid_cb = GUID::from_u128(0x657804fa_d6ad_4496_8a60_352752af4f89);
        unsafe {
            if *riid == iid_unk || *riid == iid_cb {
                *ppv = this;
                return windows::Win32::Foundation::S_OK;
            }
            *ppv = std::ptr::null_mut();
        }
        windows::Win32::Foundation::E_NOINTERFACE
    }

    unsafe extern "system" fn volume_add_ref(_this: *mut std::ffi::c_void) -> u32 {
        1
    }

    unsafe extern "system" fn volume_release(_this: *mut std::ffi::c_void) -> u32 {
        1
    }

    unsafe extern "system" fn volume_on_notify(
        _this: *mut std::ffi::c_void,
        _p_notify: *const AudioVolumeNotificationData,
    ) -> HRESULT {
        unsafe {
            if !PANEL.is_invalid() {
                let _ = PostMessageW(PANEL, WM_APP_AUDIO_CHANGE, WPARAM(0), LPARAM(0));
            }
        }
        windows::Win32::Foundation::S_OK
    }

    static VOLUME_CALLBACK_VTBL: CallbackVtbl = CallbackVtbl {
        query_interface: volume_query_interface,
        add_ref: volume_add_ref,
        release: volume_release,
        on_notify: volume_on_notify,
    };

    static VOLUME_CALLBACK_INSTANCE: VolumeCallbackImpl = VolumeCallbackImpl {
        vtbl: &VOLUME_CALLBACK_VTBL,
    };

    unsafe fn register_volume_callback() {
        if let Some(endpoint) = default_audio_endpoint() {
            let cb_ptr = &VOLUME_CALLBACK_INSTANCE as *const VolumeCallbackImpl as *mut std::ffi::c_void;
            let cb: &IAudioEndpointVolumeCallback = std::mem::transmute(&cb_ptr);
            if endpoint.RegisterControlChangeNotify(cb).is_ok() {
                REGISTERED_ENDPOINT = Some(endpoint);
                log_event("registered audio endpoint volume callback");
            }
        }
    }

    unsafe fn unregister_volume_callback() {
        if let Some(endpoint) = REGISTERED_ENDPOINT.take() {
            let cb_ptr = &VOLUME_CALLBACK_INSTANCE as *const VolumeCallbackImpl as *mut std::ffi::c_void;
            let cb: &IAudioEndpointVolumeCallback = std::mem::transmute(&cb_ptr);
            let _ = endpoint.UnregisterControlChangeNotify(cb);
            log_event("unregistered audio endpoint volume callback");
        }
    }

    unsafe fn query_system_brightness() -> u8 {
        let mut active_scheme: *mut GUID = null_mut();
        let res = PowerGetActiveScheme(None, &mut active_scheme);
        if res == WIN32_ERROR(0) && !active_scheme.is_null() {
            let video_subgroup = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
            let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
            let mut val = 0u32;
            let read = PowerReadACValueIndex(
                None,
                Some(active_scheme),
                Some(&video_subgroup),
                Some(&brightness_guid),
                &mut val,
            );
            let _ = LocalFree(HLOCAL(active_scheme as *mut _));
            if read == WIN32_ERROR(0) {
                CURRENT_BRIGHTNESS = val.clamp(0, 100) as u8;
                return CURRENT_BRIGHTNESS;
            }
        }
        CURRENT_BRIGHTNESS
    }

    unsafe fn adjust_system_brightness(delta: i8) -> u8 {
        let cur = CURRENT_BRIGHTNESS as i16 + delta as i16;
        let target = cur.clamp(0, 100) as u8;
        CURRENT_BRIGHTNESS = target;

        let mut active_scheme: *mut GUID = null_mut();
        let res = PowerGetActiveScheme(None, &mut active_scheme);
        if res == WIN32_ERROR(0) && !active_scheme.is_null() {
            let video_subgroup = GUID::from_u128(0x7516b95f_f776_4464_8c53_06167f40cc99);
            let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
            let _ = PowerWriteACValueIndex(
                None,
                active_scheme,
                Some(&video_subgroup),
                Some(&brightness_guid),
                target as u32,
            );
            let _ = PowerSetActiveScheme(None, Some(active_scheme));
            let _ = LocalFree(HLOCAL(active_scheme as *mut _));
        }
        target
    }

    unsafe fn show_volume_osd() {
        if let Some((level, muted)) = default_audio_state() {
            OSD_KIND = OsdKind::Volume;
            OSD_LEVEL = level;
            OSD_MUTED = muted;
            present_osd();
        }
    }

    unsafe fn show_brightness_osd(level: u8) {
        OSD_KIND = OsdKind::Brightness;
        OSD_LEVEL = level;
        OSD_MUTED = false;
        CURRENT_BRIGHTNESS = level;
        present_osd();
    }

    unsafe fn present_osd() {
        if OSD_WINDOW.is_invalid() {
            return;
        }
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(hmon, &mut mi);

        let mon_left = mi.rcWork.left;
        let mon_right = mi.rcWork.right;
        let mon_bottom = mi.rcWork.bottom;

        let x = mon_left + ((mon_right - mon_left) - OSD_WIDTH) / 2;
        let y = mon_bottom - 95;

        let _ = SetWindowPos(
            OSD_WINDOW,
            HWND_TOPMOST,
            x,
            y,
            OSD_WIDTH,
            OSD_HEIGHT,
            SWP_SHOWWINDOW | SWP_NOACTIVATE,
        );
        let _ = InvalidateRect(OSD_WINDOW, None, false);
        let _ = SetTimer(OSD_WINDOW, ID_OSD_TIMER, 1500, None);
    }

    unsafe fn paint_osd(hwnd: HWND, hdc: HDC) {
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);

        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bmp = CreateCompatibleBitmap(hdc, r.right, r.bottom);
        let old_bmp = SelectObject(mem_dc, mem_bmp);

        let is_dark = CONFIG.dark;
        let is_hc = high_contrast();

        let bg_col = if is_hc {
            0x000000
        } else if is_dark {
            0x201c18
        } else {
            0xfafbfe
        };

        let bg = brush(bg_col);
        FillRect(mem_dc, &r, bg);
        let _ = DeleteObject(bg);

        let border_col = if is_hc {
            0xffffff
        } else if is_dark {
            0x383430
        } else {
            0xd6dbe1
        };
        let border_pen = CreatePen(PS_SOLID, 1, COLORREF(border_col));
        let old_pen = SelectObject(mem_dc, border_pen);
        let null_brush = windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::NULL_BRUSH);
        let old_brush = SelectObject(mem_dc, null_brush);
        let _ = RoundRect(mem_dc, r.left, r.top, r.right, r.bottom, 16, 16);
        SelectObject(mem_dc, old_brush);
        SelectObject(mem_dc, old_pen);
        let _ = DeleteObject(border_pen);

        let (icon, icon_col, label, fill_col) = match OSD_KIND {
            OsdKind::Volume => {
                if OSD_MUTED {
                    ("\u{E74F}", 0xf87171, "Mute".to_string(), 0xf87171)
                } else {
                    let ic = if OSD_LEVEL < 33 {
                        "\u{E993}"
                    } else if OSD_LEVEL < 66 {
                        "\u{E994}"
                    } else {
                        "\u{E767}"
                    };
                    (ic, 0x38bdf8, format!("{}%", OSD_LEVEL), 0x38bdf8)
                }
            }
            OsdKind::Brightness => {
                ("\u{E706}", 0xfbbf24, format!("{}%", OSD_LEVEL), 0xfbbf24)
            }
        };

        icon_text(mem_dc, icon, 16, 15, if is_hc { 0xffffff } else { icon_col }, 16);

        let track_x = 48;
        let track_y = 22;
        let track_w = 148;
        let track_h = 10;

        let track_col = if is_hc {
            0x333333
        } else if is_dark {
            0x322d28
        } else {
            0xe2e8f0
        };
        let track_brush = brush(track_col);
        let track_pen = CreatePen(PS_NULL, 0, COLORREF(0));
        let t_old_pen = SelectObject(mem_dc, track_pen);
        let t_old_brush = SelectObject(mem_dc, track_brush);
        let _ = RoundRect(mem_dc, track_x, track_y, track_x + track_w, track_y + track_h, 8, 8);

        let level = if OSD_KIND == OsdKind::Volume && OSD_MUTED { 0 } else { OSD_LEVEL };
        if level > 0 {
            let fill_w = ((track_w * level as i32) / 100).max(8).min(track_w);
            let f_col = if is_hc { 0x00ffff } else { fill_col };
            let fill_brush = brush(f_col);
            SelectObject(mem_dc, fill_brush);
            let _ = RoundRect(mem_dc, track_x, track_y, track_x + fill_w, track_y + track_h, 8, 8);
            SelectObject(mem_dc, t_old_brush);
            let _ = DeleteObject(fill_brush);
        } else {
            SelectObject(mem_dc, t_old_brush);
        }
        SelectObject(mem_dc, t_old_pen);
        let _ = DeleteObject(track_brush);
        let _ = DeleteObject(track_pen);

        let text_col = if is_hc {
            0xffffff
        } else if is_dark {
            0xf1f5f9
        } else {
            0x0f172a
        };
        text(mem_dc, &label, 206, 17, text_col, 13);

        let _ = BitBlt(hdc, 0, 0, r.right, r.bottom, mem_dc, 0, 0, SRCCOPY);

        SelectObject(mem_dc, old_bmp);
        let _ = DeleteObject(mem_bmp);
        let _ = DeleteDC(mem_dc);
    }

    extern "system" fn osd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    let h = BeginPaint(hwnd, &mut ps);
                    paint_osd(hwnd, h);
                    EndPaint(hwnd, &ps);
                    LRESULT(0)
                }
                WM_TIMER => {
                    if w.0 == ID_OSD_TIMER {
                        let _ = KillTimer(hwnd, ID_OSD_TIMER);
                        ShowWindow(hwnd, SW_HIDE);
                    }
                    LRESULT(0)
                }
                WM_ERASEBKGND => LRESULT(1),
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
    }

    unsafe fn network_state() -> (bool, bool) {
        let mut size = 15_000u32;
        let (mut buffer, result) = loop {
            let mut buffer = vec![0u8; size as usize];
            let result = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GET_ADAPTERS_ADDRESSES_FLAGS(0),
                None,
                Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut size,
            );
            if result == ERROR_BUFFER_OVERFLOW.0 {
                if size <= buffer.len() as u32 {
                    return (false, false);
                }
                continue;
            }
            break (buffer, result);
        };
        if result != 0 {
            return (false, false);
        }
        let mut current = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;
        let mut connected = false;
        let mut wifi = false;
        while !current.is_null() {
            let adapter = &*current;
            if adapter.OperStatus == IfOperStatusUp && adapter.IfType != IF_TYPE_SOFTWARE_LOOPBACK {
                connected = true;
                wifi |= adapter.IfType == IF_TYPE_IEEE80211;
            }
            current = adapter.Next;
        }
        (connected, wifi)
    }

    unsafe fn wifi_details() -> Option<(String, u32)> {
        let mut negotiated_version = 0u32;
        let mut client = HANDLE::default();
        if WlanOpenHandle(2, None, &mut negotiated_version, &mut client) != 0 {
            return None;
        }
        let mut interface_list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        let result = WlanEnumInterfaces(client, None, &mut interface_list);
        if result != 0 || interface_list.is_null() {
            let _ = WlanCloseHandle(client, None);
            return None;
        }
        let list = &*interface_list;
        let mut details = None;
        for index in 0..list.dwNumberOfItems as usize {
            let interface = &*list.InterfaceInfo.as_ptr().add(index);
            if interface.isState != wlan_interface_state_connected {
                continue;
            }
            let mut data_size = 0u32;
            let mut data: *mut core::ffi::c_void = null_mut();
            let mut value_type = WLAN_OPCODE_VALUE_TYPE::default();
            let result = WlanQueryInterface(
                client,
                &interface.InterfaceGuid,
                wlan_intf_opcode_current_connection,
                None,
                &mut data_size,
                &mut data,
                Some(&mut value_type),
            );
            if result == 0
                && !data.is_null()
                && data_size as usize >= size_of::<WLAN_CONNECTION_ATTRIBUTES>()
            {
                let connection = &*(data as *const WLAN_CONNECTION_ATTRIBUTES);
                let ssid = &connection.wlanAssociationAttributes.dot11Ssid;
                let length = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
                let name = String::from_utf8_lossy(&ssid.ucSSID[..length]).to_string();
                details = Some((
                    if name.is_empty() {
                        "Wi-Fi".to_string()
                    } else {
                        name
                    },
                    connection
                        .wlanAssociationAttributes
                        .wlanSignalQuality
                        .min(100),
                ));
            }
            if !data.is_null() {
                WlanFreeMemory(data);
            }
            if details.is_some() {
                break;
            }
        }
        WlanFreeMemory(interface_list as *const core::ffi::c_void);
        let _ = WlanCloseHandle(client, None);
        details
    }

    unsafe fn network_snapshot() -> (bool, bool, Option<(String, u32)>) {
        if NETWORK_TICKS < 4 {
            NETWORK_TICKS += 1;
            return (
                NETWORK_CONNECTED,
                NETWORK_WIFI,
                NETWORK_WIFI_DETAILS.clone(),
            );
        }
        let (connected, wifi) = network_state();
        NETWORK_CONNECTED = connected;
        NETWORK_WIFI = wifi;
        NETWORK_WIFI_DETAILS = if wifi { wifi_details() } else { None };
        NETWORK_TICKS = 0;
        (
            NETWORK_CONNECTED,
            NETWORK_WIFI,
            NETWORK_WIFI_DETAILS.clone(),
        )
    }

    unsafe fn media_playing() -> Option<bool> {
        if MEDIA_MANAGER.is_none() {
            MEDIA_MANAGER = Some(
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                    .ok()?
                    .get()
                    .ok()?,
            );
        }
        let manager = MEDIA_MANAGER.as_ref()?;
        let session = manager.GetCurrentSession().ok()?;
        let status = session.GetPlaybackInfo().ok()?.PlaybackStatus().ok()?;
        match status {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => Some(true),
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => Some(false),
            _ => None,
        }
    }

    unsafe fn toggle_media() {
        if MEDIA_MANAGER.is_none() {
            let _ = media_playing();
        }
        let Some(manager) = MEDIA_MANAGER.as_ref() else {
            return;
        };
        let Ok(session) = manager.GetCurrentSession() else {
            return;
        };
        if media_playing() == Some(true) {
            let _ = session
                .TryPauseAsync()
                .and_then(|operation| operation.get());
        } else {
            let _ = session.TryPlayAsync().and_then(|operation| operation.get());
        }
    }

    trait Applet {
        fn enabled(&self, c: Config) -> bool;
        fn render(&self, hdc: HDC, right: &mut i32);
    }
    struct ClockApplet;
    struct NetworkApplet;
    struct VolumeApplet;
    struct BluetoothApplet;
    struct MediaApplet;
    struct BatteryApplet;
    struct NotificationApplet;
    fn format_date(n: SYSTEMTIME, format: u8) -> String {
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
            unsafe {
                icon_text(hdc, "\u{E823}", *right - width, 10, 0x94a3b8, 14);
                text(hdc, &value, *right - width + 20, 10, 0xe8ecf2, 13);
            }
            *right -= width;
        }
    }
    impl Applet for NetworkApplet {
        fn enabled(&self, c: Config) -> bool {
            c.show_network
        }
        fn render(&self, hdc: HDC, right: &mut i32) {
            let (connected, wifi, wifi_details) = unsafe { network_snapshot() };
            unsafe {
                icon_text(
                    hdc,
                    if wifi {
                        "\u{E701}"
                    } else if connected {
                        "\u{E839}"
                    } else {
                        "\u{E783}"
                    },
                    *right - 28,
                    10,
                    if connected { 0x38bdf8 } else { 0xf87171 },
                    16,
                );
                if let Some((ssid, quality)) = wifi_details {
                    let label = format!("{} {}%", ssid, quality);
                    text(hdc, &label, *right - 150, 11, 0xb8c0cc, 11);
                    *right -= 112;
                }
            }
            *right -= 38;
        }
    }
    impl Applet for VolumeApplet {
        fn enabled(&self, c: Config) -> bool {
            c.show_volume
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
            unsafe {
                icon_text(hdc, icon, *right - width, 10, icon_col, 15);
                text(hdc, &label, *right - width + 20, 10, 0xe2e8f0, 13);
            }
            *right -= width + 10;
        }
    }
    impl Applet for BluetoothApplet {
        fn enabled(&self, c: Config) -> bool {
            c.show_bluetooth
        }
        fn render(&self, hdc: HDC, right: &mut i32) {
            let present = unsafe {
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
            unsafe {
                icon_text(
                    hdc,
                    if present { "\u{E702}" } else { "\u{E711}" },
                    *right - 26,
                    10,
                    if present { 0x38bdf8 } else { 0x7f8a9a },
                    15,
                );
            }
            *right -= 36;
        }
    }
    impl Applet for MediaApplet {
        fn enabled(&self, c: Config) -> bool {
            c.show_media
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
            let width = 64;
            unsafe {
                icon_text(hdc, icon, *right - width, 10, col, 15);
                text(hdc, &label, *right - width + 20, 10, col, 13);
            }
            *right -= width + 8;
        }
    }
    impl Applet for NotificationApplet {
        fn enabled(&self, c: Config) -> bool {
            c.show_notifications
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

    unsafe fn show_power_menu(hwnd: HWND, x: i32, y: i32) {
        let menu = CreatePopupMenu().unwrap();
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
            ID_MENU_SETTINGS,
            PCWSTR(wide("Luma Settings").as_ptr()),
        );
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            ID_MENU_RESTORE_EXPLORER,
            PCWSTR(wide("Restore Explorer Shell").as_ptr()),
        );

        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, x, y, 0, hwnd, None);
        let _ = DestroyMenu(menu);

        match cmd.0 as usize {
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
            ID_MENU_SETTINGS => {
                ShowWindow(SETTINGS, SW_SHOW);
                SetForegroundWindow(SETTINGS);
            }
            ID_MENU_RESTORE_EXPLORER => {
                restore_explorer();
            }
            _ => {}
        }
    }

    unsafe fn paint_panel(hwnd: HWND, screen_dc: HDC) {
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
        FillRect(hdc, &luma_pill, pill_bg);
        let _ = DeleteObject(pill_bg);

        icon_text(
            hdc,
            "\u{E707}",
            16,
            11,
            if accessible { 0xffffff } else { 0x38bdf8 },
            16,
        );
        text(
            hdc,
            "Luma",
            37,
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
        FillRect(hdc, &act_pill, act_bg);
        let _ = DeleteObject(act_bg);

        icon_text(
            hdc,
            "\u{E7C4}",
            87,
            11,
            if accessible { 0xffffff } else { 0x94a3b8 },
            14,
        );
        text(
            hdc,
            "Activities",
            106,
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

        icon_text(
            hdc,
            "\u{E7E8}",
            r.right - 32,
            10,
            if accessible { 0xffffff } else { 0xf87171 },
            16,
        );
        icon_text(
            hdc,
            "\u{E713}",
            r.right - 66,
            10,
            if accessible { 0xffffff } else { 0x94a3b8 },
            16,
        );

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
                1 => (r.right * 2) / 3,
                _ => r.right - 78,
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
    unsafe fn panel_applet_at(hwnd: HWND, x: i32) -> Option<u8> {
        let mut client = RECT::default();
        GetClientRect(hwnd, &mut client).ok()?;
        let c = CONFIG;
        for zone in 0..3u8 {
            let mut right = match zone {
                0 => client.right / 3,
                1 => (client.right * 2) / 3,
                _ => client.right - 78,
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
                let width = match id {
                    0 => {
                        if c.date_format == 0 {
                            96
                        } else {
                            148
                        }
                    }
                    1 | 3 | 5 => 38,
                    6 => 46,
                    2 => 88,
                    _ => 72,
                };
                let left = right - width;
                if x >= left && x < right {
                    return Some(id);
                }
                right = left;
            }
        }
        None
    }
    unsafe fn draw_shell_icon(hdc: HDC, target: &str, x: i32, y: i32, size: i32) -> bool {
        let mut shell_info = SHFILEINFOW::default();
        let target_wide = wide(target);
        let icon_loaded = SHGetFileInfoW(
            PCWSTR(target_wide.as_ptr()),
            Default::default(),
            Some(&mut shell_info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        ) != 0
            && !shell_info.hIcon.is_invalid();
        if !icon_loaded {
            return false;
        }
        let _ = DrawIconEx(hdc, x, y, shell_info.hIcon, size, size, 0, None, DI_NORMAL);
        let _ = DestroyIcon(shell_info.hIcon);
        true
    }

    const DOCK_METADATA: [(&str, &str, u32, &str); 5] = [
        ("Home", "\u{E80F}", 0xe06d20, "shell:Home"),
        ("Files", "\u{ED25}", 0x2563eb, "explorer.exe"),
        ("Terminal", "\u{E756}", 0x1e293b, "wt.exe"),
        ("Browser", "\u{E774}", 0x0284c7, "https://www.bing.com"),
        ("Settings", "\u{E713}", 0x475569, "ms-settings:"),
    ];

    unsafe fn resolve_system_dock_icon(item_id: usize) -> HICON {
        let mut shell_info = SHFILEINFOW::default();
        match item_id {
            0 => {
                if let Some(user_dir) = env::var_os("USERPROFILE") {
                    let wpath = wide(&user_dir.to_string_lossy());
                    let res = SHGetFileInfoW(
                        PCWSTR(wpath.as_ptr()),
                        Default::default(),
                        Some(&mut shell_info),
                        size_of::<SHFILEINFOW>() as u32,
                        SHGFI_ICON | SHGFI_LARGEICON,
                    );
                    if res != 0 && !shell_info.hIcon.is_invalid() {
                        return shell_info.hIcon;
                    }
                }
            }
            1 => {
                let wpath = wide("C:\\Windows\\explorer.exe");
                let res = SHGetFileInfoW(
                    PCWSTR(wpath.as_ptr()),
                    Default::default(),
                    Some(&mut shell_info),
                    size_of::<SHFILEINFOW>() as u32,
                    SHGFI_ICON | SHGFI_LARGEICON,
                );
                if res != 0 && !shell_info.hIcon.is_invalid() {
                    return shell_info.hIcon;
                }
            }
            2 => {
                let local_app = env::var_os("LOCALAPPDATA").map(PathBuf::from);
                let wt = local_app.map(|p| p.join("Microsoft\\WindowsApps\\wt.exe"));
                let candidates = [
                    wt.as_deref().and_then(|p| p.to_str()).unwrap_or(""),
                    "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
                    "C:\\Windows\\System32\\cmd.exe",
                ];
                for path in candidates {
                    if !path.is_empty() && Path::new(path).exists() {
                        let wpath = wide(path);
                        let res = SHGetFileInfoW(
                            PCWSTR(wpath.as_ptr()),
                            Default::default(),
                            Some(&mut shell_info),
                            size_of::<SHFILEINFOW>() as u32,
                            SHGFI_ICON | SHGFI_LARGEICON,
                        );
                        if res != 0 && !shell_info.hIcon.is_invalid() {
                            return shell_info.hIcon;
                        }
                    }
                }
            }
            3 => {
                let candidates = [
                    "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                    "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
                ];
                for path in candidates {
                    if Path::new(path).exists() {
                        let wpath = wide(path);
                        let res = SHGetFileInfoW(
                            PCWSTR(wpath.as_ptr()),
                            Default::default(),
                            Some(&mut shell_info),
                            size_of::<SHFILEINFOW>() as u32,
                            SHGFI_ICON | SHGFI_LARGEICON,
                        );
                        if res != 0 && !shell_info.hIcon.is_invalid() {
                            return shell_info.hIcon;
                        }
                    }
                }
            }
            4 => {
                let candidates = [
                    "C:\\Windows\\ImmersiveControlPanel\\SystemSettings.exe",
                    "C:\\Windows\\System32\\SystemSettings.exe",
                ];
                for path in candidates {
                    if Path::new(path).exists() {
                        let wpath = wide(path);
                        let res = SHGetFileInfoW(
                            PCWSTR(wpath.as_ptr()),
                            Default::default(),
                            Some(&mut shell_info),
                            size_of::<SHFILEINFOW>() as u32,
                            SHGFI_ICON | SHGFI_LARGEICON,
                        );
                        if res != 0 && !shell_info.hIcon.is_invalid() {
                            return shell_info.hIcon;
                        }
                    }
                }
                let mut h_large = HICON::default();
                let imageres = wide("C:\\Windows\\System32\\imageres.dll");
                let count =
                    ExtractIconExW(PCWSTR(imageres.as_ptr()), 114, Some(&mut h_large), None, 1);
                if count > 0 && !h_large.is_invalid() {
                    return h_large;
                }
            }
            _ => {}
        }
        HICON(null_mut())
    }

    unsafe fn draw_dock_icon(
        hdc: HDC,
        item_id: usize,
        cx: i32,
        cy: i32,
        size: i32,
        accessible: bool,
    ) {
        let id = item_id.min(4);
        let (_, fallback_glyph, tile_color, _) = DOCK_METADATA[id];
        let mut icon = DOCK_ICON_CACHE[id];
        if icon.is_invalid() {
            icon = resolve_system_dock_icon(id);
            DOCK_ICON_CACHE[id] = icon;
        }

        let half = size / 2;
        let left = cx - half;
        let top = cy - half;
        let right = cx + half;
        let bottom = cy + half;

        if !icon.is_invalid() {
            let _ = DrawIconEx(hdc, left, top, icon, size, size, 0, None, DI_NORMAL);
        } else {
            let bg_col = if accessible { 0x000000 } else { tile_color };
            let b = brush(bg_col);
            let tile_r = RECT {
                left,
                top,
                right,
                bottom,
            };
            FillRect(hdc, &tile_r, b);
            let _ = DeleteObject(b);

            let glyph_size = (size as f32 * 0.52).max(12.0) as i32;
            let gx = cx - glyph_size / 2;
            let gy = cy - glyph_size / 2;
            icon_text(hdc, fallback_glyph, gx, gy, 0xffffff, glyph_size);
        }
    }

    unsafe fn clear_dock_icon_cache() {
        for icon in &mut DOCK_ICON_CACHE {
            if !icon.is_invalid() {
                let _ = DestroyIcon(*icon);
                *icon = HICON(null_mut());
            }
        }
    }

    unsafe fn paint_dock(hwnd: HWND, screen_dc: HDC) {
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

        let c = CONFIG;
        let accessible = high_contrast();
        let bg = brush(if accessible {
            0x000000
        } else if c.dark {
            0x1e222b
        } else {
            0xffffff
        });
        FillRect(hdc, &r, bg);
        let _ = DeleteObject(bg);

        let _ = running_window_count();
        let mut x = 24;
        for (index, item_id) in c.dock_order.iter().enumerate() {
            let id = *item_id as usize;
            let slot_cx = x + 24;
            let slot_cy = 28;

            let distance = if DOCK_POINTER_X >= 0 {
                (DOCK_POINTER_X - slot_cx).abs()
            } else {
                i32::MAX
            };
            let influence = ((110 - distance).max(0) as f32 / 110.0)
                * DOCK_MAGNIFICATION
                * (c.magnification as f32 / 45.0);
            let size = c.icon_size + (c.icon_size as f32 * 0.45 * influence) as i32;
            let active = dock_item_active(*item_id);

            if HOVERED_DOCK_ICON == index as i32 || active {
                let highlight = brush(if accessible {
                    if HOVERED_DOCK_ICON == index as i32 {
                        0xffff00
                    } else {
                        0x404040
                    }
                } else if c.dark {
                    if HOVERED_DOCK_ICON == index as i32 {
                        0x333d4e
                    } else {
                        0x283243
                    }
                } else {
                    if HOVERED_DOCK_ICON == index as i32 {
                        0xe2e8f0
                    } else {
                        0xdbeafe
                    }
                });
                let box_rect = RECT {
                    left: slot_cx - 28,
                    top: 6,
                    right: slot_cx + 28,
                    bottom: 54,
                };
                FillRect(hdc, &box_rect, highlight);
                let _ = DeleteObject(highlight);
            }

            draw_dock_icon(hdc, id, slot_cx, slot_cy, size, accessible);

            if dock_item_running(*item_id) {
                if active {
                    let dot_brush = brush(if accessible { 0xffff00 } else { 0x38bdf8 });
                    let pill_r = RECT {
                        left: slot_cx - 7,
                        top: 56,
                        right: slot_cx + 7,
                        bottom: 60,
                    };
                    FillRect(hdc, &pill_r, dot_brush);
                    let _ = DeleteObject(dot_brush);
                } else {
                    let dot_brush = brush(if accessible { 0xffffff } else { 0x94a3b8 });
                    let dot_r = RECT {
                        left: slot_cx - 2,
                        top: 56,
                        right: slot_cx + 3,
                        bottom: 60,
                    };
                    FillRect(hdc, &dot_r, dot_brush);
                    let _ = DeleteObject(dot_brush);
                }
            }

            x += c.spacing;
        }

        let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
        SelectObject(hdc, old_bmp);
        let _ = DeleteObject(mem_bmp);
        let _ = DeleteDC(hdc);
    }
    unsafe fn is_shell_window(hwnd: HWND) -> bool {
        if hwnd == PANEL
            || hwnd == DOCK
            || hwnd == DESKTOP
            || hwnd == SETTINGS
            || hwnd == LAUNCHER
            || hwnd == SWITCHER
            || hwnd == AUDIO_MIXER
        {
            return true;
        }
        (0..SECONDARY_COUNT)
            .any(|index| SECONDARY_PANELS[index] == hwnd || SECONDARY_DOCKS[index] == hwnd)
    }

    extern "system" fn count_window(hwnd: HWND, _: LPARAM) -> BOOL {
        unsafe {
            if !is_shell_window(hwnd)
                && IsWindowVisible(hwnd).as_bool()
                && GetWindowTextLengthW(hwnd) > 0
                && window_on_current_workspace(hwnd)
            {
                RUNNING_WINDOWS += 1;
                WINDOW_LIST.push(hwnd);
            }
        }
        true.into()
    }
    unsafe fn running_window_count() -> usize {
        RUNNING_WINDOWS = 0;
        WINDOW_LIST.clear();
        let _ = EnumWindows(Some(count_window), LPARAM(0));
        RUNNING_WINDOWS
    }
    unsafe fn dock_aliases(item_id: u8) -> &'static [&'static str] {
        match item_id {
            0 | 1 => &["explorer"],
            2 => &[
                "wt",
                "windowsterminal",
                "cmd",
                "powershell",
                "pwsh",
                "wezterm",
            ],
            3 => &["msedge", "chrome", "firefox", "brave", "opera", "vivaldi"],
            _ => &[],
        }
    }
    unsafe fn dock_window_index(item_id: u8) -> Option<usize> {
        if item_id == 4 {
            return None;
        }
        let aliases = dock_aliases(item_id);
        WINDOW_LIST.iter().position(|window| {
            let mut pid = 0u32;
            GetWindowThreadProcessId(*window, Some(&mut pid));
            let label = process_label(pid).to_ascii_lowercase();
            aliases.iter().any(|alias| label == *alias)
        })
    }
    unsafe fn dock_item_running(item_id: u8) -> bool {
        if item_id == 4 {
            return IsWindowVisible(SETTINGS).as_bool();
        }
        dock_window_index(item_id).is_some()
    }
    unsafe fn dock_item_active(item_id: u8) -> bool {
        let foreground = GetForegroundWindow();
        if item_id == 4 {
            return foreground == SETTINGS;
        }
        dock_window_index(item_id)
            .and_then(|index| WINDOW_LIST.get(index).copied())
            .is_some_and(|window| window == foreground)
    }
    unsafe fn window_on_current_workspace(hwnd: HWND) -> bool {
        let manager: IVirtualDesktopManager =
            match CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL) {
                Ok(value) => value,
                Err(_) => return true,
            };
        manager
            .IsWindowOnCurrentVirtualDesktop(hwnd)
            .map(|value| value.as_bool())
            .unwrap_or(true)
    }
    unsafe fn activate_window(index: usize) {
        let _ = running_window_count();
        if let Some(hwnd) = WINDOW_LIST.get(index).copied() {
            if GetForegroundWindow() == hwnd {
                ShowWindow(hwnd, SW_MINIMIZE);
            } else {
                if IsIconic(hwnd).as_bool() {
                    ShowWindow(hwnd, SW_RESTORE);
                }
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
    unsafe fn activate_dock_item(item_id: u8) {
        if item_id == 4 {
            if IsWindowVisible(SETTINGS).as_bool() && GetForegroundWindow() == SETTINGS {
                ShowWindow(SETTINGS, SW_HIDE);
            } else {
                ShowWindow(SETTINGS, SW_SHOW);
                SetForegroundWindow(SETTINGS);
            }
            return;
        }
        let _ = running_window_count();
        if let Some(index) = dock_window_index(item_id) {
            activate_window(index);
            return;
        }
        match item_id {
            0 => launch("shell:Home"),
            1 => launch("explorer.exe"),
            2 => launch("wt.exe"),
            3 => launch("https://www.bing.com"),
            _ => {}
        }
    }
    unsafe fn clear_switcher_thumbnail() {
        if SWITCHER_THUMBNAIL != 0 {
            let _ = DwmUnregisterThumbnail(SWITCHER_THUMBNAIL);
            SWITCHER_THUMBNAIL = 0;
            SWITCHER_THUMBNAIL_SOURCE = HWND(null_mut());
        }
    }
    unsafe fn update_switcher_thumbnail() {
        let _ = running_window_count();
        let Some(source) = WINDOW_LIST.get(SWITCHER_SELECTION).copied() else {
            clear_switcher_thumbnail();
            return;
        };
        if SWITCHER_THUMBNAIL_SOURCE != source {
            clear_switcher_thumbnail();
            SWITCHER_THUMBNAIL = DwmRegisterThumbnail(SWITCHER, source).unwrap_or(0);
            SWITCHER_THUMBNAIL_SOURCE = source;
        }
        if SWITCHER_THUMBNAIL == 0 {
            return;
        }
        let properties = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE | DWM_TNP_OPACITY,
            rcDestination: RECT {
                left: 300,
                top: 92,
                right: 600,
                bottom: 390,
            },
            opacity: 255,
            fVisible: true.into(),
            ..Default::default()
        };
        let _ = DwmUpdateThumbnailProperties(SWITCHER_THUMBNAIL, &properties);
    }
    unsafe fn window_title(hwnd: HWND) -> String {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return "Untitled window".to_string();
        }
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..copied as usize])
    }
    unsafe fn toggle_dock() {
        DOCK_VISIBLE = !DOCK_VISIBLE;
        ShowWindow(
            DOCK,
            if DOCK_VISIBLE {
                SW_SHOWNOACTIVATE
            } else {
                SW_HIDE
            },
        );
        if DOCK_VISIBLE {
            let _ = SetForegroundWindow(DOCK);
            let _ = SetFocus(DOCK);
        }
    }
    unsafe fn switch_workspace(direction: VIRTUAL_KEY) {
        keybd_event(VK_LWIN.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_LCONTROL.0 as u8, 0, Default::default(), 0);
        keybd_event(direction.0 as u8, 0, Default::default(), 0);
        keybd_event(direction.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LCONTROL.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    unsafe fn move_window_workspace(direction: VIRTUAL_KEY) {
        keybd_event(VK_LWIN.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_LCONTROL.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_LSHIFT.0 as u8, 0, Default::default(), 0);
        keybd_event(direction.0 as u8, 0, Default::default(), 0);
        keybd_event(direction.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LSHIFT.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LCONTROL.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    unsafe fn activate_dock_icon(index: i32) {
        let item_id = CONFIG
            .dock_order
            .get(index.max(0) as usize)
            .copied()
            .unwrap_or(0);
        activate_dock_item(item_id);
    }

    unsafe fn notify_accessibility_focus(hwnd: HWND, index: i32) {
        if index >= 0 {
            NotifyWinEvent(
                EVENT_OBJECT_FOCUS,
                hwnd,
                OBJID_CLIENT.0,
                index.saturating_add(1),
            );
        }
    }

    unsafe fn reorder_dock_icon(from: i32, to: i32) {
        if !(0..5).contains(&from) || !(0..5).contains(&to) || from == to {
            return;
        }
        let order = &mut CONFIG.dock_order;
        let item = order[from as usize];
        if from < to {
            for index in from as usize..to as usize {
                order[index] = order[index + 1];
            }
        } else {
            for index in (to as usize + 1..=from as usize).rev() {
                order[index] = order[index - 1];
            }
        }
        order[to as usize] = item;
        save_config();
    }
    unsafe fn set_docks_visible(visible: bool) {
        ShowWindow(DOCK, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
        for &secondary in SECONDARY_DOCKS.iter().take(SECONDARY_COUNT) {
            ShowWindow(secondary, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
        DOCK_VISIBLE = visible;
    }
    unsafe fn update_auto_hide() {
        if !CONFIG.auto_hide {
            return;
        }
        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let mut inside = false;
        let mut docks = Vec::with_capacity(SECONDARY_COUNT + 1);
        docks.push(DOCK);
        docks.extend_from_slice(&SECONDARY_DOCKS[..SECONDARY_COUNT]);
        for dock in docks {
            if !IsWindowVisible(dock).as_bool() {
                continue;
            }
            let mut dock_rect = RECT::default();
            let _ = GetWindowRect(dock, &mut dock_rect);
            if cursor.x >= dock_rect.left
                && cursor.x < dock_rect.right
                && cursor.y >= dock_rect.top
                && cursor.y < dock_rect.bottom
            {
                inside = true;
                break;
            }
        }
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let near_edge = cursor.y >= info.rcWork.bottom - scaled(DOCK, 10);
        set_docks_visible(inside || near_edge);
    }

    unsafe fn invalidate_shell_surfaces() {
        let mut panels = Vec::with_capacity(SECONDARY_COUNT + 1);
        panels.push(PANEL);
        panels.extend_from_slice(&SECONDARY_PANELS[..SECONDARY_COUNT]);
        for panel in panels {
            if !panel.is_invalid() {
                let _ = InvalidateRect(panel, None, false);
            }
        }
        if IsWindowVisible(DOCK).as_bool() {
            let _ = InvalidateRect(DOCK, None, false);
        }
        for dock in &SECONDARY_DOCKS[..SECONDARY_COUNT] {
            if IsWindowVisible(*dock).as_bool() {
                let _ = InvalidateRect(*dock, None, false);
            }
        }
        if IsWindowVisible(AUDIO_MIXER).as_bool() {
            let _ = InvalidateRect(AUDIO_MIXER, None, false);
        }
    }

    unsafe fn refresh_app_index_if_due() {
        APP_INDEX_TICKS = APP_INDEX_TICKS.saturating_add(1);
        if APP_INDEX_TICKS < 120 {
            return;
        }
        APP_INDEX_TICKS = 0;
        APP_INDEX = Some(build_app_index());
        if IsWindowVisible(LAUNCHER).as_bool() {
            LAUNCHER_SELECTION = 0;
            let _ = InvalidateRect(LAUNCHER, None, false);
        }
        log_event("application index refreshed");
    }

    unsafe fn position_dock_on_monitor(dock: HWND, monitor: HMONITOR) {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let w = info.rcWork.right - info.rcWork.left;
        let h = info.rcWork.bottom - info.rcWork.top;
        let c = CONFIG;
        let dock_width = scaled(dock, DOCK_WIDTH);
        let dock_height = scaled(dock, DOCK_HEIGHT);
        let margin = scaled(dock, 24);
        let bottom_margin = scaled(dock, 22);
        let x = match c.position {
            DockPosition::Left => info.rcWork.left + margin,
            DockPosition::Right => info.rcWork.right - dock_width - margin,
            DockPosition::Center => info.rcWork.left + (w - dock_width) / 2,
        } + scaled(dock, c.offset_x);
        let _ = SetWindowPos(
            dock,
            HWND_TOPMOST,
            x,
            info.rcWork.top + h - dock_height - bottom_margin + scaled(dock, c.offset_y),
            dock_width,
            dock_height,
            SWP_SHOWWINDOW,
        );
    }

    unsafe fn position_dock() {
        position_dock_on_monitor(DOCK, MonitorFromWindow(PANEL, MONITOR_DEFAULTTONEAREST));
    }

    unsafe fn destroy_secondary_monitors() {
        for index in 0..SECONDARY_COUNT {
            if let Some(mut appbar) = SECONDARY_APPBARS[index].take() {
                unregister_panel_appbar(&mut appbar);
            }
            if !SECONDARY_PANELS[index].is_invalid() {
                let _ = DestroyWindow(SECONDARY_PANELS[index]);
            }
            if !SECONDARY_DOCKS[index].is_invalid() {
                let _ = DestroyWindow(SECONDARY_DOCKS[index]);
            }
            SECONDARY_PANELS[index] = HWND(null_mut());
            SECONDARY_DOCKS[index] = HWND(null_mut());
        }
        SECONDARY_COUNT = 0;
    }

    unsafe fn create_secondary_monitor(hmonitor: HMONITOR) {
        if SECONDARY_COUNT >= SECONDARY_PANELS.len() {
            return;
        }
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(hmonitor, &mut info);
        let width = info.rcMonitor.right - info.rcMonitor.left;
        let panel = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaPanel").as_ptr()),
            PCWSTR(wide("Luma Panel").as_ptr()),
            WS_POPUP | WS_VISIBLE,
            info.rcMonitor.left,
            info.rcMonitor.top,
            width,
            PANEL_HEIGHT,
            None,
            None,
            INSTANCE,
            None,
        )
        .unwrap();
        let dock = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaDock").as_ptr()),
            PCWSTR(wide("Luma Dock").as_ptr()),
            WS_POPUP | WS_VISIBLE,
            info.rcWork.left,
            info.rcWork.bottom - DOCK_HEIGHT - 22,
            DOCK_WIDTH,
            DOCK_HEIGHT,
            None,
            None,
            INSTANCE,
            None,
        )
        .unwrap();
        style_window(panel);
        style_window(dock);
        let appbar = register_panel_appbar_for_rect(panel, info.rcMonitor);
        position_dock_on_monitor(dock, hmonitor);
        SECONDARY_PANELS[SECONDARY_COUNT] = panel;
        SECONDARY_DOCKS[SECONDARY_COUNT] = dock;
        SECONDARY_APPBARS[SECONDARY_COUNT] = Some(appbar);
        SECONDARY_COUNT += 1;
    }

    extern "system" fn monitor_callback(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        _: LPARAM,
    ) -> BOOL {
        unsafe {
            if hmonitor != MonitorFromWindow(PANEL, MONITOR_DEFAULTTONEAREST) {
                create_secondary_monitor(hmonitor);
            }
        }
        true.into()
    }

    unsafe fn rebuild_secondary_monitors() {
        destroy_secondary_monitors();
        let _ = EnumDisplayMonitors(None, None, Some(monitor_callback), LPARAM(0));
    }

    unsafe fn reposition_secondary_panel(panel: HWND) {
        for index in 0..SECONDARY_COUNT {
            if SECONDARY_PANELS[index] == panel {
                let monitor = MonitorFromWindow(panel, MONITOR_DEFAULTTONEAREST);
                position_dock_on_monitor(SECONDARY_DOCKS[index], monitor);
                break;
            }
        }
    }

    fn button_style(extra: i32) -> WINDOW_STYLE {
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | extra as u32)
    }
    #[allow(clippy::too_many_arguments)]
    unsafe fn create_control(
        class: &str,
        label: &str,
        style: WINDOW_STYLE,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        id: usize,
    ) -> HWND {
        CreateWindowExW(
            WS_EX_LEFT,
            PCWSTR(wide(class).as_ptr()),
            PCWSTR(wide(label).as_ptr()),
            style,
            x,
            y,
            w,
            h,
            SETTINGS,
            HMENU(id as *mut _),
            GetModuleHandleW(None).unwrap(),
            None,
        )
        .unwrap()
    }
    unsafe fn move_settings_control(id: usize, x: i32, y: i32, width: i32, height: i32) {
        if let Ok(control) = GetDlgItem(SETTINGS, id as i32) {
            let _ = SetWindowPos(
                control,
                HWND_TOP,
                x,
                y,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    unsafe fn sync_settings_layout() {
        for (id, y) in [
            (ID_PAGE_GENERAL, 112),
            (ID_PAGE_DOCK, 152),
            (ID_PAGE_APPLETS, 192),
            (ID_PAGE_RECOVERY, 232),
        ] {
            move_settings_control(id, 22, y, 140, 30);
        }
        move_settings_control(ID_DARK, 210, 142, 250, 30);
        move_settings_control(ID_DESKTOP_TOGGLE, 210, 182, 300, 30);
        move_settings_control(ID_CLOCK_24H, 210, 222, 150, 30);
        move_settings_control(ID_DATE_FORMAT, 390, 222, 180, 30);

        for (id, x) in [(ID_LEFT, 210), (ID_CENTER, 305), (ID_RIGHT, 410)] {
            move_settings_control(id, x, 142, 90, 30);
        }
        move_settings_control(ID_DOCK_ORDER, 210, 225, 430, 30);
        move_settings_control(ID_AUTO_HIDE, 210, 270, 180, 30);
        move_settings_control(ID_ICON_SMALL, 210, 320, 75, 30);
        move_settings_control(ID_ICON_MEDIUM, 292, 320, 90, 30);
        move_settings_control(ID_ICON_LARGE, 390, 320, 75, 30);
        move_settings_control(ID_MAGNIFICATION, 210, 360, 150, 30);
        move_settings_control(ID_TRANSPARENCY, 390, 360, 150, 30);
        move_settings_control(ID_SPACING, 210, 400, 150, 30);
        move_settings_control(ID_OFFSETS, 390, 400, 150, 30);

        for (id, x, y) in [
            (ID_CLOCK_CHECK, 210, 142),
            (ID_NETWORK_CHECK, 330, 142),
            (ID_VOLUME_CHECK, 450, 142),
            (ID_BATTERY_CHECK, 210, 178),
            (ID_BLUETOOTH, 330, 178),
            (ID_NOTIFICATIONS, 450, 178),
            (ID_MEDIA, 210, 214),
        ] {
            move_settings_control(id, x, y, 110, 30);
        }
        move_settings_control(ID_ORDER, 210, 280, 430, 30);
        move_settings_control(ID_ZONES, 210, 340, 430, 30);

        move_settings_control(ID_SET_LUMA_SHELL, 210, 170, 230, 34);
        move_settings_control(ID_RESTORE_EXPLORER, 210, 218, 230, 34);
        move_settings_control(ID_STARTUP, 210, 268, 230, 28);
        move_settings_control(ID_RESET, 210, 312, 230, 34);
        move_settings_control(ID_OPEN_DIAGNOSTICS, 470, 170, 230, 34);
        move_settings_control(ID_OPEN_WATCHDOG, 470, 218, 230, 34);
        move_settings_control(ID_RESTORE_SETTINGS, 470, 268, 230, 34);
    }
    unsafe fn sync_settings_page() {
        sync_settings_layout();
        let groups: [&[usize]; 4] = [
            &[ID_DARK, ID_DESKTOP_TOGGLE, ID_CLOCK_24H, ID_DATE_FORMAT],
            &[
                ID_LEFT,
                ID_CENTER,
                ID_RIGHT,
                ID_AUTO_HIDE,
                ID_ICON_SMALL,
                ID_ICON_MEDIUM,
                ID_ICON_LARGE,
                ID_MAGNIFICATION,
                ID_TRANSPARENCY,
                ID_SPACING,
                ID_OFFSETS,
                ID_DOCK_ORDER,
            ],
            &[
                ID_CLOCK_CHECK,
                ID_NETWORK_CHECK,
                ID_VOLUME_CHECK,
                ID_BATTERY_CHECK,
                ID_NOTIFICATIONS,
                ID_BLUETOOTH,
                ID_MEDIA,
                ID_ORDER,
                ID_ZONES,
            ],
            &[
                ID_SET_LUMA_SHELL,
                ID_RESTORE_EXPLORER,
                ID_STARTUP,
                ID_RESET,
                ID_OPEN_DIAGNOSTICS,
                ID_OPEN_WATCHDOG,
                ID_RESTORE_SETTINGS,
            ],
        ];
        for (page, ids) in groups.iter().enumerate() {
            for id in *ids {
                if let Ok(hwnd) = GetDlgItem(SETTINGS, *id as i32) {
                    ShowWindow(
                        hwnd,
                        if page == SETTINGS_PAGE {
                            SW_SHOW
                        } else {
                            SW_HIDE
                        },
                    );
                }
            }
        }
        let _ = InvalidateRect(SETTINGS, None, false);
    }
    unsafe fn sync_settings_edit_fields() {
        let c = CONFIG;
        let order = wide(&order_text(c.applet_order));
        if let Ok(h) = GetDlgItem(SETTINGS, ID_ORDER as i32) {
            let _ = SetWindowTextW(h, PCWSTR(order.as_ptr()));
        }
        let dock_order = wide(&dock_order_text(c.dock_order));
        if let Ok(h) = GetDlgItem(SETTINGS, ID_DOCK_ORDER as i32) {
            let _ = SetWindowTextW(h, PCWSTR(dock_order.as_ptr()));
        }
        for (id, value) in [
            (ID_MAGNIFICATION, c.magnification.to_string()),
            (ID_TRANSPARENCY, c.transparency.to_string()),
            (ID_SPACING, c.spacing.to_string()),
            (ID_OFFSETS, format!("{},{}", c.offset_x, c.offset_y)),
        ] {
            let value = wide(&value);
            if let Ok(h) = GetDlgItem(SETTINGS, id as i32) {
                let _ = SetWindowTextW(h, PCWSTR(value.as_ptr()));
            }
        }
        let zones = wide(&zones_text(c.panel_zones));
        if let Ok(h) = GetDlgItem(SETTINGS, ID_ZONES as i32) {
            let _ = SetWindowTextW(h, PCWSTR(zones.as_ptr()));
        }
        let date_format = wide(match c.date_format {
            0 => "none",
            2 => "long",
            _ => "short",
        });
        if let Ok(h) = GetDlgItem(SETTINGS, ID_DATE_FORMAT as i32) {
            let _ = SetWindowTextW(h, PCWSTR(date_format.as_ptr()));
        }
    }

    unsafe fn sync_settings_controls() {
        if SYNCING_SETTINGS {
            return;
        }
        SYNCING_SETTINGS = true;
        let c = CONFIG;
        for (id, checked) in [
            (ID_DARK, c.dark),
            (ID_CLOCK_CHECK, c.show_clock),
            (ID_NETWORK_CHECK, c.show_network),
            (ID_VOLUME_CHECK, c.show_volume),
            (ID_BATTERY_CHECK, c.show_battery),
            (ID_NOTIFICATIONS, c.show_notifications),
            (ID_BLUETOOTH, c.show_bluetooth),
            (ID_MEDIA, c.show_media),
            (ID_AUTO_HIDE, c.auto_hide),
            (ID_ICON_SMALL, c.icon_size == 20),
            (ID_ICON_MEDIUM, c.icon_size == 25),
            (ID_ICON_LARGE, c.icon_size == 30),
            (ID_CLOCK_24H, c.clock_24h),
            (ID_DESKTOP_TOGGLE, c.desktop_enabled),
            (ID_LEFT, matches!(c.position, DockPosition::Left)),
            (ID_CENTER, matches!(c.position, DockPosition::Center)),
            (ID_RIGHT, matches!(c.position, DockPosition::Right)),
        ] {
            if let Ok(h) = GetDlgItem(SETTINGS, id as i32) {
                let _ = SendMessageW(
                    h,
                    BM_SETCHECK,
                    WPARAM(if checked { 1 } else { 0 }),
                    LPARAM(0),
                );
            }
        }
        if let Ok(startup) = GetDlgItem(SETTINGS, ID_STARTUP as i32) {
            let _ = SendMessageW(
                startup,
                BM_SETCHECK,
                WPARAM(if startup_enabled() { 1 } else { 0 }),
                LPARAM(0),
            );
        }
        if let Ok(btn) = GetDlgItem(SETTINGS, ID_SET_LUMA_SHELL as i32) {
            let label = wide(if is_luma_shell_active() {
                "Restore Windows Explorer"
            } else {
                "Set Luma as Default Shell"
            });
            let _ = SetWindowTextW(btn, PCWSTR(label.as_ptr()));
        }
        sync_settings_page();
        SYNCING_SETTINGS = false;
    }
    unsafe fn read_control_text(hwnd: HWND) -> String {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..copied as usize])
    }
    unsafe fn paint_settings(hwnd: HWND, screen_dc: HDC) {
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

        let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
        FillRect(hdc, &r, bg);
        let _ = DeleteObject(bg);
        text(
            hdc,
            "Luma Settings",
            34,
            28,
            if CONFIG.dark { 0xf7f8fa } else { 0x172033 },
            25,
        );
        text(
            hdc,
            "Panel, dock, and applets",
            36,
            66,
            if CONFIG.dark { 0x9da6b4 } else { 0x64748b },
            14,
        );
        match SETTINGS_PAGE {
            0 => {
                text(hdc, "Appearance & Desktop", 210, 112, 0x60a5fa, 16);
                text(
                    hdc,
                    "Date format: none / short / long",
                    210,
                    255,
                    0x60a5fa,
                    14,
                );
            }
            1 => {
                text(hdc, "Dock position", 210, 112, 0x60a5fa, 16);
                text(
                    hdc,
                    "Dock order (drag or comma separated)",
                    210,
                    210,
                    0x60a5fa,
                    14,
                );
                text(hdc, "Dock behavior", 210, 255, 0x60a5fa, 16);
                text(hdc, "Icon size", 210, 305, 0x60a5fa, 16);
                text(hdc, "Magnification %", 210, 345, 0x60a5fa, 12);
                text(hdc, "Transparency", 390, 345, 0x60a5fa, 12);
                text(hdc, "Spacing", 210, 385, 0x60a5fa, 12);
                text(hdc, "Offset x,y", 390, 385, 0x60a5fa, 12);
            }
            2 => {
                text(hdc, "Applet visibility", 210, 112, 0x60a5fa, 16);
                text(
                    hdc,
                    "Applet order (comma separated)",
                    210,
                    260,
                    0x60a5fa,
                    14,
                );
                text(
                    hdc,
                    "Zones: applet:left|center|right",
                    210,
                    320,
                    0x60a5fa,
                    14,
                );
            }
            _ => {
                text(hdc, "Windows Shell Replacement", 210, 112, 0x60a5fa, 16);
                let shell_active = is_luma_shell_active();
                if shell_active {
                    text(
                        hdc,
                        "● Luma Shell is set as default Windows Shell",
                        210,
                        140,
                        0x22c55e,
                        13,
                    );
                } else {
                    text(
                        hdc,
                        "○ Windows Explorer is the current default shell",
                        210,
                        140,
                        if CONFIG.dark { 0x94a3b8 } else { 0x64748b },
                        13,
                    );
                }
                text(hdc, "System Diagnostics & Backups", 470, 112, 0x60a5fa, 16);
                text(
                    hdc,
                    "Crash logs and atomic settings snapshots",
                    470,
                    140,
                    if CONFIG.dark { 0x94a3b8 } else { 0x64748b },
                    12,
                );
            }
        }
        text(
            hdc,
            "Changes are saved automatically",
            210,
            470,
            if CONFIG.dark { 0x7f8a9a } else { 0x64748b },
            12,
        );
        let _ = BitBlt(screen_dc, 0, 0, w, h, hdc, 0, 0, SRCCOPY);
        SelectObject(hdc, old_bmp);
        let _ = DeleteObject(mem_bmp);
        let _ = DeleteDC(hdc);
    }

    unsafe fn paint_launcher(hwnd: HWND, hdc: HDC) {
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r).unwrap();
        let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
        FillRect(hdc, &r, bg);
        let _ = DeleteObject(bg);
        text(
            hdc,
            "Applications",
            28,
            28,
            if CONFIG.dark { 0xf7f8fa } else { 0x172033 },
            23,
        );
        text(
            hdc,
            "Search installed apps",
            30,
            62,
            if CONFIG.dark { 0x9da6b4 } else { 0x64748b },
            13,
        );
        text(
            hdc,
            "Ctrl+D or right-click an app to favorite it",
            30,
            110,
            if CONFIG.dark { 0x7f8a9a } else { 0x64748b },
            11,
        );
        let query = launcher_query();
        let entries = launcher_entries(&query);
        let mut y = 140;
        for (index, (name, target)) in entries.iter().take(10).enumerate() {
            if index as i32 == LAUNCHER_SELECTION {
                let highlight = brush(if CONFIG.dark { 0x303846 } else { 0xe2e8f0 });
                let row = RECT {
                    left: 20,
                    top: y - 5,
                    right: r.right - 20,
                    bottom: y + 27,
                };
                FillRect(hdc, &row, highlight);
                let _ = DeleteObject(highlight);
            }
            if is_favorite(target) {
                icon_text(hdc, "\u{E735}", 34, y - 1, 0xfbbf24, 16);
            } else if !draw_shell_icon(hdc, target, 34, y - 2, 18) {
                if target.starts_with("ms-settings:") {
                    icon_text(hdc, "\u{E713}", 34, y - 1, 0x38bdf8, 16);
                } else if target.ends_with(".url") || target.starts_with("http") {
                    icon_text(hdc, "\u{E774}", 34, y - 1, 0x38bdf8, 16);
                } else {
                    icon_text(hdc, "\u{E71D}", 34, y - 1, 0x38bdf8, 16);
                }
            }
            text(
                hdc,
                name,
                72,
                y + 2,
                if CONFIG.dark { 0xe8ecf2 } else { 0x334155 },
                15,
            );
            y += 34;
        }
        if y == 140 {
            text(hdc, "No matching applications", 34, y, 0x7f8a9a, 14);
        }
    }

    unsafe fn position_popup_on_monitor(popup: HWND, monitor: HMONITOR) {
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return;
        }
        let mut rect = RECT::default();
        let _ = GetWindowRect(popup, &mut rect);
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let work_width = info.rcWork.right - info.rcWork.left;
        let work_height = info.rcWork.bottom - info.rcWork.top;
        let x = info.rcWork.left + (work_width - width).max(0) / 2;
        let y = info.rcWork.top + (work_height - height).max(0) / 2;
        let _ = SetWindowPos(popup, HWND_TOP, x, y, width, height, SWP_NOACTIVATE);
    }

    unsafe fn show_launcher() {
        LAUNCHER_SELECTION = 0;
        let mut cursor = POINT::default();
        GetCursorPos(&mut cursor);
        position_popup_on_monitor(LAUNCHER, MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST));
        ShowWindow(LAUNCHER, SW_SHOW);
        SetForegroundWindow(LAUNCHER);
        SetFocus(LAUNCHER_EDIT);
    }

    unsafe fn activate_launcher_selection() {
        let query = launcher_query();
        let entries = launcher_entries(&query);
        if let Some((_, target)) = entries.get(LAUNCHER_SELECTION.max(0) as usize) {
            launch(target);
            ShowWindow(LAUNCHER, SW_HIDE);
        }
    }

    unsafe fn handle_launcher_key(msg: &MSG) -> bool {
        if msg.message != WM_KEYDOWN || msg.hwnd != LAUNCHER_EDIT {
            return false;
        }
        let entries = launcher_entries(&launcher_query());
        let max = entries.len().min(10).saturating_sub(1) as i32;
        match msg.wParam.0 as u32 {
            value if value == VK_UP.0 as u32 => {
                LAUNCHER_SELECTION = (LAUNCHER_SELECTION - 1).max(0);
            }
            value if value == VK_DOWN.0 as u32 => {
                LAUNCHER_SELECTION = (LAUNCHER_SELECTION + 1).min(max);
            }
            value if value == VK_HOME.0 as u32 => {
                LAUNCHER_SELECTION = 0;
            }
            value if value == VK_END.0 as u32 => {
                LAUNCHER_SELECTION = max;
            }
            value if value == VK_PRIOR.0 as u32 => {
                LAUNCHER_SELECTION = (LAUNCHER_SELECTION - 3).max(0);
            }
            value if value == VK_NEXT.0 as u32 => {
                LAUNCHER_SELECTION = (LAUNCHER_SELECTION + 3).min(max);
            }
            value if value == VK_RETURN.0 as u32 => {
                activate_launcher_selection();
            }
            value if value == 'D' as u32 && GetKeyState(VK_LCONTROL.0 as i32) < 0 => {
                if let Some((_, target)) = entries.get(LAUNCHER_SELECTION.max(0) as usize) {
                    toggle_favorite(target);
                }
            }
            value if value == VK_ESCAPE.0 as u32 => {
                ShowWindow(LAUNCHER, SW_HIDE);
            }
            _ => return false,
        }
        if !entries.is_empty() {
            notify_accessibility_focus(LAUNCHER, LAUNCHER_SELECTION);
        }
        let _ = InvalidateRect(LAUNCHER, None, false);
        true
    }

    unsafe fn paint_switcher(hwnd: HWND, hdc: HDC) {
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r).unwrap();
        let bg = brush(if CONFIG.dark { 0x1b1e24 } else { 0xf7f9fc });
        FillRect(hdc, &r, bg);
        let _ = DeleteObject(bg);
        text(
            hdc,
            "Window switcher",
            28,
            30,
            if CONFIG.dark { 0xf7f8fa } else { 0x172033 },
            23,
        );
        text(
            hdc,
            "Win + Tab · choose a running window",
            30,
            62,
            if CONFIG.dark { 0x9da6b4 } else { 0x64748b },
            13,
        );
        let count = running_window_count();
        for (index, window) in WINDOW_LIST.iter().take(9).enumerate() {
            let y = 104 + index as i32 * 34;
            if index == SWITCHER_SELECTION {
                let highlight = brush(if CONFIG.dark { 0x303846 } else { 0xe2e8f0 });
                let row = RECT {
                    left: 24,
                    top: y - 6,
                    right: 288,
                    bottom: y + 24,
                };
                FillRect(hdc, &row, highlight);
                let _ = DeleteObject(highlight);
            }
            text(hdc, &format!("{:>2}", index + 1), 30, y + 1, 0x60a5fa, 12);
            let mut win_icon = HICON(
                SendMessageW(*window, WM_GETICON, WPARAM(ICON_SMALL as usize), LPARAM(0)).0
                    as *mut _,
            );
            if win_icon.is_invalid() {
                win_icon = HICON(GetClassLongPtrW(*window, GCLP_HICONSM) as *mut _);
            }
            if !win_icon.is_invalid() {
                let _ = DrawIconEx(hdc, 50, y - 1, win_icon, 18, 18, 0, None, DI_NORMAL);
            } else {
                icon_text(hdc, "\u{E737}", 50, y - 1, 0x60a5fa, 15);
            }
            text(
                hdc,
                &window_title(*window),
                76,
                y,
                if CONFIG.dark { 0xe8ecf2 } else { 0x334155 },
                14,
            );
        }
        if count == 0 {
            text(hdc, "No visible application windows", 34, 112, 0x7f8a9a, 14);
        }
        update_switcher_thumbnail();
    }

    unsafe fn show_switcher() {
        SWITCHER_SELECTION = 0;
        let _ = InvalidateRect(SWITCHER, None, false);
        ShowWindow(SWITCHER, SW_SHOW);
        SetForegroundWindow(SWITCHER);
    }

    unsafe fn paint_audio_mixer(hwnd: HWND, hdc: HDC) {
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r).unwrap();
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
                    bottom: y + 22,
                };
                FillRect(hdc, &selected_row, highlight);
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
    }

    unsafe fn show_audio_mixer() {
        if AUDIO_MIXER.0.is_null() {
            return;
        }
        AUDIO_MIXER_SELECTION = 0;
        let mut cursor = POINT::default();
        GetCursorPos(&mut cursor);
        let x = cursor.x.saturating_sub(AUDIO_MIXER_WIDTH / 2);
        let y = cursor.y.saturating_sub(AUDIO_MIXER_HEIGHT + 12);
        let _ = SetWindowPos(
            AUDIO_MIXER,
            HWND_TOPMOST,
            x,
            y.max(0),
            AUDIO_MIXER_WIDTH,
            AUDIO_MIXER_HEIGHT,
            SWP_NOACTIVATE,
        );
        let _ = InvalidateRect(AUDIO_MIXER, None, false);
        ShowWindow(AUDIO_MIXER, SW_SHOW);
        SetForegroundWindow(AUDIO_MIXER);
        SetFocus(AUDIO_MIXER);
    }

    extern "system" fn audio_mixer_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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

    unsafe fn paint_desktop(hwnd: HWND, hdc: HDC) {
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        let w = r.right - r.left;
        let h = r.bottom - r.top;

        let mut wallpaper_buf = [0u16; 512];
        let mut drawn = false;
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
                    let hbmp = LoadImageW(
                        None,
                        PCWSTR(wallpaper_buf.as_ptr()),
                        IMAGE_BITMAP,
                        0,
                        0,
                        LR_LOADFROMFILE,
                    );
                    if let Ok(handle) = hbmp {
                        let hbitmap = HBITMAP(handle.0);
                        if !hbitmap.is_invalid() {
                            let mem_dc = CreateCompatibleDC(hdc);
                            let old_bmp = SelectObject(mem_dc, hbitmap);
                            let mut bmp_info = BITMAP::default();
                            if GetObjectW(
                                hbitmap,
                                size_of::<BITMAP>() as i32,
                                Some(&mut bmp_info as *mut _ as *mut _),
                            ) > 0
                            {
                                let _ = StretchBlt(
                                    hdc,
                                    0,
                                    0,
                                    w,
                                    h,
                                    mem_dc,
                                    0,
                                    0,
                                    bmp_info.bmWidth,
                                    bmp_info.bmHeight,
                                    SRCCOPY,
                                );
                                drawn = true;
                            }
                            SelectObject(mem_dc, old_bmp);
                            let _ = DeleteDC(mem_dc);
                            let _ = DeleteObject(hbitmap);
                        }
                    }
                }
            }
        }

        if !drawn {
            let steps = 40;
            for i in 0..steps {
                let y1 = (i * h) / steps;
                let y2 = ((i + 1) * h) / steps;
                let t = i as f32 / steps as f32;
                let r_col = (13.0 + t * 15.0) as u32;
                let g_col = (18.0 + t * 8.0) as u32;
                let b_col = (30.0 + t * 35.0) as u32;
                let color = (b_col << 16) | (g_col << 8) | r_col;
                let b = brush(color);
                let band = RECT {
                    left: 0,
                    top: y1,
                    right: w,
                    bottom: y2,
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
    }

    unsafe fn show_desktop_context_menu(hwnd: HWND, x: i32, y: i32) {
        let menu = CreatePopupMenu().unwrap();
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            ID_DESKTOP_TERMINAL,
            PCWSTR(wide("Terminal\tCtrl+Alt+T").as_ptr()),
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

        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD, x, y, 0, hwnd, None);
        let _ = DestroyMenu(menu);

        match cmd.0 as usize {
            ID_DESKTOP_TERMINAL => {
                let local_app = env::var_os("LOCALAPPDATA").map(PathBuf::from);
                let wt = local_app.map(|p| p.join("Microsoft\\WindowsApps\\wt.exe"));
                if let Some(wt_path) = wt.filter(|p| p.exists()) {
                    launch(&wt_path.to_string_lossy());
                } else {
                    launch("powershell.exe");
                }
            }
            ID_DESKTOP_FILES => launch("explorer.exe"),
            ID_DESKTOP_LAUNCHER => show_launcher(),
            ID_DESKTOP_SWITCHER => show_switcher(),
            ID_DESKTOP_DISPLAY => launch("ms-settings:display"),
            ID_DESKTOP_PERSONALIZE => launch("ms-settings:personalization"),
            ID_MENU_SETTINGS => {
                ShowWindow(SETTINGS, SW_SHOW);
                SetForegroundWindow(SETTINGS);
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
            _ => {}
        }
    }

    extern "system" fn desktop_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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

    unsafe fn init_shell_integration(panel_hwnd: HWND, desktop_hwnd: HWND) {
        if !desktop_hwnd.is_invalid() && is_luma_shell_active() {
            let user32 = GetModuleHandleW(PCWSTR(wide("user32.dll").as_ptr())).unwrap();
            if let Some(set_shell) = windows::Win32::System::LibraryLoader::GetProcAddress(
                user32,
                windows::core::s!("SetShellWindow"),
            ) {
                let func: unsafe extern "system" fn(HWND) -> BOOL = std::mem::transmute(set_shell);
                let ok = func(desktop_hwnd);
                log_event(&format!("SetShellWindow result: {}", ok.as_bool()));
            }
        }

        let user32 = GetModuleHandleW(PCWSTR(wide("user32.dll").as_ptr())).unwrap();
        if let Some(reg_hook) = windows::Win32::System::LibraryLoader::GetProcAddress(
            user32,
            windows::core::s!("RegisterShellHookWindow"),
        ) {
            let func: unsafe extern "system" fn(HWND) -> BOOL = std::mem::transmute(reg_hook);
            let ok = func(panel_hwnd);
            log_event(&format!("RegisterShellHookWindow result: {}", ok.as_bool()));
        }

        SHELLHOOK_MSG = RegisterWindowMessageW(PCWSTR(wide("SHELLHOOK").as_ptr()));
        register_volume_callback();
        let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
        POWER_NOTIFY = RegisterPowerSettingNotification(
            HANDLE(panel_hwnd.0),
            &brightness_guid,
            REGISTER_NOTIFICATION_FLAGS(0),
        )
        .unwrap_or(HPOWERNOTIFY(0));
        query_system_brightness();
    }

    unsafe fn cleanup_shell_integration(hwnd: HWND) {
        unregister_volume_callback();
        if POWER_NOTIFY.0 != 0 {
            let _ = UnregisterPowerSettingNotification(POWER_NOTIFY);
            POWER_NOTIFY = HPOWERNOTIFY(0);
        }
        let user32 = GetModuleHandleW(PCWSTR(wide("user32.dll").as_ptr())).unwrap();
        if let Some(dereg_hook) = windows::Win32::System::LibraryLoader::GetProcAddress(
            user32,
            windows::core::s!("DeregisterShellHookWindow"),
        ) {
            let func: unsafe extern "system" fn(HWND) -> BOOL = std::mem::transmute(dereg_hook);
            let _ = func(hwnd);
        }
    }

    unsafe extern "system" fn ll_keyboard_proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
        if code >= 0 {
            let kbd = *(l.0 as *const KBDLLHOOKSTRUCT);
            let vk = kbd.vkCode as u16;
            let is_win = vk == VK_LWIN.0 || vk == VK_RWIN.0;

            // Handle Volume keys immediately
            let is_vol_key = vk == VK_VOLUME_MUTE.0 || vk == VK_VOLUME_DOWN.0 || vk == VK_VOLUME_UP.0;
            if is_vol_key {
                if w.0 as u32 == WM_KEYDOWN || w.0 as u32 == WM_SYSKEYDOWN {
                    if vk == VK_VOLUME_MUTE.0 {
                        toggle_audio_mute();
                    } else if vk == VK_VOLUME_DOWN.0 {
                        adjust_master_volume(-0.02);
                    } else if vk == VK_VOLUME_UP.0 {
                        adjust_master_volume(0.02);
                    }
                    invalidate_shell_surfaces();
                    show_volume_osd();
                }
                return LRESULT(1);
            }

            // Handle Brightness hotkeys (Win+F5/F6 or Win+Alt+Down/Up)
            if WIN_KEY_DOWN {
                let is_alt_down = (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
                let is_bright_down = vk == VK_F5.0 || (is_alt_down && vk == VK_DOWN.0);
                let is_bright_up = vk == VK_F6.0 || (is_alt_down && vk == VK_UP.0);

                if is_bright_down || is_bright_up {
                    WIN_COMBO_USED = true;
                    if w.0 as u32 == WM_KEYDOWN || w.0 as u32 == WM_SYSKEYDOWN {
                        let delta = if is_bright_down { -5 } else { 5 };
                        let new_b = adjust_system_brightness(delta);
                        show_brightness_osd(new_b);
                    }
                    return LRESULT(1);
                }
            }

            // Win combos we want to suppress so they don't reach Explorer/Windows shell.
            // VK_D = 0x44 (show desktop), VK_TAB = task view (we handle switcher ourselves).
            const VK_D: u16 = 0x44;
            let is_blocked_win_combo =
                WIN_KEY_DOWN && (vk == VK_D || vk == VK_TAB.0);

            if w.0 as u32 == WM_KEYDOWN || w.0 as u32 == WM_SYSKEYDOWN {
                if is_win {
                    WIN_KEY_DOWN = true;
                    WIN_COMBO_USED = false;
                } else if WIN_KEY_DOWN {
                    WIN_COMBO_USED = true;
                    // Intercept and handle Win+D (show desktop) and Win+Tab (task view)
                    if is_blocked_win_combo {
                        // Send dummy keypress so Windows marks a combo as used even though D/Tab is swallowed
                        keybd_event(0xFF, 0, Default::default(), 0);
                        keybd_event(0xFF, 0, KEYEVENTF_KEYUP, 0);
                        if vk == VK_D {
                            toggle_show_desktop();
                        } else if vk == VK_TAB.0 {
                            show_switcher();
                        }
                        return LRESULT(1);
                    }
                }
            } else if w.0 as u32 == WM_KEYUP || w.0 as u32 == WM_SYSKEYUP {
                if is_win {
                    if WIN_KEY_DOWN && !WIN_COMBO_USED {
                        // Send dummy keypress (0xFF / VK_NONAME) to tell Windows a key combo occurred.
                        // This masks the Windows key release and prevents Windows 11 from opening its native Start Menu.
                        keybd_event(0xFF, 0, Default::default(), 0);
                        keybd_event(0xFF, 0, KEYEVENTF_KEYUP, 0);

                        if IsWindowVisible(LAUNCHER).as_bool() {
                            ShowWindow(LAUNCHER, SW_HIDE);
                        } else {
                            show_launcher();
                        }
                    }
                    WIN_KEY_DOWN = false;
                    WIN_COMBO_USED = false;
                    // Note: Do NOT return LRESULT(1) here!
                    // Returning LRESULT(1) swallows the WM_KEYUP event, preventing Windows
                    // from updating its asynchronous keystate table. That causes the Windows key
                    // to stay logically held down (so every subsequent keypress acted as Win + Key).
                    // Passing WM_KEYUP through to CallNextHookEx allows Windows to clear the pressed
                    // state, while the dummy 0xFF keypress above already prevented the Start Menu.
                } else if is_blocked_win_combo {
                    // Also swallow the key-up for blocked combos
                    return LRESULT(1);
                }
            }
        }
        CallNextHookEx(KEYBOARD_HOOK, code, w, l)
    }

    unsafe fn init_keyboard_hook() {
        KEYBOARD_HOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(ll_keyboard_proc),
            HINSTANCE(null_mut()),
            0,
        )
        .unwrap_or(HHOOK(null_mut()));
    }

    unsafe fn cleanup_keyboard_hook() {
        if !KEYBOARD_HOOK.is_invalid() {
            let _ = UnhookWindowsHookEx(KEYBOARD_HOOK);
            KEYBOARD_HOOK = HHOOK(null_mut());
        }
    }

    extern "system" fn panel_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_NCHITTEST => {
                    let mut point = POINT {
                        x: ((l.0 & 0xffff) as u16 as i16) as i32,
                        y: (((l.0 >> 16) & 0xffff) as u16 as i16) as i32,
                    };
                    let _ = ScreenToClient(hwnd, &mut point);
                    let mut client = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client);
                    let interactive = (10..175).contains(&point.x)
                        || point.x > client.right - 74
                        || panel_applet_at(hwnd, point.x).is_some();
                    if interactive {
                        LRESULT(HTCLIENT as isize)
                    } else {
                        LRESULT(HTTRANSPARENT as isize)
                    }
                }
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    let h = BeginPaint(hwnd, &mut ps);
                    paint_panel(hwnd, h);
                    EndPaint(hwnd, &ps);
                    LRESULT(0)
                }
                WM_LBUTTONUP => {
                    let x = (l.0 as u32 & 0xffff) as i32;
                    let mut client = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client);
                    if (10..76).contains(&x) {
                        show_launcher();
                    } else if (78..174).contains(&x) {
                        position_popup_on_monitor(
                            SWITCHER,
                            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                        );
                        show_switcher();
                    } else if x >= client.right - 38 {
                        show_power_menu(hwnd, client.right - 180, PANEL_HEIGHT + 4);
                    } else if x >= client.right - 74 && x < client.right - 38 {
                        position_popup_on_monitor(
                            SETTINGS,
                            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                        );
                        ShowWindow(SETTINGS, SW_SHOW);
                        SetForegroundWindow(SETTINGS);
                    } else if let Some(id) = panel_applet_at(hwnd, x) {
                        match id {
                            0 => launch("ms-settings:dateandtime"),
                            1 => launch("ms-settings:network-wifi"),
                            2 => show_audio_mixer(),
                            3 => launch("ms-settings:bluetooth"),
                            4 => launch("ms-settings:batterysaver"),
                            5 => launch("ms-settings:notifications"),
                            6 => toggle_media(),
                            _ => {}
                        }
                    }
                    LRESULT(0)
                }
                m if SHELLHOOK_MSG != 0 && m == SHELLHOOK_MSG => {
                    let _ = InvalidateRect(DOCK, None, false);
                    for &secondary in SECONDARY_DOCKS.iter().take(SECONDARY_COUNT) {
                        let _ = InvalidateRect(secondary, None, false);
                    }
                    LRESULT(0)
                }
                WM_HOTKEY => {
                    if w.0 as i32 == HOTKEY_LAUNCHER {
                        show_launcher();
                    } else if w.0 as i32 == HOTKEY_SWITCHER {
                        show_switcher();
                    } else if w.0 as i32 == HOTKEY_WORKSPACE_PREVIOUS {
                        switch_workspace(VK_LEFT);
                    } else if w.0 as i32 == HOTKEY_WORKSPACE_NEXT {
                        switch_workspace(VK_RIGHT);
                    } else if w.0 as i32 == HOTKEY_MOVE_WINDOW_PREVIOUS {
                        move_window_workspace(VK_LEFT);
                    } else if w.0 as i32 == HOTKEY_MOVE_WINDOW_NEXT {
                        move_window_workspace(VK_RIGHT);
                    } else {
                        toggle_dock();
                    }
                    LRESULT(0)
                }
                x if x == WM_APP + 40 => {
                    match w.0 as u32 {
                        ABN_POSCHANGED => {
                            if hwnd == PANEL {
                                reposition_primary_panel();
                                position_dock();
                                rebuild_secondary_monitors();
                            } else {
                                reposition_secondary_panel(hwnd);
                            }
                        }
                        ABN_FULLSCREENAPP => {
                            let is_fullscreen = l.0 != 0;
                            set_docks_visible(!is_fullscreen);
                            let z_order = if is_fullscreen { HWND_BOTTOM } else { HWND_TOPMOST };
                            let _ = SetWindowPos(PANEL, z_order, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                            for &secondary in SECONDARY_PANELS.iter().take(SECONDARY_COUNT) {
                                let _ = SetWindowPos(secondary, z_order, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                            }
                        }
                        _ => {}
                    }
                    LRESULT(0)
                }
                WM_POWERBROADCAST => {
                    if w.0 as u32 == PBT_POWERSETTINGCHANGE && l.0 != 0 {
                        let setting = &*(l.0 as *const POWERBROADCAST_SETTING);
                        let brightness_guid = GUID::from_u128(0xaded5e82_b909_4619_9949_f5d71dac0bcb);
                        if setting.PowerSetting == brightness_guid && setting.DataLength >= 4 {
                            let val = std::ptr::read_unaligned(setting.Data.as_ptr() as *const u32);
                            show_brightness_osd(val as u8);
                        }
                    }
                    LRESULT(1)
                }
                x if x == WM_APP_AUDIO_CHANGE => {
                    invalidate_shell_surfaces();
                    show_volume_osd();
                    LRESULT(0)
                }
                WM_MOUSEWHEEL => {
                    let delta = ((w.0 as u32 >> 16) & 0xffff) as i16;
                    let x = ((l.0 & 0xffff) as u16 as i16) as i32;
                    let mut pt = POINT { x, y: 0 };
                    let _ = ScreenToClient(hwnd, &mut pt);
                    if let Some(id) = panel_applet_at(hwnd, pt.x) {
                        if id == 2 {
                            if delta > 0 {
                                adjust_master_volume(0.02);
                            } else if delta < 0 {
                                adjust_master_volume(-0.02);
                            }
                            invalidate_shell_surfaces();
                            show_volume_osd();
                        }
                    }
                    LRESULT(0)
                }
                WM_TIMER => {
                    hide_windows_shell_windows();
                    update_auto_hide();
                    refresh_app_index_if_due();
                    invalidate_shell_surfaces();
                    LRESULT(0)
                }
                WM_DISPLAYCHANGE | WM_DPICHANGED => {
                    reposition_primary_panel();
                    position_dock();
                    rebuild_secondary_monitors();
                    invalidate_shell_surfaces();
                    restyle_shell_windows();
                    LRESULT(0)
                }
                WM_SETTINGCHANGE | WM_THEMECHANGED => {
                    restyle_shell_windows();
                    invalidate_shell_surfaces();
                    LRESULT(0)
                }
                WM_QUERYENDSESSION => LRESULT(1),
                WM_ENDSESSION if w.0 != 0 => {
                    log_event("Windows session shutdown received");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_ERASEBKGND => LRESULT(1),
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
    }
    extern "system" fn dock_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        unsafe {
            match msg {
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
                        update_auto_hide();
                        let _ = InvalidateRect(hwnd, None, false);
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
                            let _ = InvalidateRect(hwnd, None, false);
                        }
                    }
                    LRESULT(0)
                }
                WM_MOUSEMOVE => {
                    let x = (l.0 as u32 & 0xffff) as i32;
                    DOCK_POINTER_X = x;
                    let spacing = CONFIG.spacing.max(1);
                    let next = if x >= 0 && x < 5 * spacing {
                        x / spacing
                    } else {
                        -1
                    };
                    if DRAGGED_DOCK_ICON >= 0 && next >= 0 && next != DRAGGED_DOCK_ICON {
                        reorder_dock_icon(DRAGGED_DOCK_ICON, next);
                        DRAGGED_DOCK_ICON = next;
                        DOCK_DRAG_MOVED = true;
                    }
                    if HOVERED_DOCK_ICON != next {
                        HOVERED_DOCK_ICON = next;
                        notify_accessibility_focus(hwnd, next);
                        let _ = InvalidateRect(hwnd, None, false);
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
                    let spacing = CONFIG.spacing.max(1);
                    if y <= 46 && (0..5 * spacing).contains(&x) {
                        DRAGGED_DOCK_ICON = x / spacing;
                        DOCK_DRAG_MOVED = false;
                        SetCapture(hwnd);
                    }
                    LRESULT(0)
                }
                WM_MOUSE_LEAVE => {
                    HOVERED_DOCK_ICON = -1;
                    DOCK_POINTER_X = -1;
                    let _ = InvalidateRect(hwnd, None, false);
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
                            HOVERED_DOCK_ICON = (HOVERED_DOCK_ICON + 1).min(4);
                            notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                        }
                        value if value == VK_HOME.0 as u32 => {
                            HOVERED_DOCK_ICON = 0;
                            notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                        }
                        value if value == VK_END.0 as u32 => {
                            HOVERED_DOCK_ICON = 4;
                            notify_accessibility_focus(hwnd, HOVERED_DOCK_ICON);
                        }
                        value if value == VK_RETURN.0 as u32 => {
                            activate_dock_icon(HOVERED_DOCK_ICON)
                        }
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
                    let spacing = CONFIG.spacing.max(1);
                    if (0..5 * spacing).contains(&x) {
                        let launcher_index = (x / spacing).clamp(0, 4) as usize;
                        if let Some(item_id) = CONFIG.dock_order.get(launcher_index).copied() {
                            activate_dock_item(item_id);
                        }
                    }
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
    }
    extern "system" fn settings_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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
                        ID_ICON_SMALL => CONFIG.icon_size = 20,
                        ID_ICON_MEDIUM => CONFIG.icon_size = 25,
                        ID_ICON_LARGE => CONFIG.icon_size = 30,
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
    extern "system" fn launcher_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_CREATE => {
                    LAUNCHER_EDIT = CreateWindowExW(
                        WS_EX_CLIENTEDGE,
                        PCWSTR(wide("EDIT").as_ptr()),
                        PCWSTR(wide("").as_ptr()),
                        WINDOW_STYLE(
                            WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | ES_AUTOHSCROLL as u32,
                        ),
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
    extern "system" fn switcher_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
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

    unsafe fn register_class(instance: HINSTANCE, name: &str, proc: WNDPROC) {
        let n = wide(name);
        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            hInstance: instance,
            lpszClassName: PCWSTR(n.as_ptr()),
            lpfnWndProc: proc,
            hbrBackground: HBRUSH(null_mut()),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        RegisterClassW(&wc);
    }

    pub unsafe fn run() {
        std::panic::set_hook(Box::new(|info| {
            let msg = format!("PANIC: {}\n", info);
            let _ = fs::write("D:\\project\\panic.log", &msg);
            let _ = fs::write("panic.log", &msg);
        }));
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        log_event("starting shell");
        let _instance_guard = match acquire_instance() {
            Some(handle) => handle,
            None => return,
        };
        let winrt_initialized = RoInitialize(RO_INIT_MULTITHREADED).is_ok();
        let com_initialized = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        CONFIG = load_config();
        let instance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
        // Kill Explorer so LumaShell fully replaces the Windows taskbar and shell
        if is_explorer_running() {
            kill_explorer();
        }
        hide_windows_shell_windows();
        register_class(instance, "LumaDesktop", Some(desktop_proc));
        register_class(instance, "LumaPanel", Some(panel_proc));
        register_class(instance, "LumaDock", Some(dock_proc));
        register_class(instance, "LumaSettings", Some(settings_proc));
        register_class(instance, "LumaLauncher", Some(launcher_proc));
        register_class(instance, "LumaSwitcher", Some(switcher_proc));
        register_class(instance, "LumaAudioMixer", Some(audio_mixer_proc));
        register_class(instance, "LumaOSD", Some(osd_proc));
        APP_INDEX = Some(build_app_index());
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vcx = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vcy = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        let show_desktop = CONFIG.desktop_enabled;
        DESKTOP = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaDesktop").as_ptr()),
            PCWSTR(wide("Luma Desktop").as_ptr()),
            WS_POPUP
                | if show_desktop {
                    WS_VISIBLE
                } else {
                    WINDOW_STYLE(0)
                },
            vx,
            vy,
            vcx,
            vcy,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        let _ = SetWindowPos(
            DESKTOP,
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        PANEL = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaPanel").as_ptr()),
            PCWSTR(wide("Luma Panel").as_ptr()),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            sw,
            PANEL_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        let _ = SetTimer(PANEL, 1, 250, None);
        DOCK = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaDock").as_ptr()),
            PCWSTR(wide("Luma Dock").as_ptr()),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            DOCK_WIDTH,
            DOCK_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        let _ = SetTimer(DOCK, 2, 500, None);
        let _ = SetTimer(DOCK, 3, 16, None);
        SETTINGS = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaSettings").as_ptr()),
            PCWSTR(wide("Luma Settings").as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN,
            (sw - SETTINGS_WIDTH) / 2,
            (sh - SETTINGS_HEIGHT) / 2,
            SETTINGS_WIDTH,
            SETTINGS_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        LAUNCHER = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaLauncher").as_ptr()),
            PCWSTR(wide("Luma Applications").as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            (sw - LAUNCHER_WIDTH) / 2,
            (sh - LAUNCHER_HEIGHT) / 2,
            LAUNCHER_WIDTH,
            LAUNCHER_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        SWITCHER = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            PCWSTR(wide("LumaSwitcher").as_ptr()),
            PCWSTR(wide("Luma Window Switcher").as_ptr()),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            (sw - SWITCHER_WIDTH) / 2,
            (sh - SWITCHER_HEIGHT) / 2,
            SWITCHER_WIDTH,
            SWITCHER_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        AUDIO_MIXER = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            PCWSTR(wide("LumaAudioMixer").as_ptr()),
            PCWSTR(wide("Luma Audio Mixer").as_ptr()),
            WS_POPUP,
            0,
            0,
            AUDIO_MIXER_WIDTH,
            AUDIO_MIXER_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        OSD_WINDOW = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
            PCWSTR(wide("LumaOSD").as_ptr()),
            PCWSTR(wide("Luma OSD").as_ptr()),
            WS_POPUP,
            0,
            0,
            OSD_WIDTH,
            OSD_HEIGHT,
            None,
            None,
            instance,
            None,
        )
        .unwrap();
        style_window(DOCK);
        let settings_corner = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            SETTINGS,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &settings_corner as *const _ as *const _,
            size_of::<i32>() as u32,
        );
        let settings_dark: BOOL = CONFIG.dark.into();
        let _ = DwmSetWindowAttribute(
            SETTINGS,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &settings_dark as *const _ as *const _,
            size_of::<BOOL>() as u32,
        );
        style_window(LAUNCHER);
        style_window(SWITCHER);
        style_window(AUDIO_MIXER);
        if !OSD_WINDOW.is_invalid() {
            style_window(OSD_WINDOW);
        }
        let primary_monitor = MonitorFromWindow(PANEL, MONITOR_DEFAULTTONEAREST);
        let mut primary_info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(primary_monitor, &mut primary_info);
        let appbar = register_panel_appbar_for_rect(PANEL, primary_info.rcMonitor);
        PRIMARY_APPBAR = Some(appbar);
        position_dock();
        rebuild_secondary_monitors();
        log_event("shell windows ready");
        let _ = RegisterHotKey(PANEL, HOTKEY_DOCK, MOD_WIN | MOD_ALT, VK_SPACE.0 as u32);
        let _ = RegisterHotKey(PANEL, HOTKEY_SWITCHER, MOD_WIN, VK_TAB.0 as u32);
        if RegisterHotKey(
            PANEL,
            HOTKEY_WORKSPACE_PREVIOUS,
            MOD_WIN | MOD_ALT,
            VK_LEFT.0 as u32,
        )
        .is_err()
        {
            log_event("workspace previous hotkey unavailable");
        }
        if RegisterHotKey(
            PANEL,
            HOTKEY_WORKSPACE_NEXT,
            MOD_WIN | MOD_ALT,
            VK_RIGHT.0 as u32,
        )
        .is_err()
        {
            log_event("workspace next hotkey unavailable");
        }
        if RegisterHotKey(
            PANEL,
            HOTKEY_MOVE_WINDOW_PREVIOUS,
            MOD_WIN | MOD_ALT | MOD_SHIFT,
            VK_LEFT.0 as u32,
        )
        .is_err()
        {
            log_event("move-window previous hotkey unavailable");
        }
        if RegisterHotKey(
            PANEL,
            HOTKEY_MOVE_WINDOW_NEXT,
            MOD_WIN | MOD_ALT | MOD_SHIFT,
            VK_RIGHT.0 as u32,
        )
        .is_err()
        {
            log_event("move-window next hotkey unavailable");
        }
        if RegisterHotKey(PANEL, HOTKEY_LAUNCHER, MOD_WIN | MOD_ALT, 'L' as u32).is_err() {
            log_event("launcher hotkey unavailable");
        }
        init_shell_integration(PANEL, DESKTOP);
        init_keyboard_hook();
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if handle_launcher_key(&msg) {
                continue;
            }
            if !SETTINGS.is_invalid()
                && IsWindowVisible(SETTINGS).as_bool()
                && IsDialogMessageW(SETTINGS, &msg).as_bool()
            {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        cleanup_keyboard_hook();
        cleanup_shell_integration(PANEL);
        show_windows_shell_windows();
        // Restart Explorer on clean exit so the user gets their normal shell back
        if !is_explorer_running() {
            log_event("restarting explorer.exe on clean exit");
            launch("explorer.exe");
        }
        if !DESKTOP.is_invalid() {
            let _ = DestroyWindow(DESKTOP);
            DESKTOP = HWND(null_mut());
        }
        if !OSD_WINDOW.is_invalid() {
            let _ = DestroyWindow(OSD_WINDOW);
            OSD_WINDOW = HWND(null_mut());
        }
        destroy_secondary_monitors();
        clear_switcher_thumbnail();
        clear_dock_icon_cache();
        if let Some(mut appbar) = PRIMARY_APPBAR.take() {
            unregister_panel_appbar(&mut appbar);
        }
        UnregisterHotKey(PANEL, HOTKEY_DOCK);
        UnregisterHotKey(PANEL, HOTKEY_SWITCHER);
        UnregisterHotKey(PANEL, HOTKEY_WORKSPACE_PREVIOUS);
        UnregisterHotKey(PANEL, HOTKEY_WORKSPACE_NEXT);
        UnregisterHotKey(PANEL, HOTKEY_MOVE_WINDOW_PREVIOUS);
        UnregisterHotKey(PANEL, HOTKEY_MOVE_WINDOW_NEXT);
        UnregisterHotKey(PANEL, HOTKEY_LAUNCHER);
        if winrt_initialized {
            RoUninitialize();
        }
        if com_initialized {
            CoUninitialize();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn applet_order_rejects_unknown_and_duplicate_entries() {
            assert_eq!(
                parse_order("media,clock,media,unknown"),
                [6, 0, 1, 2, 3, 4, 5]
            );
        }

        #[test]
        fn osd_brightness_clamping() {
            unsafe {
                CURRENT_BRIGHTNESS = 98;
                let b = adjust_system_brightness(5);
                assert_eq!(b, 100);
                let b = adjust_system_brightness(-120);
                assert_eq!(b, 0);
                let _ = adjust_system_brightness(100); // restore to 100
            }
        }

        #[test]
        fn osd_progress_bar_fill_calculation() {
            let track_w = 148;
            for level in 0..=100 {
                let fill_w = if level > 0 {
                    ((track_w * level as i32) / 100).max(8).min(track_w)
                } else {
                    0
                };
                assert!(fill_w <= track_w);
                if level > 0 {
                    assert!(fill_w >= 8);
                }
            }
        }

        #[test]
        fn dock_order_fills_missing_items_deterministically() {
            assert_eq!(
                parse_dock_order("browser,files,browser,unknown"),
                [3, 1, 0, 2, 4]
            );
            assert_eq!(
                parse_dock_order("home,files,terminal,browser,mail,settings"),
                [0, 1, 2, 3, 4]
            );
        }

        #[test]
        fn order_serialization_matches_config_names() {
            assert_eq!(
                order_text([0, 1, 2, 3, 4, 5, 6]),
                "clock,network,volume,bluetooth,battery,notifications,media"
            );
            assert_eq!(
                dock_order_text([0, 1, 2, 3, 4]),
                "home,files,terminal,browser,settings"
            );
        }

        #[test]
        fn panel_zones_default_unknown_values_to_right() {
            assert_eq!(
                parse_zones("clock:left,network:center,volume:invalid,unknown:left"),
                [0, 1, 2, 2, 2, 2, 2]
            );
            assert_eq!(
                zones_text([0, 1, 2, 2, 2, 2, 2]),
                "clock:left,network:center,volume:right,bluetooth:right,battery:right,notifications:right,media:right"
            );
        }

        #[test]
        fn config_default_has_desktop_enabled_and_version_7() {
            let config = default_config();
            assert!(config.desktop_enabled);
            assert_eq!(CURRENT_CONFIG_VERSION, 7);
        }
    }
}

#[cfg(target_os = "windows")]
fn main() {
    unsafe {
        shell::run();
    }
}
#[cfg(not(target_os = "windows"))]
fn main() {
    println!("Luma Shell is a native Windows 11 shell. Build this project on Windows to run it.");
}
