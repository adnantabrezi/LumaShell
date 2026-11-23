//! Prototype: COSMIC-styled iced + wgpu surfaces for Luma.
//!
//! Standalone binary: cargo run --bin luma_iced

use iced::widget::{column, container, row, text};
use iced::{Element, Length, Subscription, Task, Theme};

#[derive(Debug, Clone)]
enum Message {
    Tick,
    SelectWorkspace(usize),
}

struct App {
    progress: f32,
    workspace: usize,
    animating: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            progress: 0.0,
            workspace: 0,
            animating: true,
        }
    }
}

fn title(_app: &App) -> String {
    String::from("Luma COSMIC Prototype (iced + wgpu)")
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            if app.animating {
                app.progress = (app.progress + 0.02).min(1.0);
                if app.progress >= 1.0 {
                    app.animating = false;
                }
            }
        }
        Message::SelectWorkspace(i) => {
            app.workspace = i;
            app.progress = 0.0;
            app.animating = true;
        }
    }
    Task::none()
}

fn subscription(app: &App) -> Subscription<Message> {
    if app.animating {
        iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    }
}

fn view(app: &App) -> Element<'_, Message> {
    let panel = container(
        row![
            text("Luma").size(16),
            text("·").size(16),
            text(format!("Workspace {}", app.workspace + 1)).size(14),
        ]
        .spacing(8),
    )
    .padding(10)
    .width(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.12, 0.12, 0.14, 0.85,
        ))),
        border: iced::Border {
            radius: 12.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let tile = container(text("Tiled app").size(14))
        .padding(16)
        .width(Length::Fixed(260.0))
        .height(Length::Fixed(160.0))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.18,
                0.18,
                0.22,
                0.85 + 0.1 * app.progress,
            ))),
            border: iced::Border {
                radius: 10.0.into(),
                width: 1.0,
                color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            },
            ..Default::default()
        });

    let status = text(format!("Animation progress: {:.0}%", app.progress * 100.0)).size(12);

    let workspaces = row![
        workspace_button(0, app.workspace),
        workspace_button(1, app.workspace),
        workspace_button(2, app.workspace),
    ]
    .spacing(6);

    let content = column![panel, workspaces, tile, status]
        .spacing(12)
        .padding(16);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.06, 0.06, 0.08,
            ))),
            ..Default::default()
        })
        .into()
}

fn workspace_button(index: usize, current: usize) -> Element<'static, Message> {
    let label = format!("{}", index + 1);
    iced::widget::button(text(label).size(12))
        .on_press(Message::SelectWorkspace(index))
        .style(move |_theme, _status| iced::widget::button::Style {
            background: Some(iced::Background::Color(if index == current {
                iced::Color::from_rgb(0.35, 0.55, 0.95)
            } else {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            })),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn main() -> iced::Result {
    iced::application(title, update, view)
        .subscription(subscription)
        .theme(|_| Theme::Dark)
        .run()
}
