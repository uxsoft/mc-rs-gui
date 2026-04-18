pub mod executor;

use crate::vfs::VfsPath;

#[derive(Debug, Clone)]
pub enum OperationKind {
    Copy {
        sources: Vec<VfsPath>,
        destination: VfsPath,
    },
    Move {
        sources: Vec<VfsPath>,
        destination: VfsPath,
    },
    Delete {
        targets: Vec<VfsPath>,
    },
    Mkdir {
        path: VfsPath,
    },
}

#[derive(Debug, Clone)]
pub struct OperationProgress {
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub current_file: String,
    pub files_done: usize,
    pub files_total: usize,
}

impl OperationProgress {
    pub fn new(files_total: usize, total_bytes: u64) -> Self {
        Self {
            total_bytes,
            transferred_bytes: 0,
            current_file: String::new(),
            files_done: 0,
            files_total,
        }
    }
}
