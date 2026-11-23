use std::path::PathBuf;
use std::{env, fs};

#[derive(Clone, Copy, PartialEq)]
pub enum DockPosition {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy)]
pub struct Config {
    pub dark: bool,
    pub show_clock: bool,
    pub show_network: bool,
    pub show_volume: bool,
    pub show_battery: bool,
    pub show_notifications: bool,
    pub show_bluetooth: bool,
    pub show_media: bool,
    pub position: DockPosition,
    pub auto_hide: bool,
    pub icon_size: i32,
    pub magnification: u8,
    pub transparency: u8,
    pub spacing: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub clock_24h: bool,
    pub date_format: u8,
    pub desktop_enabled: bool,
    pub applet_order: [u8; 7],
    pub panel_zones: [u8; 7],
    pub dock_order: [u8; 5],
    pub accent_color: u32,
    pub wallpaper_style: u8,
    pub adaptive_dock_transparency: bool,
    pub panel_transparent: bool,
}

pub fn config_path() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("LumaShell")
        .join("settings.ini")
}
pub fn config_backup_path() -> PathBuf {
    config_path().with_file_name("settings.ini.bak")
}

pub fn parse_order(raw: &str) -> [u8; 7] {
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
pub fn order_text(order: [u8; 7]) -> String {
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
pub fn parse_zones(raw: &str) -> [u8; 7] {
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
pub fn zones_text(zones: [u8; 7]) -> String {
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
pub fn parse_dock_order(raw: &str) -> [u8; 5] {
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
pub fn dock_order_text(order: [u8; 5]) -> String {
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
pub fn load_config() -> Config {
    let mut c = unsafe { super::CONFIG };
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
                "icon_size" => c.icon_size = v.parse().unwrap_or(42).clamp(24, 64),
                "magnification" => c.magnification = v.parse().unwrap_or(32).clamp(0, 100) as u8,
                "transparency" => c.transparency = v.parse().unwrap_or(148).clamp(40, 230) as u8,
                "spacing" => c.spacing = v.parse().unwrap_or(56).clamp(36, 96),
                "offset_x" => c.offset_x = v.parse().unwrap_or(0).clamp(-200, 200),
                "offset_y" => c.offset_y = v.parse().unwrap_or(0).clamp(-100, 100),
                "clock_24h" => c.clock_24h = v == "true",
                "desktop" => c.desktop_enabled = v == "true",
                "adaptive_dock_transparency" => c.adaptive_dock_transparency = v == "true",
                "panel_style" => c.panel_transparent = v == "transparent",
                "accent_color" => {
                    let clean = v.trim().trim_start_matches('#');
                    if let Ok(parsed) = u32::from_str_radix(clean, 16) {
                        c.accent_color = parsed & 0x00FF_FFFF;
                    }
                }
                "wallpaper_style" => c.wallpaper_style = v.parse().unwrap_or(0).min(4),
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
        if version > super::CURRENT_CONFIG_VERSION {
            super::log_event("unsupported future configuration version; defaults loaded");
            return default_config();
        }
        if version < 9 {
            // Upgrade legacy config to COSMIC Orbital defaults (42px icons, 56px spacing, 148 translucent glass)
            c.icon_size = 42;
            c.magnification = 32;
            c.transparency = 148;
            c.spacing = 56;
            c.adaptive_dock_transparency = true;
            c.panel_transparent = true;
            c.panel_zones = [1, 2, 2, 2, 2, 2, 0];
        }
    }
    c
}
pub fn save_config() {
    let c = unsafe { super::CONFIG };
    let position = match c.position {
        DockPosition::Left => "left",
        DockPosition::Right => "right",
        DockPosition::Center => "center",
    };
    let text = format!(
        "version={}\ndark={}\ndesktop={}\nclock={}\nnetwork={}\nvolume={}\nbattery={}\nnotifications={}\nbluetooth={}\nmedia={}\nposition={}\nauto_hide={}\nicon_size={}\nmagnification={}\ntransparency={}\nspacing={}\noffset_x={}\noffset_y={}\nclock_24h={}\ndate_format={}\norder={}\nzones={}\ndock_order={}\naccent_color=#{:06x}\nwallpaper_style={}\nadaptive_dock_transparency={}\npanel_style={}\n",
        super::CURRENT_CONFIG_VERSION,
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
        dock_order_text(c.dock_order),
        c.accent_color & 0x00FF_FFFF,
        c.wallpaper_style,
        c.adaptive_dock_transparency,
        if c.panel_transparent { "transparent" } else { "full" }
    );
    if let Some(parent) = config_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
    let path = config_path();
    let temp = path.with_file_name("settings.ini.tmp");
    let backup = config_backup_path();
    if fs::write(&temp, text).is_err() {
        super::log_event("configuration write failed");
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
        super::log_event("configuration commit failed; backup restored");
        return;
    }
    super::log_event("configuration saved with backup");
}
pub fn restore_config_backup() -> bool {
    let path = config_path();
    let backup = config_backup_path();
    if !backup.exists() {
        super::log_event("configuration backup restore requested but no backup exists");
        return false;
    }
    let temp = path.with_file_name("settings.ini.restore.tmp");
    if fs::copy(&backup, &temp).is_err() {
        super::log_event("configuration backup staging failed");
        return false;
    }
    let previous = path.with_file_name("settings.ini.restore-old");
    if path.exists() && fs::copy(&path, &previous).is_err() {
        let _ = fs::remove_file(&temp);
        super::log_event("configuration pre-restore snapshot failed");
        return false;
    }
    let _ = fs::remove_file(&path);
    if fs::rename(&temp, &path).is_err() {
        if previous.exists() {
            let _ = fs::copy(&previous, &path);
        }
        let _ = fs::remove_file(&temp);
        let _ = fs::remove_file(&previous);
        super::log_event("configuration backup restore commit failed");
        return false;
    }
    let _ = fs::remove_file(&previous);
    super::log_event("configuration backup restored");
    true
}
pub fn default_config() -> Config {
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
        icon_size: 42,
        magnification: 32,
        transparency: 148,
        spacing: 56,
        offset_x: 0,
        offset_y: 0,
        clock_24h: false,
        date_format: 1,
        desktop_enabled: true,
        applet_order: [0, 6, 1, 2, 3, 4, 5],
        panel_zones: [1, 2, 2, 2, 2, 2, 0],
        dock_order: [0, 1, 2, 3, 4],
        accent_color: 0x38bdf8,
        wallpaper_style: 0,
        adaptive_dock_transparency: true,
        panel_transparent: true,
    }
}

pub static mut CONFIG: Config = Config {
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
    icon_size: 42,
    magnification: 32,
    transparency: 148,
    spacing: 56,
    offset_x: 0,
    offset_y: 0,
    clock_24h: false,
    date_format: 1,
    desktop_enabled: true,
    applet_order: [0, 6, 1, 2, 3, 4, 5],
    panel_zones: [1, 2, 2, 2, 2, 2, 0],
    dock_order: [0, 1, 2, 3, 4],
    accent_color: 0x38bdf8,
    wallpaper_style: 0,
    adaptive_dock_transparency: true,
    panel_transparent: true,
};
