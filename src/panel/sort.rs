use crate::vfs::VfsEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Extension,
    Size,
    Modified,
}

pub fn sort_entries(entries: &mut Vec<VfsEntry>, mode: SortMode, ascending: bool) {
    // Directories always first
    entries.sort_by(|a, b| {
        let dir_ord = b.is_dir().cmp(&a.is_dir());
        if dir_ord != std::cmp::Ordering::Equal {
            return dir_ord;
        }

        let ord = match mode {
            SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortMode::Extension => {
                let a_ext = a.extension().unwrap_or("").to_lowercase();
                let b_ext = b.extension().unwrap_or("").to_lowercase();
                a_ext
                    .cmp(&b_ext)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            }
            SortMode::Size => a.size.cmp(&b.size),
            SortMode::Modified => a.modified.cmp(&b.modified),
        };

        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}
