use super::*;

pub unsafe fn toggle_show_desktop() {
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

pub unsafe fn hide_windows_shell_windows() {
    let class = wide("Shell_TrayWnd");
    if let Ok(tray) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
        if !tray.is_invalid() && tray != PANEL && IsWindowVisible(tray).as_bool() {
            let _ = ShowWindow(tray, SW_HIDE);
        }
    }
    let class_sec = wide("Shell_SecondaryTrayWnd");
    let mut prev = HWND(null_mut());
    loop {
        match FindWindowExW(
            HWND(null_mut()),
            prev,
            PCWSTR(class_sec.as_ptr()),
            PCWSTR::null(),
        ) {
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

pub unsafe fn show_windows_shell_windows() {
    let class = wide("Shell_TrayWnd");
    if let Ok(tray) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
        if !tray.is_invalid() && tray != PANEL {
            let _ = ShowWindow(tray, SW_SHOW);
        }
    }
    let class_sec = wide("Shell_SecondaryTrayWnd");
    let mut prev = HWND(null_mut());
    loop {
        match FindWindowExW(
            HWND(null_mut()),
            prev,
            PCWSTR(class_sec.as_ptr()),
            PCWSTR::null(),
        ) {
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

pub fn is_luma_shell_active() -> bool {
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

pub unsafe fn is_explorer_running() -> bool {
    let class = wide("Shell_TrayWnd");
    if let Ok(hwnd) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
        if !hwnd.is_invalid() && hwnd != PANEL {
            return true;
        }
    }
    false
}

pub unsafe fn kill_explorer() {
    log_event("terminating explorer.exe");
    let class = wide("Shell_TrayWnd");
    if let Ok(hwnd) = FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) {
        if !hwnd.is_invalid() && hwnd != PANEL {
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

pub fn enable_luma_shell_replacement() -> bool {
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
    let bytes =
        unsafe { std::slice::from_raw_parts(cmd_wide.as_ptr() as *const u8, cmd_wide.len() * 2) };
    let set_res =
        unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes)) };
    let auto_restart_name = wide("AutoRestartShell");
    let zero: u32 = 0;
    let zero_bytes = unsafe { std::slice::from_raw_parts(&zero as *const u32 as *const u8, 4) };
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

pub fn restore_explorer_shell() -> bool {
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
        let _ = unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), 0, REG_SZ, Some(bytes)) };
        let auto_restart_name = wide("AutoRestartShell");
        let _ = unsafe { RegDeleteValueW(key, PCWSTR(auto_restart_name.as_ptr())) };
        unsafe { RegCloseKey(key) };
    }
    log_event("explorer shell restored in registry");
    unsafe {
        show_windows_shell_windows();
    }
    launch("explorer.exe");
    true
}

pub fn launch(target: &str) {
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
    // When Luma is running as the shell, Explorer is usually not running to
    // broker `ms-settings:` protocol URLs. Try the normal ShellExecute path
    // first, then use ShellExecute_RunDLL as a shell-independent fallback.
    if target.starts_with("ms-settings:") {
        let v = wide(target);
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(wide("open").as_ptr()),
                PCWSTR(v.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        if (result.0 as usize) > 32 {
            return;
        }
        let verb = wide("open");
        let file = wide("rundll32.exe");
        let args = wide(&format!("shell32.dll,ShellExec_RunDLL {}", target));
        unsafe {
            let _ = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(file.as_ptr()),
                PCWSTR(args.as_ptr()),
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
pub fn exit_to_explorer() {
    let _ = restore_explorer_shell();
    std::process::exit(0);
}

pub fn restore_explorer() {
    let _ = restore_explorer_shell();
    let Some(local_app_data) = env::var_os("LOCALAPPDATA") else {
        std::process::exit(0);
    };
    let root = PathBuf::from(local_app_data).join("LumaShell");
    let script = root.join("restore-explorer.ps1");
    let backup = root.join("explorer-shell.backup.json");
    if !script.exists() {
        std::process::exit(0);
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
    std::process::exit(0);
}
