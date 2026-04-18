pub mod finder;

use iced::widget::{Column, Space, button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Font, Length, Padding};

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

pub fn search_view<'a>(state: &'a SearchState) -> Element<'a, Message> {
    let title = text("File Search")
        .size(16)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let dir_input = row![
        text("Directory:")
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.7, 0.7, 0.75))
            .width(Length::Fixed(100.0)),
        text_input("", &state.directory)
            .on_input(|s| Message::Search(SearchMessage::DirectoryChanged(s)))
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font")),
    ]
    .spacing(8);

    let pattern_input = row![
        text("File name:")
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.7, 0.7, 0.75))
            .width(Length::Fixed(100.0)),
        text_input("*", &state.pattern)
            .on_input(|s| Message::Search(SearchMessage::PatternChanged(s)))
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font")),
    ]
    .spacing(8);

    let content_input = row![
        text("Content:")
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.7, 0.7, 0.75))
            .width(Length::Fixed(100.0)),
        text_input("", &state.content_pattern)
            .on_input(|s| Message::Search(SearchMessage::ContentChanged(s)))
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font")),
    ]
    .spacing(8);

    let search_button = button(
        text(if state.searching {
            "Searching..."
        } else {
            "Search"
        })
        .size(14)
        .font(Font::with_name("Caskaydia Mono Nerd Font")),
    )
    .padding(Padding::from([6, 16]))
    .on_press_maybe(if state.searching {
        None
    } else {
        Some(Message::Search(SearchMessage::Start))
    });

    // Results list
    let results: Vec<Element<'a, Message>> = state
        .results
        .iter()
        .map(|path| {
            button(
                text(path.to_string())
                    .size(12)
                    .font(Font::with_name("Caskaydia Mono Nerd Font"))
                    .color(Color::from_rgb(0.7, 0.8, 0.95)),
            )
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: Color::WHITE,
                ..Default::default()
            })
            .on_press(Message::Search(SearchMessage::GoToResult(path.clone())))
            .into()
        })
        .collect();

    let results_list =
        scrollable(Column::with_children(results).width(Length::Fill)).height(Length::Fixed(300.0));

    let status = text(format!("{} results found", state.results.len()))
        .size(12)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.5, 0.5, 0.55));

    let close_button = button(
        text("Close")
            .size(13)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.85, 0.85, 0.9)),
    )
    .padding(Padding::from([4, 12]))
    .on_press(Message::Search(SearchMessage::Close));

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

    container(
        container(dialog_content)
            .max_width(600)
            .padding(20)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
                border: iced::Border {
                    color: Color::from_rgb(0.3, 0.4, 0.7),
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
