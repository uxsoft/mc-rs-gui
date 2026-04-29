use iced::widget::{Space, column, container, row, text, text_editor};
use iced::{Color, Element, Font, Length, Padding};
use iced_longbridge::components::button::{Variant, button_ex};
use iced_longbridge::theme::{AppTheme, Size};

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

pub fn editor_view<'a>(theme: &AppTheme, state: &'a EditorState) -> Element<'a, Message> {
    let t = *theme;
    // Header
    let dirty_marker = if state.dirty { " [modified]" } else { "" };
    let header = row![
        text(format!("{}{dirty_marker}", state.file_name))
            .size(14)
            .color(t.foreground),
        Space::new().width(Length::Fill),
        if let Some(ref msg) = state.status_message {
            text(msg.clone())
                .size(12)
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
        button_ex(
            theme,
            "F2 Save",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Editor(EditorMessage::Save)),
            false,
            false
        ),
        Space::new().width(4),
        button_ex(
            theme,
            "F7 Find",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Editor(EditorMessage::Find)),
            false,
            false
        ),
        Space::new().width(4),
        button_ex(
            theme,
            "F10 Quit",
            Variant::Ghost,
            Size::Sm,
            Some(Message::Editor(EditorMessage::Close)),
            false,
            false
        ),
    ]
    .padding(Padding::from([4, 8]));

    let content = column![header, editor, footer];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(t.background)),
            ..Default::default()
        })
        .into()
}
