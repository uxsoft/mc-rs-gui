use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

use super::{EntryType, VfsEntry, VfsError, VfsPath, VfsProvider};

/// FTP VFS provider using suppaftp
pub struct FtpVfsProvider {
    stream: Arc<Mutex<Option<suppaftp::FtpStream>>>,
}

impl FtpVfsProvider {
    pub fn new() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(
        &self,
        host: &str,
        port: u16,
        user: &str,
        password: &str,
    ) -> Result<(), VfsError> {
        let addr = format!("{host}:{port}");
        let host = host.to_string();
        let user = user.to_string();
        let password = password.to_string();

        let ftp = tokio::task::spawn_blocking(move || {
            let mut ftp = suppaftp::FtpStream::connect(&addr)
                .map_err(|e| VfsError::Connection(format!("FTP connection failed: {e}")))?;
            ftp.login(&user, &password)
                .map_err(|e| VfsError::Connection(format!("FTP login failed: {e}")))?;
            Ok::<_, VfsError>(ftp)
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))??;

        let mut guard = self.stream.lock().await;
        *guard = Some(ftp);
        Ok(())
    }
}

#[async_trait::async_trait]
impl VfsProvider for FtpVfsProvider {
    fn scheme(&self) -> &str {
        "ftp"
    }

    fn handles(&self, path: &VfsPath) -> bool {
        path.scheme == "ftp"
    }

    async fn read_dir(&self, path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError> {
        let stream = self.stream.clone();
        let dir_path = path.path.to_string_lossy().to_string();
        let authority = path.authority.clone();

        tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;

            let list = ftp
                .list(Some(&dir_path))
                .map_err(|e| VfsError::Other(format!("FTP list failed: {e}")))?;

            let mut entries = Vec::new();
            for line in &list {
                if let Some(entry) = parse_ftp_list_line(line, &dir_path, &authority) {
                    entries.push(entry);
                }
            }
            Ok(entries)
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
        let stream = self.stream.clone();
        let file_path = path.path.to_string_lossy().to_string();

        let data = tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;
            let cursor = ftp
                .retr_as_buffer(&file_path)
                .map_err(|e| VfsError::Other(format!("FTP retr failed: {e}")))?;
            Ok::<_, VfsError>(cursor.into_inner())
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

    async fn create_dir(&self, path: &VfsPath) -> Result<(), VfsError> {
        let stream = self.stream.clone();
        let dir_path = path.path.to_string_lossy().to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;
            ftp.mkdir(&dir_path)
                .map_err(|e| VfsError::Other(format!("FTP mkdir failed: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn remove_file(&self, path: &VfsPath) -> Result<(), VfsError> {
        let stream = self.stream.clone();
        let file_path = path.path.to_string_lossy().to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;
            ftp.rm(&file_path)
                .map_err(|e| VfsError::Other(format!("FTP rm failed: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn remove_dir(&self, path: &VfsPath, _recursive: bool) -> Result<(), VfsError> {
        let stream = self.stream.clone();
        let dir_path = path.path.to_string_lossy().to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;
            ftp.rmdir(&dir_path)
                .map_err(|e| VfsError::Other(format!("FTP rmdir failed: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn rename(&self, from: &VfsPath, to: &VfsPath) -> Result<(), VfsError> {
        let stream = self.stream.clone();
        let from_path = from.path.to_string_lossy().to_string();
        let to_path = to.path.to_string_lossy().to_string();
        tokio::task::spawn_blocking(move || {
            let mut guard = stream.blocking_lock();
            let ftp = guard
                .as_mut()
                .ok_or(VfsError::Connection("Not connected".into()))?;
            ftp.rename(&from_path, &to_path)
                .map_err(|e| VfsError::Other(format!("FTP rename failed: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }
}

/// Parse a single line from FTP LIST output (Unix-style)
fn parse_ftp_list_line(
    line: &str,
    parent_dir: &str,
    authority: &Option<String>,
) -> Option<VfsEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let perms = parts[0];
    let size: u64 = parts[4].parse().unwrap_or(0);
    let name = parts[8..].join(" ");

    if name == "." || name == ".." {
        return None;
    }

    let entry_type = if perms.starts_with('d') {
        EntryType::Directory
    } else if perms.starts_with('l') {
        EntryType::Symlink
    } else {
        EntryType::File
    };

    let full_path = if parent_dir == "/" {
        format!("/{name}")
    } else {
        format!("{parent_dir}/{name}")
    };

    Some(VfsEntry {
        name,
        path: VfsPath {
            scheme: "ftp".into(),
            authority: authority.clone(),
            path: PathBuf::from(&full_path),
        },
        entry_type,
        size,
        modified: None,
        permissions: None,
        owner: None,
        group: None,
        link_target: None,
    })
}
