pub mod confirm;
pub mod input;
pub mod progress;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Color, Element, Font, Length, Padding};

use crate::app::Message;

#[derive(Debug, Clone)]
pub enum DialogKind {
    Confirm(confirm::ConfirmDialog),
    Input(input::InputDialog),
    Progress(progress::ProgressDialog),
}

#[derive(Debug, Clone)]
pub enum DialogMessage {
    Confirm(bool),
    InputChanged(String),
    InputSubmit,
    Cancel,
}

pub fn dialog_overlay<'a>(dialog: &'a DialogKind) -> Element<'a, Message> {
    let dialog_content: Element<'a, Message> = match dialog {
        DialogKind::Confirm(d) => confirm::confirm_view(d),
        DialogKind::Input(d) => input::input_view(d),
        DialogKind::Progress(d) => progress::progress_view(d),
    };

    // Semi-transparent backdrop with centered dialog
    container(
        container(dialog_content)
            .max_width(500)
            .padding(20)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.4, 0.7),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.0, 0.0, 0.0, 0.6,
        ))),
        ..Default::default()
    })
    .into()
}

fn dialog_button<'a>(label: &str, msg: Message, primary: bool) -> Element<'a, Message> {
    let bg = if primary {
        Color::from_rgb(0.2, 0.35, 0.7)
    } else {
        Color::from_rgb(0.2, 0.2, 0.25)
    };

    button(
        text(label.to_string())
            .size(14)
            .font(Font::MONOSPACE)
            .color(Color::from_rgb(0.9, 0.9, 0.95)),
    )
    .padding(Padding::from([6, 16]))
    .style(move |_theme, _status| button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::WHITE,
        border: iced::Border {
            color: if primary {
                Color::from_rgb(0.3, 0.45, 0.8)
            } else {
                Color::from_rgb(0.3, 0.3, 0.35)
            },
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .on_press(msg)
    .into()
}
