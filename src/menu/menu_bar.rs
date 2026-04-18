use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::{Message, PanelSide};
use crate::config::AppConfig;
use crate::panel::sort::SortMode;
use crate::panel::PanelMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuId {
    Left,
    File,
    Command,
    Options,
    Right,
}

impl MenuId {
    pub fn prev(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::File => Self::Left,
            Self::Command => Self::File,
            Self::Options => Self::Command,
            Self::Right => Self::Options,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Left => Self::File,
            Self::File => Self::Command,
            Self::Command => Self::Options,
            Self::Options => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Default)]
pub struct MenuBarState {
    pub open_menu: Option<MenuId>,
}

// ── Header bar ──────────────────────────────────────────────────────

const MENU_FONT_SIZE: f32 = 13.0;
const HEADER_WIDTH: f32 = 100.0;

fn menu_header_button<'a>(
    label: &'a str,
    id: MenuId,
    is_open: bool,
) -> Element<'a, Message> {
    let bg = if is_open {
        Color::from_rgb(0.2, 0.35, 0.7)
    } else {
        Color::TRANSPARENT
    };

    button(
        text(label)
            .size(MENU_FONT_SIZE)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.85, 0.85, 0.9)),
    )
    .padding(Padding::from([3, 10]))
    .width(Length::Fixed(HEADER_WIDTH))
    .style(move |_theme, _status| button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: Color::WHITE,
        border: iced::Border::default(),
        shadow: iced::Shadow::default(),
    })
    .on_press(Message::MenuOpen(id))
    .into()
}

pub fn menu_bar_view(state: &MenuBarState) -> Element<'_, Message> {
    let open = state.open_menu;
    let headers = row![
        menu_header_button("Left", MenuId::Left, open == Some(MenuId::Left)),
        menu_header_button("File", MenuId::File, open == Some(MenuId::File)),
        menu_header_button("Command", MenuId::Command, open == Some(MenuId::Command)),
        menu_header_button("Options", MenuId::Options, open == Some(MenuId::Options)),
        Space::with_width(Length::Fill),
        menu_header_button("Right", MenuId::Right, open == Some(MenuId::Right)),
    ];

    container(headers)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.08, 0.08, 0.1))),
            ..Default::default()
        })
        .into()
}

// ── Dropdown overlay ────────────────────────────────────────────────

enum MenuItem {
    Action {
        label: &'static str,
        shortcut: &'static str,
        message: Message,
    },
    Toggle {
        label: &'static str,
        shortcut: &'static str,
        checked: bool,
        message: Message,
    },
    Separator,
}

fn menu_item_button(label: String, shortcut: &str, message: Message) -> Element<'_, Message> {
    let content = row![
        text(label)
            .size(MENU_FONT_SIZE)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.85, 0.85, 0.9))
            .width(Length::Fill),
        text(shortcut)
            .size(MENU_FONT_SIZE)
            .font(Font::with_name("Caskaydia Mono Nerd Font"))
            .color(Color::from_rgb(0.5, 0.5, 0.55)),
    ]
    .spacing(16);

    button(content)
        .padding(Padding::from([4, 12]))
        .width(Length::Fill)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => Color::from_rgb(0.2, 0.25, 0.4),
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        })
        .on_press(Message::MenuAction(Box::new(message)))
        .into()
}

fn menu_separator<'a>() -> Element<'a, Message> {
    container(iced::widget::horizontal_rule(1))
        .padding(Padding::from([2, 8]))
        .width(Length::Fill)
        .into()
}

fn render_menu_items<'a>(items: Vec<MenuItem>) -> Element<'a, Message> {
    let mut children: Vec<Element<'_, Message>> = Vec::new();

    for item in items {
        children.push(match item {
            MenuItem::Action {
                label,
                shortcut,
                message,
            } => menu_item_button(format!("  {label}"), shortcut, message),
            MenuItem::Toggle {
                label,
                shortcut,
                checked,
                message,
            } => {
                let prefix = if checked { "\u{2713} " } else { "  " };
                menu_item_button(format!("{prefix}{label}"), shortcut, message)
            }
            MenuItem::Separator => menu_separator(),
        });
    }

    container(
        column(children).width(Length::Fixed(240.0)),
    )
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
        border: iced::Border {
            color: Color::from_rgb(0.25, 0.25, 0.3),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .padding(Padding::from([4, 0]))
    .into()
}

fn panel_menu_items(side: PanelSide) -> Vec<MenuItem> {
    vec![
        MenuItem::Action {
            label: "Sort by name",
            shortcut: "",
            message: Message::Panel(side, PanelMessage::Sort(SortMode::Name)),
        },
        MenuItem::Action {
            label: "Sort by extension",
            shortcut: "",
            message: Message::Panel(side, PanelMessage::Sort(SortMode::Extension)),
        },
        MenuItem::Action {
            label: "Sort by size",
            shortcut: "",
            message: Message::Panel(side, PanelMessage::Sort(SortMode::Size)),
        },
        MenuItem::Action {
            label: "Sort by modified",
            shortcut: "",
            message: Message::Panel(side, PanelMessage::Sort(SortMode::Modified)),
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Refresh",
            shortcut: "Ctrl+R",
            message: Message::Panel(side, PanelMessage::Refresh),
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Filter...",
            shortcut: "",
            message: Message::OpenFilter(side),
        },
    ]
}

fn file_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::Action {
            label: "View",
            shortcut: "F3",
            message: Message::ViewFile,
        },
        MenuItem::Action {
            label: "Edit",
            shortcut: "F4",
            message: Message::EditFile,
        },
        MenuItem::Action {
            label: "Copy",
            shortcut: "F5",
            message: Message::CopySelected,
        },
        MenuItem::Action {
            label: "Rename/Move",
            shortcut: "F6",
            message: Message::MoveSelected,
        },
        MenuItem::Action {
            label: "Mkdir",
            shortcut: "F7",
            message: Message::Mkdir,
        },
        MenuItem::Action {
            label: "Delete",
            shortcut: "F8",
            message: Message::DeleteSelected,
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Rename",
            shortcut: "Shift+F6",
            message: Message::Rename,
        },
        MenuItem::Action {
            label: "Chmod",
            shortcut: "",
            message: Message::Chmod,
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Quit",
            shortcut: "F10",
            message: Message::Quit,
        },
    ]
}

fn command_menu_items() -> Vec<MenuItem> {
    use crate::bookmarks::BookmarkMessage;
    vec![
        MenuItem::Action {
            label: "Find file",
            shortcut: "Ctrl+F",
            message: Message::OpenSearch,
        },
        MenuItem::Action {
            label: "Swap panels",
            shortcut: "",
            message: Message::SwapPanels,
        },
        MenuItem::Action {
            label: "Compare directories",
            shortcut: "",
            message: Message::CompareDirectories,
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Directory hotlist",
            shortcut: "Ctrl+D",
            message: Message::Bookmark(BookmarkMessage::Open),
        },
    ]
}

fn options_menu_items(config: &AppConfig) -> Vec<MenuItem> {
    vec![
        MenuItem::Toggle {
            label: "Show hidden files",
            shortcut: "Ctrl+H",
            checked: config.show_hidden,
            message: Message::ToggleHidden,
        },
        MenuItem::Toggle {
            label: "Confirm delete",
            shortcut: "",
            checked: config.confirm_delete,
            message: Message::ToggleConfirmDelete,
        },
        MenuItem::Toggle {
            label: "Confirm overwrite",
            shortcut: "",
            checked: config.confirm_overwrite,
            message: Message::ToggleConfirmOverwrite,
        },
        MenuItem::Separator,
        MenuItem::Action {
            label: "Save setup",
            shortcut: "",
            message: Message::SaveConfig,
        },
    ]
}

pub fn menu_dropdown_overlay<'a>(
    state: &MenuBarState,
    config: &AppConfig,
) -> Option<Element<'a, Message>> {
    let menu_id = state.open_menu?;

    let items = match menu_id {
        MenuId::Left => panel_menu_items(PanelSide::Left),
        MenuId::File => file_menu_items(),
        MenuId::Command => command_menu_items(),
        MenuId::Options => options_menu_items(config),
        MenuId::Right => panel_menu_items(PanelSide::Right),
    };

    let dropdown = render_menu_items(items);

    // Compute horizontal offset based on menu position
    let leading: Element<'a, Message> = match menu_id {
        MenuId::Left => Space::with_width(Length::Fixed(0.0)).into(),
        MenuId::File => Space::with_width(Length::Fixed(HEADER_WIDTH)).into(),
        MenuId::Command => Space::with_width(Length::Fixed(HEADER_WIDTH * 2.0)).into(),
        MenuId::Options => Space::with_width(Length::Fixed(HEADER_WIDTH * 3.0)).into(),
        MenuId::Right => Space::with_width(Length::Fill).into(),
    };

    let positioned = row![leading, column![Space::with_height(24), dropdown]];

    // Click-catcher backdrop: closes menu when clicking outside
    let backdrop = button(Space::new(Length::Fill, Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme, _status| button::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            text_color: Color::TRANSPARENT,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
        })
        .on_press(Message::MenuClose);

    // Stack: backdrop behind, dropdown on top, both at the top of the screen
    let overlay = iced::widget::stack![
        backdrop,
        container(positioned)
            .width(Length::Fill)
            .padding(Padding::from([0, 0])),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    Some(overlay.into())
}
