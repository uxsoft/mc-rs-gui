use iced::widget::{button, container, text, text_input};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::{Message, PanelSide};
use crate::panel::PanelMessage;

/// Renders the path bar as either a read-only label (clickable) or an editable text input.
pub fn path_input_view<'a>(
    current_path: &str,
    editing: bool,
    input_value: &str,
    side: PanelSide,
) -> Element<'a, Message> {
    if editing {
        let input = text_input("Enter path...", input_value)
            .on_input(move |s| Message::Panel(side, PanelMessage::PathInputChanged(s)))
            .on_submit(Message::Panel(side, PanelMessage::PathInputSubmit))
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"));

        container(input)
            .width(Length::Fill)
            .padding(Padding::from([2, 6]))
            .into()
    } else {
        let path_text = text(current_path.to_string())
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.7, 0.8, 0.95));

        let btn = button(path_text)
            .on_press(Message::Panel(side, PanelMessage::PathBarClicked))
            .padding(Padding::from([4, 8]))
            .width(Length::Fill)
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: Color::WHITE,
                ..Default::default()
            });

        btn.into()
    }
}
