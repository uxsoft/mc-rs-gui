# mc-rs

A cross-platform desktop file manager inspired by [GNU Midnight Commander](https://midnight-commander.org/), rewritten from scratch in Rust with the [Iced](https://iced.rs) GUI framework.

## Features

### Dual-Pane File Browser
- Side-by-side directory panels with active panel highlighting
- Keyboard-driven navigation (arrows, PgUp/PgDn, Home/End)
- Multi-file selection with Insert key
- Sortable columns (name, size, date, extension) -- click headers or use keyboard
- Quick directory traversal with Enter/Backspace
- Hidden file toggle (Ctrl+H)

### File Operations
- **Copy** (F5) -- async chunked copy with progress reporting
- **Move** (F6) -- rename when possible, falls back to copy+delete
- **Delete** (F8) -- recursive delete with confirmation dialog
- **Rename** (Shift+F6) -- inline rename via input dialog
- **Mkdir** (F7) -- create directories

### Built-in File Viewer (F3)
- Text mode with line numbers and scrolling
- Hex mode with address | hex bytes | ASCII columns
- Toggle between modes with F4
- In-viewer search

### Built-in Text Editor (F4)
- Full text editing powered by Iced's text editor widget
- Save (F2 / Ctrl+S), dirty-file tracking
- Find text (F7)

### Virtual Filesystem (VFS)
All filesystem backends implement a unified `VfsProvider` trait, so the panels, operations, viewer, and editor work transparently across:

| Backend | Status | Description |
|---------|--------|-------------|
| Local | Complete | Native filesystem via `tokio::fs` |
| ZIP | Complete | Read-only browsing and extraction of `.zip` / `.jar` archives |
| TAR | Complete | Read-only browsing of `.tar` / `.tar.gz` / `.tgz` archives |
| FTP | Complete | Synchronous FTP via `suppaftp` |
| SFTP | Stub | Interface ready, SSH transport to be wired |

Press Enter on an archive file to browse its contents as a virtual directory.

### Search (Ctrl+F)
- Recursive file search by name glob pattern
- Optional content matching (grep-like)
- Click results to navigate

### Bookmarks (Ctrl+D)
- Bookmark frequently visited directories
- Persisted to `~/.config/mc-rs/bookmarks.json`

### Configuration
- Settings persisted to `~/.config/mc-rs/config.json`
- Show/hide hidden files, confirm-on-delete, editor tab size

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Tab | Switch active panel |
| Enter | Open directory / enter archive |
| Backspace | Go to parent directory |
| Insert | Toggle file selection |
| F3 | View file |
| F4 | Edit file |
| F5 | Copy |
| F6 | Move |
| Shift+F6 | Rename |
| F7 | Create directory |
| F8 | Delete |
| F10 | Quit |
| Ctrl+R | Refresh panel |
| Ctrl+F | Search files |
| Ctrl+H | Toggle hidden files |
| Ctrl+D | Bookmark current directory |
| Ctrl+S | Save (in editor) |

## Building

### Prerequisites

- Rust 1.85+ (edition 2024)
- System dependencies (Linux):
  ```
  sudo apt install libwayland-dev libxkbcommon-dev libvulkan-dev
  ```

### Build and Run

```sh
cargo build --release
cargo run --release
```

### Run Tests

```sh
cargo test
```

## Project Structure

```
src/
  main.rs              Entry point
  app.rs               App state, Message enum, update/view logic
  vfs/                 Virtual Filesystem abstraction
    mod.rs             VfsProvider trait, VfsPath, VfsEntry, VfsRouter
    local.rs           Local filesystem provider
    archive.rs         ZIP and TAR archive provider
    ftp.rs             FTP provider
    sftp.rs            SFTP provider (stub)
  panel/               Dual-pane file list
    mod.rs             PanelState, panel_view()
    sort.rs            Sort modes and comparators
  operations/          Async file operations engine
    mod.rs             OperationKind, OperationProgress
    executor.rs        Copy, move, delete with progress
  viewer/              Built-in file viewer
    mod.rs             ViewerState, mode switching
    text_view.rs       Text mode rendering
    hex_view.rs        Hex dump rendering
  editor/              Built-in text editor
    mod.rs             EditorState, save/find commands
  search/              File search subsystem
    mod.rs             Search UI overlay
    finder.rs          Async recursive finder
  dialogs/             Modal dialog system
    mod.rs             Dialog overlay rendering
    confirm.rs         Yes/No confirmation
    input.rs           Text input dialog
    progress.rs        Operation progress bar
  menu/                Function key bar
    mod.rs             F1-F10 key bar view
  bookmarks/           Directory bookmarks
    mod.rs             BookmarkStore
    storage.rs         JSON persistence
  config/              Application settings
    mod.rs             AppConfig, load/save
    keymap.rs          Key binding definitions
  util/                Shared utilities
    human_size.rs      Byte size formatting
    time_fmt.rs        Date/time formatting
    icons.rs           File type icons
```

## CI/CD

- **CI** -- builds and tests on Linux, macOS, Windows for every pull request; runs clippy and format checks
- **Release** -- manual workflow that builds release binaries for Linux x64, Linux ARM64, macOS ARM64, and Windows x64, then creates a GitHub release with changelog and attached artifacts

## License

MIT
