use std::sync::Arc;

use iced::keyboard;
use iced::widget::{column, container, row, stack, text_input, operation};
use iced::{Color, Element, Length, Subscription, Task};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::bookmarks::storage::{load_bookmarks, save_bookmarks};
use crate::bookmarks::{Bookmark, BookmarkMessage, BookmarkStore};
use crate::config::AppConfig;
use crate::dialogs::chmod::ChmodDialog;
use crate::dialogs::confirm::ConfirmDialog;
use crate::dialogs::input::{self, InputDialog};
use crate::dialogs::progress::ProgressDialog;
use crate::dialogs::{self, DialogKind, DialogMessage};
use crate::editor::{self, EditorMessage, EditorState};
use crate::menu;
use crate::menu::menu_bar::{MenuBarState, MenuId};
use crate::operations::executor::execute_operation;
use crate::operations::{OperationKind, OperationProgress};
use crate::panel::{self, PanelMessage, PanelState};
use crate::search::finder::search_files;
use crate::search::{self, SearchMessage, SearchState};
use crate::vfs::archive::ArchiveVfsProvider;
use crate::vfs::local::LocalVfsProvider;
use crate::vfs::{VfsEntry, VfsPath, VfsRouter};
use crate::viewer::{self, ViewMode, ViewerMessage, ViewerState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Panel
    Panel(PanelSide, PanelMessage),
    SwitchPanel,

    // File operations
    CopySelected,
    MoveSelected,
    DeleteSelected,
    Mkdir,
    Rename,
    ViewFile,
    EditFile,

    // Operation lifecycle
    OperationProgress(OperationProgress),
    OperationComplete(Result<(), String>),
    StartOperation(OperationKind),

    // Viewer
    Viewer(ViewerMessage),
    FileLoaded(String, Result<Vec<u8>, String>),

    // Editor
    Editor(EditorMessage),
    FileLoadedForEdit(String, crate::vfs::VfsPath, Result<Vec<u8>, String>),
    FileSaved(Result<(), String>),

    // Search
    Search(SearchMessage),
    OpenSearch,

    // Bookmarks
    Bookmark(BookmarkMessage),

    // Dialogs
    DialogResult(DialogMessage),

    // Async results
    DirectoryLoaded(PanelSide, Result<Vec<VfsEntry>, String>),

    // Keyboard
    KeyPressed(keyboard::Key, keyboard::Modifiers),

    // App
    ToggleHidden,
    Quit,

    // Menu bar
    MenuOpen(MenuId),
    MenuClose,
    MenuAction(Box<Message>),

    // New trivial actions
    SwapPanels,
    ToggleConfirmDelete,
    ToggleConfirmOverwrite,
    SaveConfig,

    // Filter, Chmod, Compare
    OpenFilter(PanelSide),
    Chmod,
    CompareDirectories,
}

pub struct App {
    pub left_panel: PanelState,
    pub right_panel: PanelState,
    pub active_panel: PanelSide,
    pub vfs: Arc<VfsRouter>,
    pub dialog: Option<DialogKind>,
    pub pending_operation: Option<OperationKind>,
    pub viewer: Option<ViewerState>,
    pub editor: Option<EditorState>,
    pub search: Option<SearchState>,
    pub bookmarks: BookmarkStore,
    pub config: AppConfig,
    pub menu_bar: MenuBarState,
    pub pending_filter_side: Option<PanelSide>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let home = dirs::home_dir().unwrap_or_else(|| "/".into());
        let vfs = Arc::new(VfsRouter::new(vec![
            Box::new(LocalVfsProvider::new()),
            Box::new(ArchiveVfsProvider::new()),
        ]));

        let left_path = VfsPath::local(&home);
        let right_path = VfsPath::local(&home);

        let config = AppConfig::load();
        let mut bookmark_store = BookmarkStore::new();
        bookmark_store.bookmarks = load_bookmarks();

        let app = Self {
            left_panel: PanelState::new(left_path.clone()),
            right_panel: PanelState::new(right_path.clone()),
            active_panel: PanelSide::Left,
            vfs: vfs.clone(),
            dialog: None,
            pending_operation: None,
            viewer: None,
            editor: None,
            search: None,
            bookmarks: bookmark_store,
            config,
            menu_bar: MenuBarState::default(),
            pending_filter_side: None,
        };

        let vfs_l = vfs.clone();
        let vfs_r = vfs.clone();
        let left_task = Task::perform(
            async move { vfs_l.read_dir(&left_path).await.map_err(|e| e.to_string()) },
            |result| Message::DirectoryLoaded(PanelSide::Left, result),
        );
        let right_task = Task::perform(
            async move { vfs_r.read_dir(&right_path).await.map_err(|e| e.to_string()) },
            |result| Message::DirectoryLoaded(PanelSide::Right, result),
        );

        (app, Task::batch([left_task, right_task]))
    }

    pub fn title(&self) -> String {
        format!("mc-rs - {}", self.active_panel_state().current_path)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Panel(side, panel_msg) => {
                return self.handle_panel_message(side, panel_msg);
            }
            Message::SwitchPanel => {
                self.active_panel = match self.active_panel {
                    PanelSide::Left => PanelSide::Right,
                    PanelSide::Right => PanelSide::Left,
                };
            }
            Message::DirectoryLoaded(side, result) => {
                let show_hidden = self.config.show_hidden;
                let panel = self.panel_mut(side);
                match result {
                    Ok(mut entries) => {
                        if !show_hidden {
                            entries.retain(|e| !e.name.starts_with('.'));
                        }
                        if !panel.filter.is_empty() {
                            let filter = panel.filter.clone();
                            entries.retain(|e| e.name == ".." || glob_match(&filter, &e.name));
                        }
                        panel.set_entries(entries);
                    }
                    Err(err) => panel.set_error(err),
                }
            }
            Message::KeyPressed(key, modifiers) => {
                // Viewer keys
                if self.viewer.is_some() {
                    return self.handle_viewer_key(key, modifiers);
                }
                // Editor keys
                if self.editor.is_some() {
                    return self.handle_editor_key(key, modifiers);
                }
                // Search close
                if self.search.is_some() {
                    if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                        self.search = None;
                        return Task::none();
                    }
                    return Task::none();
                }
                // Dialog close
                if self.dialog.is_some() {
                    if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                        self.dialog = None;
                        self.pending_operation = None;
                        return Task::none();
                    }
                    return Task::none();
                }
                // Menu bar keyboard navigation
                if self.menu_bar.open_menu.is_some() {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::Escape) => {
                            self.menu_bar.open_menu = None;
                            return Task::none();
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            self.menu_bar.open_menu =
                                Some(self.menu_bar.open_menu.unwrap().prev());
                            return Task::none();
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            self.menu_bar.open_menu =
                                Some(self.menu_bar.open_menu.unwrap().next());
                            return Task::none();
                        }
                        _ => {
                            self.menu_bar.open_menu = None;
                            return self.handle_key(key, modifiers);
                        }
                    }
                }
                return self.handle_key(key, modifiers);
            }

            // --- File operations ---
            Message::CopySelected => return self.initiate_copy(),
            Message::MoveSelected => return self.initiate_move(),
            Message::DeleteSelected => return self.initiate_delete(),
            Message::Mkdir => return self.initiate_mkdir(),
            Message::Rename => return self.initiate_rename(),

            Message::StartOperation(op) => {
                self.dialog = None;
                return self.start_operation(op);
            }
            Message::OperationProgress(progress) => {
                self.dialog = Some(DialogKind::Progress(ProgressDialog {
                    title: "Operation in progress...".into(),
                    current_file: progress.current_file.clone(),
                    total_bytes: progress.total_bytes,
                    transferred_bytes: progress.transferred_bytes,
                    files_done: progress.files_done,
                    files_total: progress.files_total,
                }));
            }
            Message::OperationComplete(result) => {
                self.dialog = None;
                if let Err(err) = result {
                    self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
                        title: "Error".into(),
                        message: err,
                        on_confirm: Box::new(Message::DialogResult(DialogMessage::Cancel)),
                    }));
                }
                return self.refresh_both_panels();
            }

            Message::DialogResult(dialog_msg) => {
                return self.handle_dialog(dialog_msg);
            }

            // --- Viewer ---
            Message::ViewFile => {
                return self.open_viewer();
            }
            Message::FileLoaded(name, result) => match result {
                Ok(content) => {
                    self.viewer = Some(ViewerState::new(name, content));
                }
                Err(err) => {
                    self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
                        title: "Error".into(),
                        message: format!("Cannot open file: {err}"),
                        on_confirm: Box::new(Message::DialogResult(DialogMessage::Cancel)),
                    }));
                }
            },
            Message::Viewer(viewer_msg) => {
                return self.handle_viewer(viewer_msg);
            }

            // --- Editor ---
            Message::EditFile => {
                return self.open_editor();
            }
            Message::FileLoadedForEdit(name, path, result) => match result {
                Ok(content) => {
                    let text = String::from_utf8_lossy(&content).to_string();
                    self.editor = Some(EditorState::new(name, path, text));
                }
                Err(err) => {
                    self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
                        title: "Error".into(),
                        message: format!("Cannot open file: {err}"),
                        on_confirm: Box::new(Message::DialogResult(DialogMessage::Cancel)),
                    }));
                }
            },
            Message::Editor(editor_msg) => {
                return self.handle_editor(editor_msg);
            }
            Message::FileSaved(result) => {
                if let Some(ref mut editor) = self.editor {
                    match result {
                        Ok(()) => {
                            editor.dirty = false;
                            editor.status_message = Some("Saved".into());
                        }
                        Err(err) => {
                            editor.status_message = Some(format!("Save failed: {err}"));
                        }
                    }
                }
            }

            // --- Search ---
            Message::OpenSearch => {
                let dir = self.active_panel_state().current_path.to_string();
                self.search = Some(SearchState::new(dir));
            }
            Message::Search(search_msg) => {
                return self.handle_search(search_msg);
            }

            // --- Bookmarks ---
            Message::Bookmark(bm_msg) => {
                return self.handle_bookmark(bm_msg);
            }

            Message::ToggleHidden => {
                self.config.show_hidden = !self.config.show_hidden;
                self.config.save();
                return self.refresh_both_panels();
            }

            Message::Quit => {
                return iced::exit();
            }

            // Menu bar
            Message::MenuOpen(id) => {
                if self.menu_bar.open_menu == Some(id) {
                    self.menu_bar.open_menu = None;
                } else {
                    self.menu_bar.open_menu = Some(id);
                }
            }
            Message::MenuClose => {
                self.menu_bar.open_menu = None;
            }
            Message::MenuAction(inner) => {
                self.menu_bar.open_menu = None;
                return self.update(*inner);
            }

            // Trivial new actions
            Message::SwapPanels => {
                std::mem::swap(&mut self.left_panel, &mut self.right_panel);
                self.active_panel = match self.active_panel {
                    PanelSide::Left => PanelSide::Right,
                    PanelSide::Right => PanelSide::Left,
                };
            }
            Message::ToggleConfirmDelete => {
                self.config.confirm_delete = !self.config.confirm_delete;
            }
            Message::ToggleConfirmOverwrite => {
                self.config.confirm_overwrite = !self.config.confirm_overwrite;
            }
            Message::SaveConfig => {
                self.config.save();
            }

            Message::OpenFilter(side) => {
                let current_filter = self.panel_mut(side).filter.clone();
                self.pending_filter_side = Some(side);
                self.dialog = Some(DialogKind::Input(InputDialog {
                    title: "Filter".into(),
                    label: "Pattern (e.g. *.rs, empty to clear):".into(),
                    value: current_filter,
                    on_submit: |_| Message::DialogResult(DialogMessage::InputSubmit),
                }));
                return operation::focus(input::INPUT_DIALOG_ID);
            }

            Message::Chmod => {
                return self.initiate_chmod();
            }

            Message::CompareDirectories => {
                self.compare_directories();
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Viewer takes over the whole screen
        if let Some(ref viewer) = self.viewer {
            return viewer::viewer_view(viewer);
        }

        // Editor takes over the whole screen
        if let Some(ref editor) = self.editor {
            return editor::editor_view(editor);
        }

        let top_menu = menu::menu_bar::menu_bar_view(&self.menu_bar);
        let left = panel::panel_view(
            &self.left_panel,
            PanelSide::Left,
            self.active_panel == PanelSide::Left,
        );
        let right = panel::panel_view(
            &self.right_panel,
            PanelSide::Right,
            self.active_panel == PanelSide::Right,
        );

        let panels = row![left, right].spacing(2).height(Length::Fill);
        let fn_bar = menu::fn_key_bar();
        let main_content = column![top_menu, panels, fn_bar];

        let base: Element<'_, Message> = container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.06, 0.06, 0.08))),
                ..Default::default()
            })
            .into();

        // Menu dropdown overlay (highest priority)
        if let Some(dropdown) = menu::menu_bar::menu_dropdown_overlay(&self.menu_bar, &self.config)
        {
            return stack![base, dropdown].into();
        }

        // Overlay search dialog
        if let Some(ref search_state) = self.search {
            return stack![base, search::search_view(search_state)].into();
        }

        // Overlay regular dialog
        if let Some(ref dialog) = self.dialog {
            return stack![base, dialogs::dialog_overlay(dialog)].into();
        }

        base
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| {
            match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    Message::KeyPressed(key, modifiers)
                }
                _ => Message::KeyPressed(
                    keyboard::Key::Named(keyboard::key::Named::Alt),
                    keyboard::Modifiers::empty(),
                ),
            }
        })
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    // ==================== Helpers ====================

    fn active_panel_state(&self) -> &PanelState {
        match self.active_panel {
            PanelSide::Left => &self.left_panel,
            PanelSide::Right => &self.right_panel,
        }
    }

    fn panel_mut(&mut self, side: PanelSide) -> &mut PanelState {
        match side {
            PanelSide::Left => &mut self.left_panel,
            PanelSide::Right => &mut self.right_panel,
        }
    }

    fn inactive_panel_state(&self) -> &PanelState {
        match self.active_panel {
            PanelSide::Left => &self.right_panel,
            PanelSide::Right => &self.left_panel,
        }
    }

    fn selected_paths(&self) -> Vec<VfsPath> {
        self.active_panel_state()
            .selected_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect()
    }

    fn navigate_to(&mut self, side: PanelSide, path: VfsPath) -> Task<Message> {
        let panel = self.panel_mut(side);
        panel.current_path = path.clone();
        panel.loading = true;
        let vfs = self.vfs.clone();
        Task::perform(
            async move { vfs.read_dir(&path).await.map_err(|e| e.to_string()) },
            move |result| Message::DirectoryLoaded(side, result),
        )
    }

    fn try_enter_archive(&mut self, side: PanelSide, entry: &VfsEntry) -> Task<Message> {
        if entry.is_file() {
            if let Some(archive_path) = entry.path.as_local_path() {
                let p = archive_path.to_string_lossy();
                let is_archive = p.ends_with(".zip")
                    || p.ends_with(".jar")
                    || p.ends_with(".tar")
                    || p.ends_with(".tar.gz")
                    || p.ends_with(".tgz");
                if is_archive {
                    let scheme = if p.ends_with(".zip") || p.ends_with(".jar") {
                        "zip"
                    } else {
                        "tar"
                    };
                    let vfs_path = VfsPath {
                        scheme: scheme.into(),
                        authority: Some(p.to_string()),
                        path: "/".into(),
                    };
                    return self.navigate_to(side, vfs_path);
                }
            }
        }
        Task::none()
    }

    fn refresh_panel(&mut self, side: PanelSide) -> Task<Message> {
        let path = self.panel_mut(side).current_path.clone();
        self.navigate_to(side, path)
    }

    fn refresh_both_panels(&mut self) -> Task<Message> {
        let left_path = self.left_panel.current_path.clone();
        let right_path = self.right_panel.current_path.clone();
        let t1 = self.navigate_to(PanelSide::Left, left_path);
        let t2 = self.navigate_to(PanelSide::Right, right_path);
        Task::batch([t1, t2])
    }

    // ==================== File Operations ====================

    fn initiate_copy(&mut self) -> Task<Message> {
        let sources = self.selected_paths();
        if sources.is_empty() {
            return Task::none();
        }
        let dest = self.inactive_panel_state().current_path.clone();
        let op = OperationKind::Copy {
            sources: sources.clone(),
            destination: dest.clone(),
        };
        self.pending_operation = Some(op.clone());
        self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
            title: "Copy".into(),
            message: format!("Copy {} item(s) to {}?", sources.len(), dest),
            on_confirm: Box::new(Message::StartOperation(op)),
        }));
        Task::none()
    }

    fn initiate_move(&mut self) -> Task<Message> {
        let sources = self.selected_paths();
        if sources.is_empty() {
            return Task::none();
        }
        let dest = self.inactive_panel_state().current_path.clone();
        let op = OperationKind::Move {
            sources: sources.clone(),
            destination: dest.clone(),
        };
        self.pending_operation = Some(op.clone());
        self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
            title: "Move".into(),
            message: format!("Move {} item(s) to {}?", sources.len(), dest),
            on_confirm: Box::new(Message::StartOperation(op)),
        }));
        Task::none()
    }

    fn initiate_delete(&mut self) -> Task<Message> {
        let targets = self.selected_paths();
        if targets.is_empty() {
            return Task::none();
        }
        let op = OperationKind::Delete {
            targets: targets.clone(),
        };
        self.pending_operation = Some(op.clone());
        self.dialog = Some(DialogKind::Confirm(ConfirmDialog {
            title: "Delete".into(),
            message: format!("Delete {} item(s)?", targets.len()),
            on_confirm: Box::new(Message::StartOperation(op)),
        }));
        Task::none()
    }

    fn initiate_mkdir(&mut self) -> Task<Message> {
        self.dialog = Some(DialogKind::Input(InputDialog {
            title: "Create directory".into(),
            label: "Directory name:".into(),
            value: String::new(),
            on_submit: |_| Message::DialogResult(DialogMessage::InputSubmit),
        }));
        operation::focus(input::INPUT_DIALOG_ID)
    }

    fn initiate_rename(&mut self) -> Task<Message> {
        let panel = self.active_panel_state();
        if let Some(entry) = panel.current_entry() {
            self.dialog = Some(DialogKind::Input(InputDialog {
                title: "Rename".into(),
                label: "New name:".into(),
                value: entry.name.clone(),
                on_submit: |_| Message::DialogResult(DialogMessage::InputSubmit),
            }));
            return operation::focus(input::INPUT_DIALOG_ID);
        }
        Task::none()
    }

    fn initiate_chmod(&mut self) -> Task<Message> {
        let panel = self.active_panel_state();
        if let Some(entry) = panel.current_entry() {
            let mode = entry.permissions.unwrap_or(0o644);
            self.dialog = Some(DialogKind::Chmod(ChmodDialog::new(
                entry.path.clone(),
                entry.name.clone(),
                mode,
            )));
        }
        Task::none()
    }

    fn compare_directories(&mut self) {
        use std::collections::HashMap;

        // If already highlighted, clear (toggle off)
        if !self.left_panel.highlighted.is_empty() || !self.right_panel.highlighted.is_empty() {
            self.left_panel.highlighted.clear();
            self.right_panel.highlighted.clear();
            return;
        }

        // Build name -> (size, modified) maps for each panel
        let left_map: HashMap<&str, (u64, Option<std::time::SystemTime>)> = self
            .left_panel
            .entries
            .iter()
            .filter(|e| e.name != "..")
            .map(|e| (e.name.as_str(), (e.size, e.modified)))
            .collect();

        let right_map: HashMap<&str, (u64, Option<std::time::SystemTime>)> = self
            .right_panel
            .entries
            .iter()
            .filter(|e| e.name != "..")
            .map(|e| (e.name.as_str(), (e.size, e.modified)))
            .collect();

        // Highlight entries that differ
        let mut left_hl = std::collections::HashSet::new();
        for (i, entry) in self.left_panel.entries.iter().enumerate() {
            if entry.name == ".." {
                continue;
            }
            match right_map.get(entry.name.as_str()) {
                None => {
                    left_hl.insert(i);
                }
                Some(&(size, modified)) => {
                    if entry.size != size || entry.modified != modified {
                        left_hl.insert(i);
                    }
                }
            }
        }

        let mut right_hl = std::collections::HashSet::new();
        for (i, entry) in self.right_panel.entries.iter().enumerate() {
            if entry.name == ".." {
                continue;
            }
            match left_map.get(entry.name.as_str()) {
                None => {
                    right_hl.insert(i);
                }
                Some(&(size, modified)) => {
                    if entry.size != size || entry.modified != modified {
                        right_hl.insert(i);
                    }
                }
            }
        }

        self.left_panel.highlighted = left_hl;
        self.right_panel.highlighted = right_hl;
    }

    fn start_operation(&mut self, op: OperationKind) -> Task<Message> {
        let (tx, _rx) = mpsc::unbounded_channel();
        let vfs = self.vfs.clone();

        self.dialog = Some(DialogKind::Progress(ProgressDialog {
            title: "Starting operation...".into(),
            current_file: String::new(),
            total_bytes: 0,
            transferred_bytes: 0,
            files_done: 0,
            files_total: 0,
        }));

        Task::perform(
            async move { execute_operation(vfs, op, tx).await },
            Message::OperationComplete,
        )
    }

    fn handle_dialog(&mut self, msg: DialogMessage) -> Task<Message> {
        match msg {
            DialogMessage::Confirm(true) => {
                if let Some(op) = self.pending_operation.take() {
                    self.dialog = None;
                    return self.start_operation(op);
                }
                self.dialog = None;
            }
            DialogMessage::Cancel => {
                self.dialog = None;
                self.pending_operation = None;
            }
            DialogMessage::InputChanged(value) => {
                if let Some(DialogKind::Input(ref mut input)) = self.dialog {
                    input.value = value;
                }
            }
            DialogMessage::InputSubmit => {
                if let Some(DialogKind::Input(ref input)) = self.dialog {
                    let value = input.value.clone();
                    let title = input.title.clone();
                    self.dialog = None;

                    if title == "Create directory" {
                        let path = self.active_panel_state().current_path.join(&value);
                        let vfs = self.vfs.clone();
                        return Task::perform(
                            async move { vfs.create_dir(&path).await.map_err(|e| e.to_string()) },
                            Message::OperationComplete,
                        );
                    } else if title == "Rename" {
                        let panel = self.active_panel_state();
                        if let Some(entry) = panel.current_entry() {
                            let from = entry.path.clone();
                            let to = panel.current_path.join(&value);
                            let vfs = self.vfs.clone();
                            return Task::perform(
                                async move { vfs.rename(&from, &to).await.map_err(|e| e.to_string()) },
                                Message::OperationComplete,
                            );
                        }
                    } else if title == "Filter" {
                        if let Some(side) = self.pending_filter_side.take() {
                            self.panel_mut(side).filter = value;
                            return self.refresh_panel(side);
                        }
                    }
                }
            }
            DialogMessage::ChmodToggleBit(bit) => {
                if let Some(DialogKind::Chmod(ref mut d)) = self.dialog {
                    d.mode ^= bit;
                    d.octal_input = format!("{:04o}", d.mode & 0o7777);
                }
            }
            DialogMessage::ChmodOctalChanged(value) => {
                if let Some(DialogKind::Chmod(ref mut d)) = self.dialog {
                    d.octal_input = value.clone();
                    if let Ok(mode) = u32::from_str_radix(&value, 8) {
                        d.mode = mode & 0o7777;
                    }
                }
            }
            DialogMessage::ChmodApply => {
                if let Some(DialogKind::Chmod(ref d)) = self.dialog {
                    let path = d.path.clone();
                    let mode = d.mode;
                    let vfs = self.vfs.clone();
                    self.dialog = None;
                    return Task::perform(
                        async move {
                            vfs.set_permissions(&path, mode)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::OperationComplete,
                    );
                }
            }
            _ => {}
        }
        Task::none()
    }

    // ==================== Viewer ====================

    fn open_viewer(&mut self) -> Task<Message> {
        let panel = self.active_panel_state();
        if let Some(entry) = panel.current_entry() {
            if entry.is_file() {
                let name = entry.name.clone();
                let path = entry.path.clone();
                let vfs = self.vfs.clone();
                return Task::perform(
                    async move {
                        let mut reader = vfs.open_read(&path).await.map_err(|e| e.to_string())?;
                        let mut buf = Vec::new();
                        reader
                            .read_to_end(&mut buf)
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok(buf)
                    },
                    move |result| Message::FileLoaded(name.clone(), result),
                );
            }
        }
        Task::none()
    }

    fn handle_viewer(&mut self, msg: ViewerMessage) -> Task<Message> {
        let viewer = match self.viewer.as_mut() {
            Some(v) => v,
            None => return Task::none(),
        };

        match msg {
            ViewerMessage::Close => {
                self.viewer = None;
            }
            ViewerMessage::SwitchMode(mode) => {
                viewer.mode = mode;
                viewer.offset = 0;
            }
            ViewerMessage::ScrollUp => viewer.scroll_up(1),
            ViewerMessage::ScrollDown => viewer.scroll_down(1),
            ViewerMessage::PageUp => viewer.scroll_up(viewer.lines_per_page),
            ViewerMessage::PageDown => viewer.scroll_down(viewer.lines_per_page),
            ViewerMessage::GoTop => viewer.offset = 0,
            ViewerMessage::GoBottom => {
                viewer.offset = viewer.total_lines().saturating_sub(viewer.lines_per_page);
            }
            ViewerMessage::SearchOpen => {
                viewer.search_active = true;
            }
            ViewerMessage::SearchChanged(query) => {
                viewer.search_query = query;
            }
            ViewerMessage::SearchNext => {
                // Simple text search in viewer
                if viewer.mode == ViewMode::Text && !viewer.search_query.is_empty() {
                    let text_content = String::from_utf8_lossy(&viewer.content);
                    let lines: Vec<&str> = text_content.lines().collect();
                    let query = &viewer.search_query;
                    for (i, line) in lines.iter().enumerate().skip(viewer.offset + 1) {
                        if line.contains(query.as_str()) {
                            viewer.offset = i;
                            break;
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn handle_viewer_key(
        &mut self,
        key: keyboard::Key,
        _modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        match key {
            keyboard::Key::Named(named) => match named {
                keyboard::key::Named::Escape | keyboard::key::Named::F3 => {
                    return self.update(Message::Viewer(ViewerMessage::Close));
                }
                keyboard::key::Named::ArrowUp => {
                    return self.update(Message::Viewer(ViewerMessage::ScrollUp));
                }
                keyboard::key::Named::ArrowDown => {
                    return self.update(Message::Viewer(ViewerMessage::ScrollDown));
                }
                keyboard::key::Named::PageUp => {
                    return self.update(Message::Viewer(ViewerMessage::PageUp));
                }
                keyboard::key::Named::PageDown => {
                    return self.update(Message::Viewer(ViewerMessage::PageDown));
                }
                keyboard::key::Named::Home => {
                    return self.update(Message::Viewer(ViewerMessage::GoTop));
                }
                keyboard::key::Named::End => {
                    return self.update(Message::Viewer(ViewerMessage::GoBottom));
                }
                keyboard::key::Named::F4 => {
                    let new_mode = if self.viewer.as_ref().map(|v| v.mode) == Some(ViewMode::Text) {
                        ViewMode::Hex
                    } else {
                        ViewMode::Text
                    };
                    return self.update(Message::Viewer(ViewerMessage::SwitchMode(new_mode)));
                }
                _ => {}
            },
            _ => {}
        }
        Task::none()
    }

    // ==================== Editor ====================

    fn open_editor(&mut self) -> Task<Message> {
        let panel = self.active_panel_state();
        if let Some(entry) = panel.current_entry() {
            if entry.is_file() {
                let name = entry.name.clone();
                let file_path = entry.path.clone();
                let vfs = self.vfs.clone();
                let fp = file_path.clone();
                return Task::perform(
                    async move {
                        let mut reader = vfs.open_read(&fp).await.map_err(|e| e.to_string())?;
                        let mut buf = Vec::new();
                        reader
                            .read_to_end(&mut buf)
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok(buf)
                    },
                    move |result| {
                        Message::FileLoadedForEdit(name.clone(), file_path.clone(), result)
                    },
                );
            }
        }
        Task::none()
    }

    fn handle_editor(&mut self, msg: EditorMessage) -> Task<Message> {
        let editor = match self.editor.as_mut() {
            Some(e) => e,
            None => return Task::none(),
        };

        match msg {
            EditorMessage::ActionPerformed(action) => {
                let is_edit = action.is_edit();
                editor.content.perform(action);
                if is_edit {
                    editor.dirty = true;
                    editor.status_message = None;
                }
            }
            EditorMessage::Save => {
                let text = editor.content.text();
                let path = editor.file_path.clone();
                let vfs = self.vfs.clone();
                return Task::perform(
                    async move {
                        let mut writer = vfs.open_write(&path).await.map_err(|e| e.to_string())?;
                        writer
                            .write_all(text.as_bytes())
                            .await
                            .map_err(|e| e.to_string())?;
                        writer.flush().await.map_err(|e| e.to_string())?;
                        Ok(())
                    },
                    Message::FileSaved,
                );
            }
            EditorMessage::Close => {
                // TODO: prompt save if dirty
                self.editor = None;
                return self.refresh_both_panels();
            }
            EditorMessage::Find => {
                if let Some(ref mut e) = self.editor {
                    e.find_active = !e.find_active;
                }
            }
            EditorMessage::FindChanged(query) => {
                if let Some(ref mut e) = self.editor {
                    e.find_query = query;
                }
            }
            EditorMessage::FindNext => {
                // TODO: search in editor content
            }
        }
        Task::none()
    }

    fn handle_editor_key(
        &mut self,
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
    ) -> Task<Message> {
        match key {
            keyboard::Key::Named(named) => match named {
                keyboard::key::Named::F2 => {
                    return self.update(Message::Editor(EditorMessage::Save));
                }
                keyboard::key::Named::F10 | keyboard::key::Named::Escape => {
                    return self.update(Message::Editor(EditorMessage::Close));
                }
                keyboard::key::Named::F7 => {
                    return self.update(Message::Editor(EditorMessage::Find));
                }
                _ => {}
            },
            keyboard::Key::Character(ref c) => {
                if modifiers.command() && c.as_str() == "s" {
                    return self.update(Message::Editor(EditorMessage::Save));
                }
            }
            _ => {}
        }
        Task::none()
    }

    // ==================== Search ====================

    fn handle_search(&mut self, msg: SearchMessage) -> Task<Message> {
        match msg {
            SearchMessage::DirectoryChanged(dir) => {
                if let Some(ref mut s) = self.search {
                    s.directory = dir;
                }
            }
            SearchMessage::PatternChanged(pattern) => {
                if let Some(ref mut s) = self.search {
                    s.pattern = pattern;
                }
            }
            SearchMessage::ContentChanged(content) => {
                if let Some(ref mut s) = self.search {
                    s.content_pattern = content;
                }
            }
            SearchMessage::Start => {
                if let Some(ref mut s) = self.search {
                    s.searching = true;
                    s.results.clear();
                    let dir = VfsPath::local(&s.directory);
                    let pattern = s.pattern.clone();
                    let content = s.content_pattern.clone();
                    let vfs = self.vfs.clone();

                    let (tx, mut rx) = mpsc::unbounded_channel();

                    // Spawn the search task
                    tokio::spawn(async move {
                        let _ = search_files(vfs, dir, pattern, content, tx).await;
                    });

                    // We'll collect results via polling -- for now just mark as searching
                    // In a real implementation we'd use a subscription
                }
            }
            SearchMessage::ResultFound(path) => {
                if let Some(ref mut s) = self.search {
                    s.results.push(path);
                }
            }
            SearchMessage::Complete => {
                if let Some(ref mut s) = self.search {
                    s.searching = false;
                }
            }
            SearchMessage::GoToResult(path) => {
                self.search = None;
                if let Some(parent) = path.parent() {
                    let side = self.active_panel;
                    return self.navigate_to(side, parent);
                }
            }
            SearchMessage::Close => {
                self.search = None;
            }
        }
        Task::none()
    }

    // ==================== Bookmarks ====================

    fn handle_bookmark(&mut self, msg: BookmarkMessage) -> Task<Message> {
        match msg {
            BookmarkMessage::Add => {
                let path = self.active_panel_state().current_path.clone();
                let name = path.file_name().unwrap_or("bookmark").to_string();
                self.bookmarks.add(Bookmark::from_vfs_path(name, &path));
                save_bookmarks(&self.bookmarks.bookmarks);
            }
            BookmarkMessage::Remove(idx) => {
                self.bookmarks.remove(idx);
                save_bookmarks(&self.bookmarks.bookmarks);
            }
            BookmarkMessage::GoTo(idx) => {
                if let Some(bookmark) = self.bookmarks.bookmarks.get(idx) {
                    let path = bookmark.to_vfs_path();
                    let side = self.active_panel;
                    self.bookmarks.visible = false;
                    return self.navigate_to(side, path);
                }
            }
            BookmarkMessage::Open => {
                self.bookmarks.visible = true;
            }
            BookmarkMessage::Close => {
                self.bookmarks.visible = false;
            }
        }
        Task::none()
    }

    // ==================== Panel navigation ====================

    fn handle_panel_message(&mut self, side: PanelSide, msg: PanelMessage) -> Task<Message> {
        match msg {
            PanelMessage::Navigate(path) => {
                return self.navigate_to(side, path);
            }
            PanelMessage::GoUp => {
                let panel = self.panel_mut(side);
                let target = panel.current_path.parent()
                    .or_else(|| panel.current_path.exit_parent());
                if let Some(parent) = target {
                    return self.navigate_to(side, parent);
                }
            }
            PanelMessage::Enter => {
                let panel = self.panel_mut(side);
                if let Some(entry) = panel.current_entry().cloned() {
                    if entry.is_dir() {
                        return self.navigate_to(side, entry.path);
                    }
                    return self.try_enter_archive(side, &entry);
                }
            }
            PanelMessage::Select(index) => {
                self.active_panel = side;
                let panel = self.panel_mut(side);
                if panel.cursor == index {
                    if let Some(entry) = panel.entries.get(index).cloned() {
                        if entry.is_dir() {
                            return self.navigate_to(side, entry.path);
                        }
                        return self.try_enter_archive(side, &entry);
                    }
                } else {
                    panel.cursor = index;
                }
            }
            PanelMessage::ToggleSelect(index) => {
                self.panel_mut(side).toggle_select(index);
            }
            PanelMessage::CursorMove(delta) => {
                self.panel_mut(side).move_cursor(delta);
            }
            PanelMessage::CursorPage(delta) => {
                self.panel_mut(side).move_cursor_page(delta);
            }
            PanelMessage::CursorHome => {
                self.panel_mut(side).cursor_home();
            }
            PanelMessage::CursorEnd => {
                self.panel_mut(side).cursor_end();
            }
            PanelMessage::Sort(mode) => {
                let panel = self.panel_mut(side);
                if panel.sort_mode == mode {
                    panel.sort_ascending = !panel.sort_ascending;
                } else {
                    panel.sort_mode = mode;
                    panel.sort_ascending = true;
                }
                panel.resort();
            }
            PanelMessage::Refresh => {
                return self.refresh_panel(side);
            }
            PanelMessage::PathBarClicked => {
                let panel = self.panel_mut(side);
                panel.path_input_value = panel.current_path.to_string();
                panel.path_editing = true;
            }
            PanelMessage::PathInputChanged(s) => {
                self.panel_mut(side).path_input_value = s;
            }
            PanelMessage::PathInputSubmit => {
                let panel = self.panel_mut(side);
                let path = VfsPath::parse(&panel.path_input_value);
                panel.path_editing = false;
                return self.navigate_to(side, path);
            }
            PanelMessage::PathInputCancel => {
                self.panel_mut(side).path_editing = false;
            }
        }
        Task::none()
    }

    // ==================== Keyboard ====================

    fn handle_key(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers) -> Task<Message> {
        let side = self.active_panel;

        // When path bar is being edited, only handle Escape to cancel
        if self.active_panel_state().path_editing {
            if let keyboard::Key::Named(keyboard::key::Named::Escape) = key {
                return self.update(Message::Panel(side, PanelMessage::PathInputCancel));
            }
            return Task::none();
        }

        match key {
            keyboard::Key::Named(named) => match named {
                keyboard::key::Named::ArrowUp => {
                    if modifiers.shift() {
                        let cursor = self.active_panel_state().cursor;
                        self.panel_mut(side).toggle_select(cursor);
                    }
                    return self.update(Message::Panel(side, PanelMessage::CursorMove(-1)));
                }
                keyboard::key::Named::ArrowDown => {
                    if modifiers.shift() {
                        let cursor = self.active_panel_state().cursor;
                        self.panel_mut(side).toggle_select(cursor);
                    }
                    return self.update(Message::Panel(side, PanelMessage::CursorMove(1)));
                }
                keyboard::key::Named::PageUp => {
                    return self.update(Message::Panel(side, PanelMessage::CursorPage(-1)));
                }
                keyboard::key::Named::PageDown => {
                    return self.update(Message::Panel(side, PanelMessage::CursorPage(1)));
                }
                keyboard::key::Named::Home => {
                    return self.update(Message::Panel(side, PanelMessage::CursorHome));
                }
                keyboard::key::Named::End => {
                    return self.update(Message::Panel(side, PanelMessage::CursorEnd));
                }
                keyboard::key::Named::Enter => {
                    return self.update(Message::Panel(side, PanelMessage::Enter));
                }
                keyboard::key::Named::Backspace => {
                    return self.update(Message::Panel(side, PanelMessage::GoUp));
                }
                keyboard::key::Named::Tab => {
                    return self.update(Message::SwitchPanel);
                }
                keyboard::key::Named::Insert => {
                    let cursor = self.active_panel_state().cursor;
                    self.panel_mut(side).toggle_select(cursor);
                    self.panel_mut(side).move_cursor(1);
                }
                keyboard::key::Named::F3 => return self.update(Message::ViewFile),
                keyboard::key::Named::F4 => return self.update(Message::EditFile),
                keyboard::key::Named::F5 => return self.update(Message::CopySelected),
                keyboard::key::Named::F6 => {
                    if modifiers.shift() {
                        return self.update(Message::Rename);
                    }
                    return self.update(Message::MoveSelected);
                }
                keyboard::key::Named::F7 => return self.update(Message::Mkdir),
                keyboard::key::Named::F8 => return self.update(Message::DeleteSelected),
                keyboard::key::Named::F9 => {
                    return self.update(Message::MenuOpen(MenuId::Left));
                }
                keyboard::key::Named::F10 => return self.update(Message::Quit),
                _ => {}
            },
            keyboard::Key::Character(ref c) => {
                if modifiers.command() {
                    match c.as_str() {
                        "r" => return self.update(Message::Panel(side, PanelMessage::Refresh)),
                        "f" => return self.update(Message::OpenSearch),
                        "h" => return self.update(Message::ToggleHidden),
                        "d" => return self.update(Message::Bookmark(BookmarkMessage::Add)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Task::none()
    }
}

/// Simple glob matching: supports `*` as wildcard for any characters.
/// e.g. `*.rs` matches `main.rs`, `foo*bar` matches `fooXYZbar`.
fn glob_match(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let name = name.to_lowercase();
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        // No wildcard: substring match
        return name.contains(&pattern);
    }
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if let Some(found) = name[pos..].find(part) {
            if i == 0 && found != 0 {
                return false; // must match from start if pattern doesn't start with *
            }
            pos += found + part.len();
        } else {
            return false;
        }
    }
    // If pattern doesn't end with *, the name must end at pos
    if !pattern.ends_with('*') {
        return pos == name.len();
    }
    true
}
