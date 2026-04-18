use std::io::{Cursor, Read as StdRead};
use std::path::PathBuf;
use std::time::SystemTime;

use tokio::io::{AsyncRead, AsyncWrite};

use super::{EntryType, VfsEntry, VfsError, VfsPath, VfsProvider};

/// Archive VFS provider supporting .tar, .tar.gz, .tar.bz2, .zip
pub struct ArchiveVfsProvider;

impl ArchiveVfsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
struct ArchiveEntry {
    name: String,
    entry_type: EntryType,
    size: u64,
    modified: Option<SystemTime>,
}

/// Parse an archive and return a directory tree
fn read_zip_entries(archive_path: &str) -> Result<Vec<(String, ArchiveEntry)>, VfsError> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| VfsError::Other(format!("ZIP error: {e}")))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| VfsError::Other(format!("ZIP entry error: {e}")))?;

        let name = entry.name().to_string();
        let is_dir = entry.is_dir();
        let size = entry.size();

        entries.push((
            name.clone(),
            ArchiveEntry {
                name: PathBuf::from(&name)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(name),
                entry_type: if is_dir {
                    EntryType::Directory
                } else {
                    EntryType::File
                },
                size,
                modified: entry
                    .last_modified()
                    .and_then(|dt| time::OffsetDateTime::try_from(dt).ok())
                    .map(|t| {
                        SystemTime::UNIX_EPOCH
                            + std::time::Duration::from_secs(t.unix_timestamp().max(0) as u64)
                    }),
            },
        ));
    }
    Ok(entries)
}

fn read_tar_entries(archive_path: &str) -> Result<Vec<(String, ArchiveEntry)>, VfsError> {
    let file = std::fs::File::open(archive_path)?;

    // Detect compression
    let reader: Box<dyn StdRead> =
        if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
            Box::new(flate2::read::GzDecoder::new(file))
        } else {
            Box::new(file)
        };

    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();

    for entry in archive
        .entries()
        .map_err(|e| VfsError::Other(e.to_string()))?
    {
        let entry = entry.map_err(|e| VfsError::Other(e.to_string()))?;
        let path = entry.path().map_err(|e| VfsError::Other(e.to_string()))?;
        let path_str = path.to_string_lossy().to_string();

        let entry_type = match entry.header().entry_type() {
            tar::EntryType::Directory => EntryType::Directory,
            tar::EntryType::Symlink | tar::EntryType::Link => EntryType::Symlink,
            _ => EntryType::File,
        };

        let modified = entry
            .header()
            .mtime()
            .ok()
            .map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs));

        entries.push((
            path_str.clone(),
            ArchiveEntry {
                name: PathBuf::from(&path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(path_str),
                entry_type,
                size: entry.header().size().unwrap_or(0),
                modified,
            },
        ));
    }
    Ok(entries)
}

fn list_archive_dir(
    all_entries: &[(String, ArchiveEntry)],
    dir_path: &str,
    archive_path: &str,
    scheme: &str,
) -> Vec<VfsEntry> {
    let dir_prefix = if dir_path == "/" || dir_path.is_empty() {
        String::new()
    } else {
        let mut p = dir_path.trim_start_matches('/').to_string();
        if !p.ends_with('/') {
            p.push('/');
        }
        p
    };

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for (path, entry) in all_entries {
        let rel = if dir_prefix.is_empty() {
            path.as_str()
        } else if let Some(r) = path.strip_prefix(&dir_prefix) {
            r
        } else {
            continue;
        };

        // Skip self
        if rel.is_empty() || rel == "/" {
            continue;
        }

        // Only direct children (no more slashes, or just a trailing slash)
        let trimmed = rel.trim_end_matches('/');
        if trimmed.contains('/') {
            // This might be an implicit directory
            let first_component = trimmed.split('/').next().unwrap();
            if seen.insert(first_component.to_string()) {
                result.push(VfsEntry {
                    name: first_component.to_string(),
                    path: VfsPath {
                        scheme: scheme.into(),
                        authority: Some(archive_path.to_string()),
                        path: PathBuf::from(format!("{dir_prefix}{first_component}")),
                    },
                    entry_type: EntryType::Directory,
                    size: 0,
                    modified: None,
                    permissions: None,
                    owner: None,
                    group: None,
                    link_target: None,
                });
            }
            continue;
        }

        if seen.insert(trimmed.to_string()) {
            result.push(VfsEntry {
                name: entry.name.clone(),
                path: VfsPath {
                    scheme: scheme.into(),
                    authority: Some(archive_path.to_string()),
                    path: PathBuf::from(format!("{dir_prefix}{trimmed}")),
                },
                entry_type: entry.entry_type,
                size: entry.size,
                modified: entry.modified,
                permissions: None,
                owner: None,
                group: None,
                link_target: None,
            });
        }
    }

    result
}

fn extract_zip_file(archive_path: &str, file_path: &str) -> Result<Vec<u8>, VfsError> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| VfsError::Other(format!("ZIP error: {e}")))?;

    let mut entry = archive
        .by_name(file_path)
        .map_err(|e| VfsError::NotFound(format!("{file_path}: {e}")))?;

    let mut buf = Vec::new();
    entry
        .read_to_end(&mut buf)
        .map_err(|e| VfsError::Other(e.to_string()))?;
    Ok(buf)
}

fn extract_tar_file(archive_path: &str, file_path: &str) -> Result<Vec<u8>, VfsError> {
    let file = std::fs::File::open(archive_path)?;
    let reader: Box<dyn StdRead> =
        if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
            Box::new(flate2::read::GzDecoder::new(file))
        } else {
            Box::new(file)
        };

    let mut archive = tar::Archive::new(reader);
    for entry in archive
        .entries()
        .map_err(|e| VfsError::Other(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| VfsError::Other(e.to_string()))?;
        let path = entry.path().map_err(|e| VfsError::Other(e.to_string()))?;
        if path.to_string_lossy() == file_path {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| VfsError::Other(e.to_string()))?;
            return Ok(buf);
        }
    }
    Err(VfsError::NotFound(file_path.to_string()))
}

fn is_zip(path: &str) -> bool {
    path.ends_with(".zip") || path.ends_with(".jar") || path.ends_with(".war")
}

fn is_tar(path: &str) -> bool {
    path.ends_with(".tar")
        || path.ends_with(".tar.gz")
        || path.ends_with(".tgz")
        || path.ends_with(".tar.bz2")
}

fn archive_scheme(path: &str) -> &str {
    if is_zip(path) { "zip" } else { "tar" }
}

#[async_trait::async_trait]
impl VfsProvider for ArchiveVfsProvider {
    fn scheme(&self) -> &str {
        "archive"
    }

    fn handles(&self, path: &VfsPath) -> bool {
        matches!(path.scheme.as_str(), "zip" | "tar")
    }

    async fn read_dir(&self, path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError> {
        let archive_path = path
            .authority
            .as_ref()
            .ok_or_else(|| VfsError::Other("no archive path".into()))?
            .clone();
        let dir_path = path.path.to_string_lossy().to_string();
        let scheme = path.scheme.clone();

        tokio::task::spawn_blocking(move || {
            let all_entries = if is_zip(&archive_path) {
                read_zip_entries(&archive_path)?
            } else {
                read_tar_entries(&archive_path)?
            };
            Ok(list_archive_dir(
                &all_entries,
                &dir_path,
                &archive_path,
                &scheme,
            ))
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn stat(&self, path: &VfsPath) -> Result<VfsEntry, VfsError> {
        let name = path.file_name().unwrap_or("").to_string();
        Ok(VfsEntry {
            name,
            path: path.clone(),
            entry_type: EntryType::File,
            size: 0,
            modified: None,
            permissions: None,
            owner: None,
            group: None,
            link_target: None,
        })
    }

    async fn open_read(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>, VfsError> {
        let archive_path = path
            .authority
            .as_ref()
            .ok_or_else(|| VfsError::Other("no archive path".into()))?
            .clone();
        let file_path = path.path.to_string_lossy().to_string();

        let data = tokio::task::spawn_blocking(move || {
            if is_zip(&archive_path) {
                extract_zip_file(&archive_path, &file_path)
            } else {
                extract_tar_file(&archive_path, &file_path)
            }
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))??;

        Ok(Box::new(Cursor::new(data)))
    }

    async fn open_write(
        &self,
        _path: &VfsPath,
    ) -> Result<Box<dyn AsyncWrite + Unpin + Send>, VfsError> {
        Err(VfsError::Unsupported)
    }

    async fn create_dir(&self, _path: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }

    async fn remove_file(&self, _path: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }

    async fn remove_dir(&self, _path: &VfsPath, _recursive: bool) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }

    async fn rename(&self, _from: &VfsPath, _to: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }
}
