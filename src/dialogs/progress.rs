use iced::widget::{Space, column, progress_bar, text};
use iced::{Color, Element, Font, Length};

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
    pub fn fraction(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.transferred_bytes as f32 / self.total_bytes as f32
        }
    }
}

pub fn progress_view<'a>(dialog: &'a ProgressDialog) -> Element<'a, Message> {
    let title = text(&dialog.title)
        .size(16)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let file_text = text(&dialog.current_file)
        .size(12)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.6, 0.6, 0.65));

    let bar = progress_bar(0.0..=1.0, dialog.fraction()).girth(8);

    let stats = text(format!(
        "{} / {} ({}/{})",
        format_size(dialog.transferred_bytes),
        format_size(dialog.total_bytes),
        dialog.files_done,
        dialog.files_total,
    ))
    .size(12)
    .font(Font::with_name("Caskaydia Mono Nerd Font"))
    .color(Color::from_rgb(0.6, 0.6, 0.65));

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
