use iced::Element;
use iced_longbridge::components::menu::Item;
use iced_longbridge::components::menu_bar::{MenuBarMenu, menu_bar};
use iced_longbridge::theme::AppTheme;

use crate::app::{Message, PanelSide};
use crate::config::AppConfig;
use crate::panel::PanelMessage;
use crate::panel::sort::SortMode;

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

    fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::File => 1,
            Self::Command => 2,
            Self::Options => 3,
            Self::Right => 4,
        }
    }
}

#[derive(Debug, Default)]
pub struct MenuBarState {
    pub open_menu: Option<MenuId>,
}

fn action(label: &str, shortcut: &str, message: Message) -> Item<Message> {
    let item = Item::new(label, Message::MenuAction(Box::new(message)));
    if shortcut.is_empty() {
        item
    } else {
        item.shortcut(shortcut)
    }
}

fn toggle_action(label: &str, shortcut: &str, checked: bool, message: Message) -> Item<Message> {
    let prefix = if checked { "\u{2713} " } else { "  " };
    action(&format!("{prefix}{label}"), shortcut, message)
}

fn panel_menu_items(side: PanelSide) -> Vec<Item<Message>> {
    vec![
        action(
            "Sort by name",
            "",
            Message::Panel(side, PanelMessage::Sort(SortMode::Name)),
        ),
        action(
            "Sort by extension",
            "",
            Message::Panel(side, PanelMessage::Sort(SortMode::Extension)),
        ),
        action(
            "Sort by size",
            "",
            Message::Panel(side, PanelMessage::Sort(SortMode::Size)),
        ),
        action(
            "Sort by modified",
            "",
            Message::Panel(side, PanelMessage::Sort(SortMode::Modified)),
        ),
        Item::Separator,
        action(
            "Refresh",
            "Ctrl+R",
            Message::Panel(side, PanelMessage::Refresh),
        ),
        Item::Separator,
        action("Filter...", "", Message::OpenFilter(side)),
    ]
}

fn file_menu_items() -> Vec<Item<Message>> {
    vec![
        action("View", "F3", Message::ViewFile),
        action("Edit", "F4", Message::EditFile),
        action("Copy", "F5", Message::CopySelected),
        action("Rename/Move", "F6", Message::MoveSelected),
        action("Mkdir", "F7", Message::Mkdir),
        action("Delete", "F8", Message::DeleteSelected),
        Item::Separator,
        action("Rename", "Shift+F6", Message::Rename),
        action("Chmod", "", Message::Chmod),
        Item::Separator,
        action("Quit", "F10", Message::Quit),
    ]
}

fn command_menu_items() -> Vec<Item<Message>> {
    use crate::bookmarks::BookmarkMessage;
    vec![
        action("Find file", "Ctrl+F", Message::OpenSearch),
        action("Swap panels", "", Message::SwapPanels),
        action("Compare directories", "", Message::CompareDirectories),
        Item::Separator,
        action(
            "Directory hotlist",
            "Ctrl+D",
            Message::Bookmark(BookmarkMessage::Open),
        ),
    ]
}

fn options_menu_items(config: &AppConfig) -> Vec<Item<Message>> {
    vec![
        toggle_action(
            "Show hidden files",
            "Ctrl+H",
            config.show_hidden,
            Message::ToggleHidden,
        ),
        toggle_action(
            "Confirm delete",
            "",
            config.confirm_delete,
            Message::ToggleConfirmDelete,
        ),
        toggle_action(
            "Confirm overwrite",
            "",
            config.confirm_overwrite,
            Message::ToggleConfirmOverwrite,
        ),
        Item::Separator,
        action("Save setup", "", Message::SaveConfig),
    ]
}

pub fn menu_bar_view<'a>(
    theme: &AppTheme,
    state: &MenuBarState,
    config: &'a AppConfig,
) -> Element<'a, Message> {
    let menus = vec![
        MenuBarMenu::new(
            "Left",
            Message::MenuOpen(MenuId::Left),
            panel_menu_items(PanelSide::Left),
        ),
        MenuBarMenu::new(
            "File",
            Message::MenuOpen(MenuId::File),
            file_menu_items(),
        ),
        MenuBarMenu::new(
            "Command",
            Message::MenuOpen(MenuId::Command),
            command_menu_items(),
        ),
        MenuBarMenu::new(
            "Options",
            Message::MenuOpen(MenuId::Options),
            options_menu_items(config),
        ),
        MenuBarMenu::new(
            "Right",
            Message::MenuOpen(MenuId::Right),
            panel_menu_items(PanelSide::Right),
        ),
    ];

    menu_bar(theme, menus, state.open_menu.map(MenuId::index))
}
