pub mod storage;

use serde::{Deserialize, Serialize};

use crate::vfs::VfsPath;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: String,
    pub scheme: String,
    pub authority: Option<String>,
}

impl Bookmark {
    pub fn to_vfs_path(&self) -> VfsPath {
        VfsPath {
            scheme: self.scheme.clone(),
            authority: self.authority.clone(),
            path: self.path.clone().into(),
        }
    }

    pub fn from_vfs_path(name: String, path: &VfsPath) -> Self {
        Self {
            name,
            path: path.path.to_string_lossy().to_string(),
            scheme: path.scheme.clone(),
            authority: path.authority.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BookmarkMessage {
    Add,
    Remove(usize),
    GoTo(usize),
    Open,
    Close,
}

pub struct BookmarkStore {
    pub bookmarks: Vec<Bookmark>,
    pub visible: bool,
}

impl BookmarkStore {
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
            visible: false,
        }
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
        }
    }
}
