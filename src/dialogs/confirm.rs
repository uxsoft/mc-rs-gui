use iced::widget::{Space, column, row, text};
use iced::{Element, Length};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::theme::{AppTheme, Size};

use crate::app::Message;

use super::DialogMessage;

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: Box<Message>,
}

pub fn confirm_view<'a>(theme: &AppTheme, dialog: &'a ConfirmDialog) -> Element<'a, Message> {
    let t = *theme;
    let title = text(&dialog.title).size(16).color(t.foreground);
    let msg = text(&dialog.message).size(13).color(t.muted_foreground);

    let buttons = row![
        Space::new().width(Length::Fill),
        button_ex(
            theme,
            "Yes",
            Variant::Primary,
            Size::Sm,
            Some(Message::DialogResult(DialogMessage::Confirm(true))),
            false,
            false,
        ),
        Space::new().width(8),
        button_ex(
            theme,
            "No",
            Variant::Secondary,
            Size::Sm,
            Some(Message::DialogResult(DialogMessage::Cancel)),
            false,
            false,
        ),
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
