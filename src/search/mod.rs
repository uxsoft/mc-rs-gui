pub mod finder;

use iced::widget::{Column, Space, button, column, container, row, scrollable, text};
use iced::{Color, Element, Length};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::components::input::input;
use iced_longbridge::theme::{AppTheme, Size};

use crate::app::Message;
use crate::vfs::VfsPath;

#[derive(Debug, Clone)]
pub enum SearchMessage {
    DirectoryChanged(String),
    PatternChanged(String),
    ContentChanged(String),
    Start,
    ResultFound(VfsPath),
    Complete,
    GoToResult(VfsPath),
    Close,
}

pub struct SearchState {
    pub directory: String,
    pub pattern: String,
    pub content_pattern: String,
    pub results: Vec<VfsPath>,
    pub searching: bool,
}

impl SearchState {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
            pattern: "*".into(),
            content_pattern: String::new(),
            results: Vec::new(),
            searching: false,
        }
    }
}

pub fn search_view<'a>(theme: &AppTheme, state: &'a SearchState) -> Element<'a, Message> {
    let t = *theme;
    let title = text("File Search").size(16).color(t.foreground);

    let dir_input = row![
        text("Directory:")
            .size(13)
            .color(t.muted_foreground)
            .width(Length::Fixed(100.0)),
        input(theme, "", &state.directory)
            .on_input(|s| Message::Search(SearchMessage::DirectoryChanged(s))),
    ]
    .spacing(8);

    let pattern_input = row![
        text("File name:")
            .size(13)
            .color(t.muted_foreground)
            .width(Length::Fixed(100.0)),
        input(theme, "*", &state.pattern)
            .on_input(|s| Message::Search(SearchMessage::PatternChanged(s))),
    ]
    .spacing(8);

    let content_input = row![
        text("Content:")
            .size(13)
            .color(t.muted_foreground)
            .width(Length::Fixed(100.0)),
        input(theme, "", &state.content_pattern)
            .on_input(|s| Message::Search(SearchMessage::ContentChanged(s))),
    ]
    .spacing(8);

    let search_button = button_ex(
        theme,
        if state.searching {
            "Searching..."
        } else {
            "Search"
        },
        Variant::Primary,
        Size::Sm,
        if state.searching {
            None
        } else {
            Some(Message::Search(SearchMessage::Start))
        },
        state.searching,
        false,
    );

    // Results list
    let results: Vec<Element<'a, Message>> = state
        .results
        .iter()
        .map(|path| {
            button(text(path.to_string()).size(12).color(t.primary))
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
                })
                .on_press(Message::Search(SearchMessage::GoToResult(path.clone())))
                .into()
        })
        .collect();

    let results_list =
        scrollable(Column::with_children(results).width(Length::Fill)).height(Length::Fixed(300.0));

    let status = text(format!("{} results found", state.results.len()))
        .size(12)
        .color(t.muted_foreground);

    let close_button = button_ex(
        theme,
        "Close",
        Variant::Secondary,
        Size::Sm,
        Some(Message::Search(SearchMessage::Close)),
        false,
        false,
    );

    let buttons = row![search_button, Space::new().width(8), close_button];

    let dialog_content = column![
        title,
        Space::new().height(12),
        dir_input,
        Space::new().height(8),
        pattern_input,
        Space::new().height(8),
        content_input,
        Space::new().height(12),
        buttons,
        Space::new().height(12),
        results_list,
        Space::new().height(4),
        status,
    ];

    let popover_bg = t.popover;
    let border_color = t.border;
    container(
        container(dialog_content)
            .max_width(600)
            .padding(20)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(popover_bg)),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.0, 0.0, 0.0, 0.6,
        ))),
        ..Default::default()
    })
    .into()
}
