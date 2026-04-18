pub mod archive;
pub mod ftp;
pub mod local;
pub mod sftp;

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::io::{AsyncRead, AsyncWrite};

/// A location within a virtual filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VfsPath {
    /// Provider scheme: "file", "tar", "zip", "ftp", "sftp"
    pub scheme: String,
    /// Provider-specific authority (host:port for remote, archive path for archives)
    pub authority: Option<String>,
    /// Path within the provider's namespace
    pub path: PathBuf,
}

impl VfsPath {
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self {
            scheme: "file".into(),
            authority: None,
            path: path.into(),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(|p| Self {
            scheme: self.scheme.clone(),
            authority: self.authority.clone(),
            path: p.to_path_buf(),
        })
    }

    pub fn join(&self, name: &str) -> Self {
        Self {
            scheme: self.scheme.clone(),
            authority: self.authority.clone(),
            path: self.path.join(name),
        }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    /// Parse a string into a VfsPath.
    /// Supports plain paths like `/home/user` and URI-style like `ftp://host/path`.
    pub fn parse(s: &str) -> Self {
        if let Some(rest) = s.strip_prefix("file://") {
            VfsPath::local(rest)
        } else if let Some(pos) = s.find("://") {
            let scheme = &s[..pos];
            let after_scheme = &s[pos + 3..];
            if let Some(slash_pos) = after_scheme.find('/') {
                let authority = &after_scheme[..slash_pos];
                let path = &after_scheme[slash_pos..];
                VfsPath {
                    scheme: scheme.to_string(),
                    authority: Some(authority.to_string()),
                    path: PathBuf::from(path),
                }
            } else {
                VfsPath {
                    scheme: scheme.to_string(),
                    authority: Some(after_scheme.to_string()),
                    path: PathBuf::from("/"),
                }
            }
        } else {
            VfsPath::local(s)
        }
    }

    pub fn is_local(&self) -> bool {
        self.scheme == "file"
    }

    pub fn as_local_path(&self) -> Option<&Path> {
        if self.is_local() {
            Some(&self.path)
        } else {
            None
        }
    }
}

impl fmt::Display for VfsPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_local() {
            write!(f, "{}", self.path.display())
        } else if let Some(auth) = &self.authority {
            write!(f, "{}://{}{}", self.scheme, auth, self.path.display())
        } else {
            write!(f, "{}://{}", self.scheme, self.path.display())
        }
    }
}

#[derive(Debug, Clone)]
pub struct VfsEntry {
    pub name: String,
    pub path: VfsPath,
    pub entry_type: EntryType,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: Option<u32>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub link_target: Option<VfsPath>,
}

impl VfsEntry {
    pub fn is_dir(&self) -> bool {
        self.entry_type == EntryType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.entry_type == EntryType::File
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.path.extension().and_then(|e| e.to_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
    Symlink,
    Special,
}

#[derive(Debug, thiserror::Error)]
pub enum VfsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("not supported by this provider")]
    Unsupported,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("{0}")]
    Other(String),
}

/// Core VFS provider trait. All filesystem backends implement this.
#[async_trait::async_trait]
pub trait VfsProvider: Send + Sync {
    fn scheme(&self) -> &str;

    async fn read_dir(&self, path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError>;

    async fn stat(&self, path: &VfsPath) -> Result<VfsEntry, VfsError>;

    async fn open_read(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>, VfsError>;

    async fn open_write(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncWrite + Unpin + Send>, VfsError>;

    async fn create_dir(&self, path: &VfsPath) -> Result<(), VfsError>;

    async fn remove_file(&self, path: &VfsPath) -> Result<(), VfsError>;

    async fn remove_dir(&self, path: &VfsPath, recursive: bool) -> Result<(), VfsError>;

    async fn rename(&self, from: &VfsPath, to: &VfsPath) -> Result<(), VfsError>;

    async fn set_permissions(&self, _path: &VfsPath, _mode: u32) -> Result<(), VfsError> {
        Err(VfsError::Unsupported)
    }

    fn handles(&self, path: &VfsPath) -> bool;
}

/// Routes VFS operations to the correct provider.
pub struct VfsRouter {
    providers: Vec<Box<dyn VfsProvider>>,
}

impl VfsRouter {
    pub fn new(providers: Vec<Box<dyn VfsProvider>>) -> Self {
        Self { providers }
    }

    fn provider_for(&self, path: &VfsPath) -> Result<&dyn VfsProvider, VfsError> {
        self.providers
            .iter()
            .find(|p| p.handles(path))
            .map(|p| p.as_ref())
            .ok_or_else(|| VfsError::Other(format!("no provider for scheme: {}", path.scheme)))
    }

    pub async fn read_dir(&self, path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError> {
        self.provider_for(path)?.read_dir(path).await
    }

    pub async fn stat(&self, path: &VfsPath) -> Result<VfsEntry, VfsError> {
        self.provider_for(path)?.stat(path).await
    }

    pub async fn open_read(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>, VfsError> {
        self.provider_for(path)?.open_read(path).await
    }

    pub async fn open_write(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncWrite + Unpin + Send>, VfsError> {
        self.provider_for(path)?.open_write(path).await
    }

    pub async fn create_dir(&self, path: &VfsPath) -> Result<(), VfsError> {
        self.provider_for(path)?.create_dir(path).await
    }

    pub async fn remove_file(&self, path: &VfsPath) -> Result<(), VfsError> {
        self.provider_for(path)?.remove_file(path).await
    }

    pub async fn remove_dir(&self, path: &VfsPath, recursive: bool) -> Result<(), VfsError> {
        self.provider_for(path)?.remove_dir(path, recursive).await
    }

    pub async fn rename(&self, from: &VfsPath, to: &VfsPath) -> Result<(), VfsError> {
        self.provider_for(from)?.rename(from, to).await
    }

    pub async fn set_permissions(&self, path: &VfsPath, mode: u32) -> Result<(), VfsError> {
        self.provider_for(path)?.set_permissions(path, mode).await
    }
}
