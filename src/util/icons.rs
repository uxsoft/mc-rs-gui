use crate::vfs::EntryType;

pub fn icon_for_entry(entry_type: EntryType, name: &str) -> &'static str {
    match entry_type {
        EntryType::Directory => "\u{1F4C1}",
        EntryType::Symlink => "\u{1F517}",
        EntryType::Special => "\u{26A1}",
        EntryType::File => icon_for_file(name),
    }
}

fn icon_for_file(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_ascii_lowercase().as_str() {
        "rs" => "\u{1F980}",
        "py" => "\u{1F40D}",
        "js" | "ts" | "jsx" | "tsx" => "\u{1F4DC}",
        "html" | "htm" => "\u{1F310}",
        "css" | "scss" => "\u{1F3A8}",
        "json" | "toml" | "yaml" | "yml" | "xml" => "\u{2699}",
        "md" | "txt" | "doc" | "docx" => "\u{1F4DD}",
        "jpg" | "jpeg" | "png" | "gif" | "svg" | "bmp" | "webp" => "\u{1F5BC}",
        "mp3" | "wav" | "flac" | "ogg" => "\u{1F3B5}",
        "mp4" | "avi" | "mkv" | "mov" | "webm" => "\u{1F3AC}",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "\u{1F4E6}",
        "exe" | "bin" | "sh" | "bash" => "\u{2699}",
        "pdf" => "\u{1F4D5}",
        _ => "\u{1F4C4}",
    }
}
