use iced::widget::{Space, column, text};
use iced::Element;
use iced_longbridge::components::progress::progress;
use iced_longbridge::theme::AppTheme;

use crate::app::Message;
use crate::util::human_size::format_size;

#[derive(Debug, Clone)]
pub struct ProgressDialog {
    pub title: String,
    pub current_file: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub files_done: usize,
    pub files_total: usize,
}

impl ProgressDialog {
    pub fn percent(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.transferred_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }
}

pub fn progress_view<'a>(theme: &AppTheme, dialog: &'a ProgressDialog) -> Element<'a, Message> {
    let t = *theme;
    let title = text(&dialog.title).size(16).color(t.foreground);
    let file_text = text(&dialog.current_file).size(12).color(t.muted_foreground);

    let bar = progress(theme, dialog.percent());

    let stats = text(format!(
        "{} / {} ({}/{})",
        format_size(dialog.transferred_bytes),
        format_size(dialog.total_bytes),
        dialog.files_done,
        dialog.files_total,
    ))
    .size(12)
    .color(t.muted_foreground);

    column![
        title,
        Space::new().height(8),
        file_text,
        Space::new().height(8),
        bar,
        Space::new().height(4),
        stats,
    ]
    .into()
}
