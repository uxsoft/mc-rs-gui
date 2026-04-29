pub mod chmod;
pub mod confirm;
pub mod input;
pub mod progress;

use iced::Element;
use iced_longbridge::components::overlay;
use iced_longbridge::theme::AppTheme;

use crate::app::Message;

#[derive(Debug, Clone)]
pub enum DialogKind {
    Confirm(confirm::ConfirmDialog),
    Input(input::InputDialog),
    Progress(progress::ProgressDialog),
    Chmod(chmod::ChmodDialog),
}

#[derive(Debug, Clone)]
pub enum DialogMessage {
    Confirm(bool),
    InputChanged(String),
    InputSubmit,
    Cancel,
    ChmodToggleBit(u32),
    ChmodOctalChanged(String),
    ChmodApply,
}

pub fn dialog_overlay<'a>(
    theme: &AppTheme,
    base: Element<'a, Message>,
    dialog: &'a DialogKind,
) -> Element<'a, Message> {
    let (body, max_width): (Element<'a, Message>, f32) = match dialog {
        DialogKind::Confirm(d) => (confirm::confirm_view(theme, d), 480.0),
        DialogKind::Input(d) => (input::input_view(theme, d), 480.0),
        DialogKind::Progress(d) => (progress::progress_view(theme, d), 520.0),
        DialogKind::Chmod(d) => (chmod::chmod_view(theme, d), 520.0),
    };
    let panel = overlay::panel(theme, body, max_width);
    overlay::overlay(
        theme,
        base,
        Some(panel),
        Some(Message::DialogResult(DialogMessage::Cancel)),
    )
}
