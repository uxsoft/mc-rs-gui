use iced::widget::{Column, column, container, scrollable, text};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::Message;

use super::ViewerState;

pub fn text_content_view<'a>(state: &'a ViewerState) -> Element<'a, Message> {
    let text_content = String::from_utf8_lossy(&state.content);
    let lines: Vec<&str> = text_content.lines().collect();

    let visible_lines = &lines[state.offset..lines.len().min(state.offset + state.lines_per_page)];

    let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(visible_lines.len());

    for (i, line) in visible_lines.iter().enumerate() {
        let line_num = state.offset + i + 1;
        let line_text = format!("{line_num:>6}  {line}");

        rows.push(
            text(line_text)
                .size(13)
                .font(Font::MONOSPACE)
                .color(Color::from_rgb(0.8, 0.8, 0.85))
                .into(),
        );
    }

    container(
        scrollable(Column::with_children(rows).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding::from([4, 12]))
    .into()
}
