use iced::widget::{Space, column, row, text};
use iced::{Element, Length};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::components::input::input;
use iced_longbridge::theme::{AppTheme, Size};

use crate::app::Message;

use super::DialogMessage;

pub const INPUT_DIALOG_ID: &str = "dialog-input";

#[derive(Debug, Clone)]
pub struct InputDialog {
    pub title: String,
    pub label: String,
    pub value: String,
    pub on_submit: fn(String) -> Message,
}

pub fn input_view<'a>(theme: &AppTheme, dialog: &'a InputDialog) -> Element<'a, Message> {
    let t = *theme;
    let title = text(&dialog.title).size(16).color(t.foreground);
    let label = text(&dialog.label).size(13).color(t.muted_foreground);

    let field = input(theme, "", &dialog.value)
        .id(INPUT_DIALOG_ID)
        .on_input(|s| Message::DialogResult(DialogMessage::InputChanged(s)))
        .on_submit(Message::DialogResult(DialogMessage::InputSubmit));

    let buttons = row![
        Space::new().width(Length::Fill),
        button_ex(
            theme,
            "OK",
            Variant::Primary,
            Size::Sm,
            Some(Message::DialogResult(DialogMessage::InputSubmit)),
            false,
            false,
        ),
        Space::new().width(8),
        button_ex(
            theme,
            "Cancel",
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
        label,
        Space::new().height(4),
        field,
        Space::new().height(16),
        buttons,
    ]
    .into()
}
