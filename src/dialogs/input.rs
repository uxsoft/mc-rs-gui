use iced::widget::{Space, column, row, text, text_input};
use iced::{Color, Element, Font, Length};

use crate::app::Message;

use super::{DialogMessage, dialog_button};

#[derive(Debug, Clone)]
pub struct InputDialog {
    pub title: String,
    pub label: String,
    pub value: String,
    pub on_submit: fn(String) -> Message,
}

pub fn input_view<'a>(dialog: &'a InputDialog) -> Element<'a, Message> {
    let title = text(&dialog.title)
        .size(16)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let label = text(&dialog.label)
        .size(13)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.7, 0.7, 0.75));

    let input = text_input("", &dialog.value)
        .on_input(|s| Message::DialogResult(DialogMessage::InputChanged(s)))
        .on_submit(Message::DialogResult(DialogMessage::InputSubmit))
        .size(14)
        .font(Font::with_name("Caskaydia Mono Nerd Font"));

    let buttons = row![
        dialog_button(
            "OK",
            Message::DialogResult(DialogMessage::InputSubmit),
            true
        ),
        Space::with_width(8),
        dialog_button(
            "Cancel",
            Message::DialogResult(DialogMessage::Cancel),
            false
        ),
    ];

    column![
        title,
        Space::with_height(8),
        label,
        Space::with_height(4),
        input,
        Space::with_height(16),
        buttons,
    ]
    .into()
}
