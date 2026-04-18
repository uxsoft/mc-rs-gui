use iced::widget::{Space, checkbox, column, row, text, text_input};
use iced::{Color, Element, Font, Length};

use crate::app::Message;
use crate::vfs::VfsPath;

use super::{DialogMessage, dialog_button};

#[derive(Debug, Clone)]
pub struct ChmodDialog {
    pub path: VfsPath,
    pub file_name: String,
    pub mode: u32,
    pub octal_input: String,
}

impl ChmodDialog {
    pub fn new(path: VfsPath, file_name: String, mode: u32) -> Self {
        Self {
            path,
            file_name,
            mode,
            octal_input: format!("{:04o}", mode & 0o7777),
        }
    }
}

const BITS: [(u32, &str); 9] = [
    (0o400, "Owner read"),
    (0o200, "Owner write"),
    (0o100, "Owner exec"),
    (0o040, "Group read"),
    (0o020, "Group write"),
    (0o010, "Group exec"),
    (0o004, "Other read"),
    (0o002, "Other write"),
    (0o001, "Other exec"),
];

fn mode_string(mode: u32) -> String {
    let mut s = String::with_capacity(9);
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    s
}

pub fn chmod_view<'a>(dialog: &'a ChmodDialog) -> Element<'a, Message> {
    let title = text("Change Permissions")
        .size(16)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.9, 0.9, 0.95));

    let file_label = text(format!("File: {}", dialog.file_name))
        .size(13)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.7, 0.7, 0.75));

    let mode_display = text(format!(
        "{} ({:04o})",
        mode_string(dialog.mode),
        dialog.mode & 0o7777
    ))
    .size(14)
    .font(Font::with_name("Caskaydia Mono Nerd Font"))
    .color(Color::from_rgb(0.6, 0.8, 1.0));

    // Permission checkboxes in 3 rows of 3
    let mut perm_rows: Vec<Element<'a, Message>> = Vec::new();
    for chunk in BITS.chunks(3) {
        let mut row_items: Vec<Element<'a, Message>> = Vec::new();
        for &(bit, label) in chunk {
            let checked = dialog.mode & bit != 0;
            row_items.push(
                checkbox(checked)
                    .label(label)
                    .on_toggle(move |_| Message::DialogResult(DialogMessage::ChmodToggleBit(bit)))
                    .text_size(12)
                    .size(16)
                    .font(Font::with_name("Caskaydia Mono Nerd Font"))
                    .into(),
            );
            row_items.push(Space::new().width(Length::Fixed(12.0)).into());
        }
        perm_rows.push(
            iced::widget::Row::with_children(row_items)
                .spacing(4)
                .into(),
        );
    }

    let octal_label = text("Octal:")
        .size(13)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .color(Color::from_rgb(0.7, 0.7, 0.75));

    let octal = text_input("0755", &dialog.octal_input)
        .on_input(|s| Message::DialogResult(DialogMessage::ChmodOctalChanged(s)))
        .size(14)
        .font(Font::with_name("Caskaydia Mono Nerd Font"))
        .width(Length::Fixed(80.0));

    let octal_row =
        row![octal_label, Space::new().width(8), octal].align_y(iced::Alignment::Center);

    let buttons = row![
        dialog_button(
            "Apply",
            Message::DialogResult(DialogMessage::ChmodApply),
            true
        ),
        Space::new().width(8),
        dialog_button(
            "Cancel",
            Message::DialogResult(DialogMessage::Cancel),
            false
        ),
    ];

    let mut content = column![
        title,
        Space::new().height(8),
        file_label,
        Space::new().height(4),
        mode_display,
        Space::new().height(12),
    ]
    .spacing(0);

    for r in perm_rows {
        content = content.push(r);
        content = content.push(Space::new().height(Length::Fixed(4.0)));
    }

    content = content.push(Space::new().height(8));
    content = content.push(octal_row);
    content = content.push(Space::new().height(16));
    content = content.push(buttons);

    content.into()
}
