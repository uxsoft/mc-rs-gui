pub mod sort;

use std::collections::BTreeSet;

use iced::widget::{
    Column, Space, button, column, container, horizontal_rule, row, scrollable, text,
};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::{Message, PanelSide};
use crate::util::human_size::format_size;
use crate::util::time_fmt::format_time;
use crate::vfs::{EntryType, VfsEntry, VfsPath};

use self::sort::SortMode;

#[derive(Debug, Clone)]
pub enum PanelMessage {
    Navigate(VfsPath),
    GoUp,
    Enter,
    Select(usize),
    ToggleSelect(usize),
    CursorMove(i32),
    CursorPage(i32),
    CursorHome,
    CursorEnd,
    Sort(SortMode),
    Refresh,
}

pub struct PanelState {
    pub current_path: VfsPath,
    pub entries: Vec<VfsEntry>,
    pub selected: BTreeSet<usize>,
    pub cursor: usize,
    pub sort_mode: SortMode,
    pub sort_ascending: bool,
    pub loading: bool,
    pub error: Option<String>,
}

impl PanelState {
    pub fn new(path: VfsPath) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            selected: BTreeSet::new(),
            cursor: 0,
            sort_mode: SortMode::Name,
            sort_ascending: true,
            loading: true,
            error: None,
        }
    }

    pub fn set_entries(&mut self, mut entries: Vec<VfsEntry>) {
        sort::sort_entries(&mut entries, self.sort_mode, self.sort_ascending);
        self.entries = entries;
        self.selected.clear();
        self.cursor = 0;
        self.loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    pub fn move_cursor(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let new = self.cursor as i32 + delta;
        self.cursor = new.clamp(0, self.entries.len() as i32 - 1) as usize;
    }

    pub fn move_cursor_page(&mut self, delta: i32) {
        self.move_cursor(delta * 20);
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = self.entries.len() - 1;
        }
    }

    pub fn toggle_select(&mut self, index: usize) {
        if self.selected.contains(&index) {
            self.selected.remove(&index);
        } else {
            self.selected.insert(index);
        }
    }

    pub fn current_entry(&self) -> Option<&VfsEntry> {
        self.entries.get(self.cursor)
    }

    pub fn selected_entries(&self) -> Vec<&VfsEntry> {
        if self.selected.is_empty() {
            self.current_entry().into_iter().collect()
        } else {
            self.selected
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .collect()
        }
    }

    pub fn resort(&mut self) {
        sort::sort_entries(&mut self.entries, self.sort_mode, self.sort_ascending);
    }
}

pub fn panel_view<'a>(
    state: &'a PanelState,
    side: PanelSide,
    is_active: bool,
) -> Element<'a, Message> {
    let border_color = if is_active {
        Color::from_rgb(0.3, 0.5, 0.9)
    } else {
        Color::from_rgb(0.3, 0.3, 0.35)
    };

    // Path header
    let path_text = text(state.current_path.to_string())
        .size(13)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.7, 0.8, 0.95));

    let path_bar = container(path_text)
        .width(Length::Fill)
        .padding(Padding::from([4, 8]));

    // Column headers
    let header_row = row![
        text("Name")
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65))
            .width(Length::Fill),
        text("Size")
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65))
            .width(Length::Fixed(80.0)),
        text("Modified")
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.6, 0.6, 0.65))
            .width(Length::Fixed(140.0)),
    ]
    .spacing(4)
    .padding(Padding::from([2, 8]));

    // File list
    let file_rows: Vec<Element<'a, Message>> = if state.loading {
        vec![
            container(
                text("Loading...")
                    .size(13)
                    .color(Color::from_rgb(0.5, 0.5, 0.55)),
            )
            .padding(8)
            .into(),
        ]
    } else if let Some(ref err) = state.error {
        vec![
            container(
                text(format!("Error: {err}"))
                    .size(13)
                    .color(Color::from_rgb(0.9, 0.3, 0.3)),
            )
            .padding(8)
            .into(),
        ]
    } else {
        // ".." entry to go up
        let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(state.entries.len() + 1);

        if state.current_path.parent().is_some() {
            let up_bg = if state.cursor == 0 && state.entries.is_empty() {
                Color::from_rgb(0.2, 0.25, 0.35)
            } else {
                Color::TRANSPARENT
            };

            let up_row = button(
                row![
                    text("/..")
                        .size(13)
                        .font(Font::with_name("Caskaydia Mono Nerd Font"))
                        .color(Color::from_rgb(0.8, 0.8, 0.85))
                        .width(Length::Fill),
                    text("<DIR>")
                        .size(13)
                        .font(Font::with_name("Caskaydia Mono Nerd Font"))
                        .color(Color::from_rgb(0.5, 0.5, 0.55))
                        .width(Length::Fixed(80.0)),
                    Space::with_width(Length::Fixed(140.0)),
                ]
                .spacing(4),
            )
            .padding(Padding::from([1, 8]))
            .width(Length::Fill)
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(up_bg)),
                text_color: Color::WHITE,
                ..Default::default()
            })
            .on_press(Message::Panel(side, PanelMessage::GoUp));

            rows.push(up_row.into());
        }

        for (i, entry) in state.entries.iter().enumerate() {
            let is_cursor = i == state.cursor;
            let is_selected = state.selected.contains(&i);

            let name_color = match entry.entry_type {
                EntryType::Directory => Color::from_rgb(0.4, 0.7, 1.0),
                EntryType::Symlink => Color::from_rgb(0.5, 0.9, 0.7),
                EntryType::Special => Color::from_rgb(0.9, 0.6, 0.3),
                EntryType::File => {
                    if is_selected {
                        Color::from_rgb(1.0, 0.9, 0.3)
                    } else {
                        Color::from_rgb(0.85, 0.85, 0.9)
                    }
                }
            };

            let bg_color = if is_cursor && is_active {
                Color::from_rgb(0.2, 0.25, 0.4)
            } else if is_selected {
                Color::from_rgb(0.18, 0.2, 0.3)
            } else {
                Color::TRANSPARENT
            };

            let size_text = if entry.is_dir() {
                "<DIR>".to_string()
            } else {
                format_size(entry.size)
            };

            let modified_text = format_time(entry.modified);

            let entry_row = button(
                row![
                    text(&entry.name)
                        .size(13)
                        .font(Font::with_name("Caskaydia Mono Nerd Font"))
                        .color(name_color)
                        .width(Length::Fill),
                    text(size_text)
                        .size(13)
                        .font(Font::with_name("Caskaydia Mono Nerd Font"))
                        .color(Color::from_rgb(0.6, 0.6, 0.65))
                        .width(Length::Fixed(80.0)),
                    text(modified_text)
                        .size(13)
                        .font(Font::with_name("Caskaydia Mono Nerd Font"))
                        .color(Color::from_rgb(0.5, 0.5, 0.55))
                        .width(Length::Fixed(140.0)),
                ]
                .spacing(4),
            )
            .padding(Padding::from([1, 8]))
            .width(Length::Fill)
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            })
            .on_press(Message::Panel(side, PanelMessage::Select(i)));

            rows.push(entry_row.into());
        }

        rows
    };

    let file_list = scrollable(Column::with_children(file_rows).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill);

    // Status bar
    let total_size: u64 = state
        .selected_entries()
        .iter()
        .filter(|e| e.is_file())
        .map(|e| e.size)
        .sum();

    let status_text = if state.selected.is_empty() {
        format!("{} items", state.entries.len())
    } else {
        format!(
            "{} selected ({})",
            state.selected.len(),
            format_size(total_size)
        )
    };

    let status_bar = container(
        text(status_text)
            .size(12)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.5, 0.5, 0.55)),
    )
    .width(Length::Fill)
    .padding(Padding::from([2, 8]));

    // Assemble panel
    let panel_content = column![
        path_bar,
        horizontal_rule(1),
        header_row,
        file_list,
        status_bar,
    ];

    container(panel_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.13))),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
