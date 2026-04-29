pub mod sort;

use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};

use iced::alignment::Horizontal;
use iced::widget::{column, container, text};
use iced::{Color, Element, Font, Length, Padding};
use iced_longbridge::components::table::{
    Column, ResizeEvent, ResizeState, RowStyle, SortDir, table,
};
use iced_longbridge::theme::AppTheme;

use crate::app::{Message, PanelSide};
use crate::util::human_size::format_size;
use crate::util::time_fmt::format_time;
use crate::vfs::{EntryType, VfsEntry, VfsPath};
use crate::widgets::path_input::path_input_view;

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
    Resize(ResizeEvent),
    Refresh,
    PathBarClicked,
    PathInputChanged(String),
    PathInputSubmit,
    PathInputCancel,
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
    pub path_editing: bool,
    pub path_input_value: String,
    pub filter: String,
    pub highlighted: HashSet<usize>,
    pub column_widths: ResizeState,
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
            path_editing: false,
            path_input_value: String::new(),
            filter: String::new(),
            highlighted: HashSet::new(),
            column_widths: ResizeState::new(vec![300.0, 80.0]).min_width(40.0),
        }
    }

    pub fn set_entries(&mut self, mut entries: Vec<VfsEntry>) {
        sort::sort_entries(&mut entries, self.sort_mode, self.sort_ascending);

        // Insert ".." entry at the top if we can navigate up
        // For VFS roots (zip/tar/ftp/sftp), exit to the parent filesystem
        let parent_path = self
            .current_path
            .parent()
            .or_else(|| self.current_path.exit_parent());
        if let Some(parent) = parent_path {
            entries.insert(
                0,
                VfsEntry {
                    name: "..".to_string(),
                    path: parent,
                    entry_type: EntryType::Directory,
                    size: 0,
                    modified: None,
                    permissions: None,
                    owner: None,
                    group: None,
                    link_target: None,
                },
            );
        }

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

const ROW_FONT_SIZE: f32 = 12.0;
const ROW_FONT: &str = "Caskaydia Mono Nerd Font";

/// Per-row data passed to the table's column-render closures. Holds a borrow of
/// the entry plus precomputed visual state so the closures stay cheap.
struct RowView<'e> {
    entry: &'e VfsEntry,
    name_color: Color,
}

fn entry_color(t: &AppTheme, entry: &VfsEntry, is_selected: bool, is_highlighted: bool) -> Color {
    if is_highlighted {
        return Color::from_rgb(1.0, 0.8, 0.3);
    }
    match entry.entry_type {
        EntryType::Directory => Color::from_rgb(0.4, 0.7, 1.0),
        EntryType::Symlink => Color::from_rgb(0.5, 0.9, 0.7),
        EntryType::Special => Color::from_rgb(0.9, 0.6, 0.3),
        EntryType::File => {
            if is_selected {
                Color::from_rgb(1.0, 0.9, 0.3)
            } else {
                t.foreground
            }
        }
    }
}

/// Returns the row's background color based on its state. Distinct from the
/// header's `t.muted` so selected rows don't blend into the header.
fn row_bg(is_cursor: bool, is_selected: bool, is_active: bool) -> Color {
    if is_cursor && is_active {
        // Saturated blue for the focused/cursor row in the active panel.
        Color::from_rgb(0.20, 0.35, 0.65)
    } else if is_cursor {
        // Subtler blue when cursor is on the inactive panel.
        Color::from_rgba(0.20, 0.35, 0.65, 0.35)
    } else if is_selected {
        // Distinct translucent blue for multi-select; clearly different
        // from `t.muted` (used by the header) so the two don't blend.
        Color::from_rgba(0.30, 0.50, 0.85, 0.22)
    } else {
        Color::TRANSPARENT
    }
}

pub fn panel_view<'a>(
    theme: &AppTheme,
    state: &'a PanelState,
    side: PanelSide,
    is_active: bool,
) -> Element<'a, Message> {
    let t = *theme;
    let border_color = if is_active { t.ring } else { t.border };

    // Path header
    let path_bar = path_input_view(
        theme,
        &state.current_path.to_string(),
        state.path_editing,
        &state.path_input_value,
        side,
    );

    // Body: a placeholder for loading / error, otherwise the longbridge table.
    let body: Element<'a, Message> = if state.loading {
        container(text("Loading...").size(13).color(t.muted_foreground))
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if let Some(ref err) = state.error {
        container(text(format!("Error: {err}")).size(13).color(t.danger))
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        // Precompute per-row name color so cell closures stay cheap.
        let row_views: Vec<RowView<'a>> = state
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = state.selected.contains(&i);
                let is_highlighted = state.highlighted.contains(&i);
                RowView {
                    entry,
                    name_color: entry_color(&t, entry, is_selected, is_highlighted),
                }
            })
            .collect();

        let name_col = Column::new("Name", |r: &RowView<'a>| {
            text(&r.entry.name)
                .size(ROW_FONT_SIZE)
                .font(Font::with_name(ROW_FONT))
                .color(r.name_color)
                .into()
        })
        .width(state.column_widths.width(0))
        .sortable("name");

        let size_col = Column::new("Size", move |r: &RowView<'a>| {
            let s: Cow<'static, str> = if r.entry.is_dir() {
                Cow::Borrowed("<DIR>")
            } else {
                Cow::Owned(format_size(r.entry.size))
            };
            text(s)
                .size(ROW_FONT_SIZE)
                .font(Font::with_name(ROW_FONT))
                .color(t.muted_foreground)
                .into()
        })
        .width(state.column_widths.width(1))
        .align(Horizontal::Right)
        .sortable("size");

        let mod_col = Column::new("Modified", move |r: &RowView<'a>| {
            text(format_time(r.entry.modified))
                .size(ROW_FONT_SIZE)
                .font(Font::with_name(ROW_FONT))
                .color(t.muted_foreground)
                .into()
        })
        .width(Length::Fill)
        .sortable("modified");

        let sort_dir = if state.sort_ascending {
            SortDir::Asc
        } else {
            SortDir::Desc
        };
        let sort_state = state.sort_mode.as_key().map(|k| (k, sort_dir));

        let on_sort = move |key: &'static str| {
            let mode = SortMode::from_key(key).unwrap_or(SortMode::Name);
            Message::Panel(side, PanelMessage::Sort(mode))
        };

        table(theme, &row_views, vec![name_col, size_col, mod_col])
            .row_height(22.0)
            .height(Length::Fill)
            .striped(false)
            .row_style(move |i, _r| RowStyle {
                background: Some(row_bg(
                    i == state.cursor,
                    state.selected.contains(&i),
                    is_active,
                )),
                text_color: None,
            })
            .on_row_press(move |i| Message::Panel(side, PanelMessage::Select(i)))
            .sort(sort_state, on_sort)
            .resize(&state.column_widths, move |ev| {
                Message::Panel(side, PanelMessage::Resize(ev))
            })
            .into()
    };

    // Status bar
    let total_size: u64 = state
        .selected_entries()
        .iter()
        .filter(|e| e.is_file())
        .map(|e| e.size)
        .sum();

    let status_text = if !state.filter.is_empty() {
        format!("[Filter: {}] {} items", state.filter, state.entries.len())
    } else if state.selected.is_empty() {
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
            .color(t.muted_foreground),
    )
    .width(Length::Fill)
    .padding(Padding::from([2, 8]));

    // Assemble panel — the table draws its own header divider and outer border.
    let panel_content = column![path_bar, body, status_bar];

    let panel_bg = t.card;
    container(panel_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(panel_bg)),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
