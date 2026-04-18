use iced::widget::{Space, column, row, text};
use iced::{Color, Element, Font, Length};

use crate::app::Message;

use super::{DialogMessage, dialog_button};

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: Box<Message>,
}

pub fn confirm_view<'a>(dialog: &'a ConfirmDialog) -> Element<'a, Message> {
    let title = text(&dialog.title)
        .size(16)
        .font(Font::MONOSPACE)
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let msg = text(&dialog.message)
        .size(13)
        .font(Font::MONOSPACE)
        .color(Color::from_rgb(0.7, 0.7, 0.75));

    let buttons = row![
        dialog_button(
            "Yes",
            Message::DialogResult(DialogMessage::Confirm(true)),
            true
        ),
        Space::with_width(8),
        dialog_button("No", Message::DialogResult(DialogMessage::Cancel), false),
    ];

    column![
        title,
        Space::with_height(8),
        msg,
        Space::with_height(16),
        buttons
    ]
    .into()
}
