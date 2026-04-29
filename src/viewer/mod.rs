pub mod hex_view;
pub mod text_view;

use iced::widget::{Space, column, container, row, text};
use iced::{Element, Length, Padding};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::theme::{AppTheme, Size};

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
            ViewMode::Hex => self.content.len().div_ceil(16),
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

pub fn viewer_view<'a>(theme: &AppTheme, state: &'a ViewerState) -> Element<'a, Message> {
    let t = *theme;
    // Header bar
    let mode_label = match state.mode {
        ViewMode::Text => "Text",
        ViewMode::Hex => "Hex",
    };

    let header = row![
        text(&state.file_name).size(14).color(t.foreground),
        Space::new().width(Length::Fill),
        text(format!("Mode: {mode_label}"))
            .size(12)
            .color(t.muted_foreground),
        Space::new().width(8),
        text(format!("Line {}", state.offset + 1))
            .size(12)
            .color(t.muted_foreground),
    ]
    .padding(Padding::from([4, 12]));

    // Content area
    let content_element: Element<'a, Message> = match state.mode {
        ViewMode::Text => text_view::text_content_view(theme, state),
        ViewMode::Hex => hex_view::hex_content_view(theme, state),
    };

    // Footer
    let footer = row![
        button_ex(
            theme,
            "F3 Quit",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Viewer(ViewerMessage::Close)),
            false,
            false
        ),
        Space::new().width(4),
        button_ex(
            theme,
            "F4 Hex",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Viewer(ViewerMessage::SwitchMode(
                if state.mode == ViewMode::Text {
                    ViewMode::Hex
                } else {
                    ViewMode::Text
                }
            ))),
            false,
            false
        ),
        Space::new().width(4),
        button_ex(
            theme,
            "F7 Search",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Viewer(ViewerMessage::SearchOpen)),
            false,
            false
        ),
    ]
    .padding(Padding::from([4, 8]));

    let viewer = column![header, content_element, footer];

    container(viewer)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(t.background)),
            ..Default::default()
        })
        .into()
}
