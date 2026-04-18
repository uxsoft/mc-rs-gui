use iced::widget::{Space, column, row, text};
use iced::{Color, Element, Font};

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
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let msg = text(&dialog.message)
        .size(13)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.7, 0.7, 0.75));

    let buttons = row![
        dialog_button(
            "Yes",
            Message::DialogResult(DialogMessage::Confirm(true)),
            true
        ),
        Space::new().width(8),
        dialog_button("No", Message::DialogResult(DialogMessage::Cancel), false),
    ];

    column![
        title,
        Space::new().height(8),
        msg,
        Space::new().height(16),
        buttons
    ]
    .into()
}
