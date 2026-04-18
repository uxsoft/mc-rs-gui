pub mod hex_view;
pub mod text_view;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Text,
    Hex,
}

#[derive(Debug, Clone)]
pub enum ViewerMessage {
    SwitchMode(ViewMode),
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoTop,
    GoBottom,
    SearchOpen,
    SearchChanged(String),
    SearchNext,
    Close,
}

pub struct ViewerState {
    pub file_name: String,
    pub content: Vec<u8>,
    pub mode: ViewMode,
    pub offset: usize,
    pub search_query: String,
    pub search_active: bool,
    pub lines_per_page: usize,
}

impl ViewerState {
    pub fn new(file_name: String, content: Vec<u8>) -> Self {
        Self {
            file_name,
            content,
            mode: ViewMode::Text,
            offset: 0,
            search_query: String::new(),
            search_active: false,
            lines_per_page: 40,
        }
    }

    pub fn total_lines(&self) -> usize {
        match self.mode {
            ViewMode::Text => self.content.iter().filter(|&&b| b == b'\n').count().max(1),
            ViewMode::Hex => (self.content.len() + 15) / 16,
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.offset = self.offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max = self.total_lines().saturating_sub(self.lines_per_page);
        self.offset = (self.offset + lines).min(max);
    }
}

pub fn viewer_view<'a>(state: &'a ViewerState) -> Element<'a, Message> {
    // Header bar
    let mode_label = match state.mode {
        ViewMode::Text => "Text",
        ViewMode::Hex => "Hex",
    };

    let header = row![
        text(&state.file_name)
            .size(14)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.8, 0.85, 0.95)),
        Space::with_width(Length::Fill),
        text(format!("Mode: {mode_label}"))
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65)),
        Space::with_width(8),
        text(format!("Line {}", state.offset + 1))
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65)),
    ]
    .padding(Padding::from([4, 12]));

    // Content area
    let content_element: Element<'a, Message> = match state.mode {
        ViewMode::Text => text_view::text_content_view(state),
        ViewMode::Hex => hex_view::hex_content_view(state),
    };

    // Footer
    let footer = row![
        viewer_button("F3 Quit", Message::Viewer(ViewerMessage::Close)),
        Space::with_width(4),
        viewer_button(
            "F4 Hex",
            Message::Viewer(ViewerMessage::SwitchMode(if state.mode == ViewMode::Text {
                ViewMode::Hex
            } else {
                ViewMode::Text
            }))
        ),
        Space::with_width(4),
        viewer_button("F7 Search", Message::Viewer(ViewerMessage::SearchOpen)),
    ]
    .padding(Padding::from([4, 8]));

    let viewer = column![header, content_element, footer];

    container(viewer)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
            ..Default::default()
        })
        .into()
}

fn viewer_button<'a>(label: &str, msg: Message) -> Element<'a, Message> {
    button(
        text(label.to_string())
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.85, 0.85, 0.9)),
    )
    .padding(Padding::from([2, 8]))
    .style(|_theme, _status| button::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
        text_color: Color::WHITE,
        border: iced::Border {
            color: Color::from_rgb(0.25, 0.25, 0.3),
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    })
    .on_press(msg)
    .into()
}
