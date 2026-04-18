
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    CursorUp,
    CursorDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    GoUp,
    SwitchPanel,
    ToggleSelect,
    Copy,
    Move,
    Delete,
    Mkdir,
    Rename,
    View,
    Edit,
    Search,
    Refresh,
    Quit,
}
