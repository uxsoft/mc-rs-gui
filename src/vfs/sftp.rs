use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;

use super::{EntryType, VfsEntry, VfsError, VfsPath, VfsProvider};

/// SFTP VFS provider placeholder.
///
/// The full implementation requires russh + russh-sftp with careful version
/// coordination. This provides the VfsProvider interface so the rest of the
/// app compiles and can be connected later.
pub struct SftpVfsProvider {
    connected: Arc<Mutex<bool>>,
}

impl SftpVfsProvider {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn connect(
        &self,
        _host: &str,
        _port: u16,
        _user: &str,
        _password: &str,
    ) -> Result<(), VfsError> {
        // TODO: Implement with russh + russh-sftp
        Err(VfsError::Other("SFTP not yet implemented".into()))
    }
}

#[async_trait::async_trait]
impl VfsProvider for SftpVfsProvider {
    fn scheme(&self) -> &str {
        "sftp"
    }

    fn handles(&self, path: &VfsPath) -> bool {
        path.scheme == "sftp"
    }

    async fn read_dir(&self, _path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn stat(&self, path: &VfsPath) -> Result<VfsEntry, VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn open_read(
        &self,
        _path: &VfsPath,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>, VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn open_write(
        &self,
        _path: &VfsPath,
    ) -> Result<Box<dyn AsyncWrite + Unpin + Send>, VfsError> {
        Err(VfsError::Unsupported)
    }

    async fn create_dir(&self, _path: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn remove_file(&self, _path: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn remove_dir(&self, _path: &VfsPath, _recursive: bool) -> Result<(), VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }

    async fn rename(&self, _from: &VfsPath, _to: &VfsPath) -> Result<(), VfsError> {
        Err(VfsError::Connection("SFTP: not connected".into()))
    }
}
