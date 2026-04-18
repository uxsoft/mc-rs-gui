use std::fs;
use std::path::PathBuf;

use super::Bookmark;

fn bookmarks_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mc-rs");
    config_dir.join("bookmarks.json")
}

pub fn load_bookmarks() -> Vec<Bookmark> {
    let path = bookmarks_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(bookmarks) = serde_json::from_str(&data) {
                return bookmarks;
            }
        }
    }
    Vec::new()
}

pub fn save_bookmarks(bookmarks: &[Bookmark]) {
    let path = bookmarks_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(bookmarks) {
        let _ = fs::write(&path, data);
    }
}
