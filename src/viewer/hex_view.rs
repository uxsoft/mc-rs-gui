use iced::widget::{Column, column, container, scrollable, text};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::Message;

use super::ViewerState;

pub fn hex_content_view<'a>(state: &'a ViewerState) -> Element<'a, Message> {
    let bytes_per_line = 16;
    let start = state.offset * bytes_per_line;
    let end = (start + state.lines_per_page * bytes_per_line).min(state.content.len());

    let mut rows: Vec<Element<'a, Message>> = Vec::new();

    let mut pos = start;
    while pos < end {
        let line_end = (pos + bytes_per_line).min(state.content.len());
        let chunk = &state.content[pos..line_end];

        // Address
        let mut line = format!("{pos:08X}  ");

        // Hex bytes
        for (i, byte) in chunk.iter().enumerate() {
            line.push_str(&format!("{byte:02X} "));
            if i == 7 {
                line.push(' ');
            }
        }

        // Pad if short line
        for i in chunk.len()..bytes_per_line {
            line.push_str("   ");
            if i == 7 {
                line.push(' ');
            }
        }

        line.push_str(" |");

        // ASCII representation
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                line.push(*byte as char);
            } else {
                line.push('.');
            }
        }
        line.push('|');

        rows.push(
            text(line)
                .size(13)
                .font(Font::MONOSPACE)
                .color(Color::from_rgb(0.75, 0.8, 0.85))
                .into(),
        );

        pos = line_end;
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
