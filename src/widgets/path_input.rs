use iced::widget::{button, container, text};
use iced::{Element, Length, Padding};
use iced_longbridge::components::input::input;
use iced_longbridge::theme::AppTheme;

use crate::app::{Message, PanelSide};
use crate::panel::PanelMessage;

/// Renders the path bar as either a read-only label (clickable) or an editable text input.
pub fn path_input_view<'a>(
    theme: &AppTheme,
    current_path: &str,
    editing: bool,
    input_value: &str,
    side: PanelSide,
) -> Element<'a, Message> {
    let t = *theme;
    if editing {
        let field = input(theme, "Enter path...", input_value)
            .on_input(move |s| Message::Panel(side, PanelMessage::PathInputChanged(s)))
            .on_submit(Message::Panel(side, PanelMessage::PathInputSubmit));

        container(field)
            .width(Length::Fill)
            .padding(Padding::from([2, 6]))
            .into()
    } else {
        let path_text = text(current_path.to_string()).size(13).color(t.primary);

        let btn = button(path_text)
            .on_press(Message::Panel(side, PanelMessage::PathBarClicked))
            .padding(Padding::from([4, 8]))
            .width(Length::Fill)
            .style(move |_theme, status| {
                use button::Status::*;
                let bg = match status {
                    Hovered => Some(iced::Background::Color(t.muted)),
                    Pressed => Some(iced::Background::Color(t.accent)),
                    _ => None,
                };
                button::Style {
                    background: bg,
                    text_color: t.primary,
                    ..Default::default()
                }
            });

        btn.into()
    }
}
