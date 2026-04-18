use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::Message;

#[derive(Debug, Clone)]
pub enum EditorMessage {
    ActionPerformed(text_editor::Action),
    Save,
    Close,
    Find,
    FindChanged(String),
    FindNext,
}

pub struct EditorState {
    pub file_name: String,
    pub file_path: crate::vfs::VfsPath,
    pub content: text_editor::Content,
    pub dirty: bool,
    pub find_query: String,
    pub find_active: bool,
    pub status_message: Option<String>,
}

impl EditorState {
    pub fn new(file_name: String, file_path: crate::vfs::VfsPath, text: String) -> Self {
        Self {
            file_name,
            file_path,
            content: text_editor::Content::with_text(&text),
            dirty: false,
            find_query: String::new(),
            find_active: false,
            status_message: None,
        }
    }
}

pub fn editor_view<'a>(state: &'a EditorState) -> Element<'a, Message> {
    // Header
    let dirty_marker = if state.dirty { " [modified]" } else { "" };
    let header = row![
        text(format!("{}{dirty_marker}", state.file_name))
            .size(14)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.8, 0.85, 0.95)),
        Space::new().width(Length::Fill),
        if let Some(ref msg) = state.status_message {
            text(msg.clone())
                .size(12)
                .font(Font::with_name("Caskaydia Mono Nerd Font"))
                .color(Color::from_rgb(0.5, 0.8, 0.5))
        } else {
            text(String::new()).size(12)
        },
    ]
    .padding(Padding::from([4, 12]));

    // Editor area
    let editor = text_editor(&state.content)
        .on_action(|action| Message::Editor(EditorMessage::ActionPerformed(action)))
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .size(13);

    // Footer
    let footer = row![
        editor_button("F2 Save", Message::Editor(EditorMessage::Save)),
        Space::new().width(4),
        editor_button("F7 Find", Message::Editor(EditorMessage::Find)),
        Space::new().width(4),
        editor_button("F10 Quit", Message::Editor(EditorMessage::Close)),
    ]
    .padding(Padding::from([4, 8]));

    let content = column![header, editor, footer];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
            ..Default::default()
        })
        .into()
}

fn editor_button<'a>(label: &str, msg: Message) -> Element<'a, Message> {
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
