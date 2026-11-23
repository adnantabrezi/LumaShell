//! Luma COSMIC Desktop Shell & Settings Studio (`iced` + `wgpu` GPU Renderer).
//!
//! Run with: `cargo run --bin luma_iced`
//! Persists directly to `%APPDATA%\LumaShell\settings.ini`.

use iced::widget::{button, checkbox, column, container, row, scrollable, slider, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme};
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    General,
    Dock,
    Applets,
    GpuCompositor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelZone {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    SelectWorkspace(usize),
    SelectTab(SettingsTab),
    ToggleDark(bool),
    ToggleDesktop(bool),
    ToggleClock24h(bool),
    ToggleAutoHide(bool),
    ToggleAdaptiveTransparency(bool),
    SetDockAlign(DockAlign),
    SetIconSize(f32),
    SetMagnification(f32),
    SetTransparency(f32),
    SetSpacing(f32),
    HoverDockIcon(Option<usize>),
    MoveDockItemUp(usize),
    MoveDockItemDown(usize),
    ToggleApplet(usize, bool),
    SetAppletZone(usize, PanelZone),
    MoveAppletUp(usize),
    MoveAppletDown(usize),
    SaveSettingsIni,
}

#[derive(Debug, Clone)]
struct AppletEntry {
    id: &'static str,
    title: &'static str,
    enabled: bool,
    zone: PanelZone,
}

struct LumaCosmicApp {
    workspace: usize,
    active_tab: SettingsTab,
    dark: bool,
    desktop_enabled: bool,
    clock_24h: bool,
    auto_hide: bool,
    adaptive_dock_transparency: bool,
    dock_align: DockAlign,
    icon_size: f32,
    magnification: f32,
    transparency: f32,
    spacing: f32,
    hovered_dock: Option<usize>,
    dock_order: Vec<&'static str>,
    applets: Vec<AppletEntry>,
    shader_phase: f32,
    status_banner: String,
}

impl Default for LumaCosmicApp {
    fn default() -> Self {
        Self {
            workspace: 3,
            active_tab: SettingsTab::Dock,
            dark: true,
            desktop_enabled: true,
            clock_24h: false,
            auto_hide: false,
            adaptive_dock_transparency: true,
            dock_align: DockAlign::Center,
            icon_size: 42.0,
            magnification: 32.0,
            transparency: 148.0,
            spacing: 56.0,
            hovered_dock: None,
            dock_order: vec!["home", "files", "terminal", "browser", "settings"],
            applets: vec![
                AppletEntry {
                    id: "clock",
                    title: "Clock & Calendar",
                    enabled: true,
                    zone: PanelZone::Center,
                },
                AppletEntry {
                    id: "media",
                    title: "Media Playback",
                    enabled: true,
                    zone: PanelZone::Left,
                },
                AppletEntry {
                    id: "network",
                    title: "Network & Wi-Fi",
                    enabled: true,
                    zone: PanelZone::Right,
                },
                AppletEntry {
                    id: "volume",
                    title: "Audio & Session Mixer",
                    enabled: true,
                    zone: PanelZone::Right,
                },
                AppletEntry {
                    id: "bluetooth",
                    title: "Bluetooth Devices",
                    enabled: true,
                    zone: PanelZone::Right,
                },
                AppletEntry {
                    id: "battery",
                    title: "Power & Battery",
                    enabled: true,
                    zone: PanelZone::Right,
                },
                AppletEntry {
                    id: "notifications",
                    title: "Notifications & Do Not Disturb",
                    enabled: true,
                    zone: PanelZone::Right,
                },
            ],
            shader_phase: 0.0,
            status_banner: String::from("WGPU Hardware Surface Active · Synced with settings.ini"),
        }
    }
}

fn config_path() -> PathBuf {
    env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("LumaShell")
        .join("settings.ini")
}

fn save_to_ini(app: &LumaCosmicApp) -> Result<PathBuf, std::io::Error> {
    let pos = match app.dock_align {
        DockAlign::Left => "left",
        DockAlign::Center => "center",
        DockAlign::Right => "right",
    };
    let order = app
        .applets
        .iter()
        .map(|a| a.id)
        .collect::<Vec<_>>()
        .join(",");
    let zones = app
        .applets
        .iter()
        .map(|a| {
            let z = match a.zone {
                PanelZone::Left => "left",
                PanelZone::Center => "center",
                PanelZone::Right => "right",
            };
            format!("{}:{}", a.id, z)
        })
        .collect::<Vec<_>>()
        .join(",");
    let dock_order = app.dock_order.join(",");

    let find_enabled = |id: &str| {
        app.applets
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.enabled)
            .unwrap_or(true)
    };

    let body = format!(
        "version=9\ndark={}\ndesktop={}\nclock={}\nnetwork={}\nvolume={}\nbattery={}\nnotifications={}\nbluetooth={}\nmedia={}\nposition={}\nauto_hide={}\nicon_size={}\nmagnification={}\ntransparency={}\nspacing={}\noffset_x=0\noffset_y=0\nclock_24h={}\ndate_format=short\norder={}\nzones={}\ndock_order={}\nadaptive_dock_transparency={}\npanel_style=transparent\n",
        app.dark,
        app.desktop_enabled,
        find_enabled("clock"),
        find_enabled("network"),
        find_enabled("volume"),
        find_enabled("battery"),
        find_enabled("notifications"),
        find_enabled("bluetooth"),
        find_enabled("media"),
        pos,
        app.auto_hide,
        app.icon_size as i32,
        app.magnification as u8,
        app.transparency as u8,
        app.spacing as i32,
        app.clock_24h,
        order,
        zones,
        dock_order,
        app.adaptive_dock_transparency
    );
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, body)?;
    Ok(path)
}

fn title(_app: &LumaCosmicApp) -> String {
    String::from("Luma Shell — COSMIC Desktop & WGPU Settings")
}

fn update(app: &mut LumaCosmicApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            app.shader_phase = (app.shader_phase + 0.015) % std::f32::consts::TAU;
        }
        Message::SelectWorkspace(i) => {
            app.workspace = i;
        }
        Message::SelectTab(tab) => {
            app.active_tab = tab;
        }
        Message::ToggleDark(v) => app.dark = v,
        Message::ToggleDesktop(v) => app.desktop_enabled = v,
        Message::ToggleClock24h(v) => app.clock_24h = v,
        Message::ToggleAutoHide(v) => app.auto_hide = v,
        Message::ToggleAdaptiveTransparency(v) => app.adaptive_dock_transparency = v,
        Message::SetDockAlign(align) => app.dock_align = align,
        Message::SetIconSize(v) => app.icon_size = v,
        Message::SetMagnification(v) => app.magnification = v,
        Message::SetTransparency(v) => app.transparency = v,
        Message::SetSpacing(v) => app.spacing = v,
        Message::HoverDockIcon(idx) => app.hovered_dock = idx,
        Message::MoveDockItemUp(idx) => {
            if idx > 0 && idx < app.dock_order.len() {
                app.dock_order.swap(idx, idx - 1);
            }
        }
        Message::MoveDockItemDown(idx) => {
            if idx + 1 < app.dock_order.len() {
                app.dock_order.swap(idx, idx + 1);
            }
        }
        Message::ToggleApplet(idx, enabled) => {
            if let Some(applet) = app.applets.get_mut(idx) {
                applet.enabled = enabled;
            }
        }
        Message::SetAppletZone(idx, zone) => {
            if let Some(applet) = app.applets.get_mut(idx) {
                applet.zone = zone;
            }
        }
        Message::MoveAppletUp(idx) => {
            if idx > 0 && idx < app.applets.len() {
                app.applets.swap(idx, idx - 1);
            }
        }
        Message::MoveAppletDown(idx) => {
            if idx + 1 < app.applets.len() {
                app.applets.swap(idx, idx + 1);
            }
        }
        Message::SaveSettingsIni => match save_to_ini(app) {
            Ok(path) => {
                app.status_banner = format!("Saved to {}", path.display());
            }
            Err(err) => {
                app.status_banner = format!("Failed to write settings.ini: {}", err);
            }
        },
    }
    Task::none()
}

fn subscription(_app: &LumaCosmicApp) -> Subscription<Message> {
    iced::time::every(std::time::Duration::from_millis(32)).map(|_| Message::Tick)
}

fn view(app: &LumaCosmicApp) -> Element<'_, Message> {
    let workspace_buttons = (0..4usize)
        .map(|i| {
            let is_active = app.workspace == i;
            button(text(format!("{}", i + 1)).size(12))
                .on_press(Message::SelectWorkspace(i))
                .padding([4, 8])
                .style(move |_theme, _status| iced::widget::button::Style {
                    background: Some(Background::Color(if is_active {
                        Color::from_rgba(1.0, 1.0, 1.0, 0.28)
                    } else {
                        Color::TRANSPARENT
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: 999.0.into(),
                        width: if is_active { 1.0 } else { 0.0 },
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.40),
                    },
                    ..Default::default()
                })
                .into()
        })
        .collect::<Vec<Element<'_, Message>>>();

    let clock_label = if app.clock_24h {
        "23 Nov, 11:13"
    } else {
        "23 Nov, 11:13 am"
    };

    // Equal FillPortion(1) left & right wings lock the center clock to the exact horizontal midpoint
    let top_panel = container(
        row![
            container(row(workspace_buttons).spacing(4).align_y(Alignment::Center))
                .width(Length::FillPortion(1))
                .align_x(iced::alignment::Horizontal::Left),
            container(text(clock_label).size(13))
                .align_x(iced::alignment::Horizontal::Center),
            container(
                row![
                    text("US").size(12),
                    text("^ Tray").size(12),
                    button(text("Save INI").size(12)).on_press(Message::SaveSettingsIni),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
            )
            .width(Length::FillPortion(1))
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
    )
    .padding([6, 14])
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    });

    let sidebar = column![
        tab_button("Dock Settings", SettingsTab::Dock, app.active_tab),
        tab_button("Panel Applets", SettingsTab::Applets, app.active_tab),
        tab_button("General & Theme", SettingsTab::General, app.active_tab),
        tab_button("WGPU Compositor", SettingsTab::GpuCompositor, app.active_tab),
    ]
    .spacing(8)
    .width(Length::Fixed(210.0));

    let page_body: Element<'_, Message> = match app.active_tab {
        SettingsTab::Dock => view_dock_settings(app),
        SettingsTab::Applets => view_applet_settings(app),
        SettingsTab::General => view_general_settings(app),
        SettingsTab::GpuCompositor => view_gpu_settings(app),
    };

    let main_card = container(
        row![
            sidebar,
            scrollable(container(page_body).padding(16).width(Length::Fill)).height(Length::Fill)
        ]
        .spacing(20),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.08, 0.12, 0.92))),
        border: Border {
            radius: 16.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
        },
        ..Default::default()
    });

    let dock_buttons = app
        .dock_order
        .iter()
        .enumerate()
        .map(|(i, item)| {
            button(text(*item).size(13))
                .on_press(Message::HoverDockIcon(Some(i)))
                .padding([8, 14])
                .into()
        })
        .collect::<Vec<Element<'_, Message>>>();

    let dock_alpha = (app.transparency / 255.0).clamp(0.22, 0.68);
    let dock_bar = container(row(dock_buttons).spacing(12))
        .padding([10, 20])
        .style(move |_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(
                0.055,
                0.063,
                0.086,
                dock_alpha,
            ))),
            border: Border {
                radius: 999.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.14),
            },
            ..Default::default()
        });

    let content = column![
        top_panel,
        main_card,
        row![
            text(&app.status_banner).size(12),
            container(dock_bar).width(Length::Fill).center_x(Length::Fill)
        ]
        .align_y(Alignment::Center)
    ]
    .spacing(14)
    .padding(16);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.01, 0.015, 0.025))),
            ..Default::default()
        })
        .into()
}

fn tab_button(
    label: &'static str,
    tab: SettingsTab,
    current: SettingsTab,
) -> Element<'static, Message> {
    let active = tab == current;
    button(text(label).size(14))
        .width(Length::Fill)
        .padding([10, 14])
        .on_press(Message::SelectTab(tab))
        .style(move |_theme, _status| iced::widget::button::Style {
            background: Some(Background::Color(if active {
                Color::from_rgb(0.22, 0.74, 0.97)
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.06)
            })),
            text_color: if active {
                Color::from_rgb(0.05, 0.07, 0.10)
            } else {
                Color::from_rgb(0.90, 0.92, 0.96)
            },
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn view_dock_settings(app: &LumaCosmicApp) -> Element<'_, Message> {
    let mut order_rows = column![].spacing(6);
    for (idx, item) in app.dock_order.iter().enumerate() {
        order_rows = order_rows.push(
            row![
                text(format!("{}. {}", idx + 1, item))
                    .size(14)
                    .width(Length::Fixed(160.0)),
                button(text("Up").size(12)).on_press(Message::MoveDockItemUp(idx)),
                button(text("Down").size(12)).on_press(Message::MoveDockItemDown(idx)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
    }

    column![
        text("COSMIC Dock Configuration").size(22),
        text("Customize dock geometry, magnification, transparency, and launcher sequence.").size(13),
        row![
            text("Alignment:").size(14),
            button(text("Left").size(13)).on_press(Message::SetDockAlign(DockAlign::Left)),
            button(text("Center").size(13)).on_press(Message::SetDockAlign(DockAlign::Center)),
            button(text("Right").size(13)).on_press(Message::SetDockAlign(DockAlign::Right)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        checkbox(
            "Adaptive dock transparency (transparent on empty desktop)",
            app.adaptive_dock_transparency,
        )
        .on_toggle(Message::ToggleAdaptiveTransparency),
        checkbox("Auto-hide dock when windows overlap", app.auto_hide)
            .on_toggle(Message::ToggleAutoHide),
        text(format!("Icon Size: {:.0} px", app.icon_size)).size(14),
        slider(24.0..=64.0, app.icon_size, Message::SetIconSize),
        text(format!("Hover Magnification: {:.0}%", app.magnification)).size(14),
        slider(0.0..=100.0, app.magnification, Message::SetMagnification),
        text(format!("Surface Alpha: {:.0}/255", app.transparency)).size(14),
        slider(40.0..=230.0, app.transparency, Message::SetTransparency),
        text(format!("Launcher Spacing: {:.0} px", app.spacing)).size(14),
        slider(36.0..=96.0, app.spacing, Message::SetSpacing),
        text("Dock Launcher Order").size(16),
        order_rows,
    ]
    .spacing(12)
    .into()
}

fn view_applet_settings(app: &LumaCosmicApp) -> Element<'_, Message> {
    let mut list = column![].spacing(10);
    for (idx, applet) in app.applets.iter().enumerate() {
        list = list.push(
            row![
                checkbox(applet.title, applet.enabled)
                    .on_toggle(move |v| Message::ToggleApplet(idx, v))
                    .width(Length::Fixed(240.0)),
                button(text("Left").size(12))
                    .on_press(Message::SetAppletZone(idx, PanelZone::Left)),
                button(text("Center").size(12))
                    .on_press(Message::SetAppletZone(idx, PanelZone::Center)),
                button(text("Right").size(12))
                    .on_press(Message::SetAppletZone(idx, PanelZone::Right)),
                button(text("Up").size(12)).on_press(Message::MoveAppletUp(idx)),
                button(text("Down").size(12)).on_press(Message::MoveAppletDown(idx)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );
    }

    column![
        text("COSMIC Panel Applets & Zones").size(22),
        text("Assign applets to Left (Start), Center, or Right (End) zones and reorder them.")
            .size(13),
        list,
    ]
    .spacing(12)
    .into()
}

fn view_general_settings(app: &LumaCosmicApp) -> Element<'_, Message> {
    column![
        text("Appearance & Desktop").size(22),
        checkbox("COSMIC Dark Theme", app.dark).on_toggle(Message::ToggleDark),
        checkbox(
            "Enable Luma Desktop Wallpaper & Context Menu",
            app.desktop_enabled
        )
        .on_toggle(Message::ToggleDesktop),
        checkbox("Use 24-Hour Clock", app.clock_24h).on_toggle(Message::ToggleClock24h),
    ]
    .spacing(12)
    .into()
}

fn view_gpu_settings(app: &LumaCosmicApp) -> Element<'_, Message> {
    column![
        text("WGPU & SDF Surface Compositor").size(22),
        text("Hardware-accelerated WGPU pipeline with sub-pixel SDF rounded corners and cosine magnification.").size(13),
        text(format!("Shader Phase Uniform: {:.2} rad", app.shader_phase)).size(14),
        row![
            button(text("Workspace 1").size(13)).on_press(Message::SelectWorkspace(0)),
            button(text("Workspace 2").size(13)).on_press(Message::SelectWorkspace(1)),
            button(text("Workspace 3").size(13)).on_press(Message::SelectWorkspace(2)),
        ]
        .spacing(10),
    ]
    .spacing(12)
    .into()
}

fn main() -> iced::Result {
    iced::application(title, update, view)
        .subscription(subscription)
        .theme(|app| if app.dark { Theme::Dark } else { Theme::Light })
        .run()
}
