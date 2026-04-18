use std::fs;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};

use super::{EntryType, VfsEntry, VfsError, VfsPath, VfsProvider};

pub struct LocalVfsProvider;

impl LocalVfsProvider {
    pub fn new() -> Self {
        Self
    }

    fn to_local_path(path: &VfsPath) -> Result<&Path, VfsError> {
        path.as_local_path()
            .ok_or_else(|| VfsError::Other("not a local path".into()))
    }
}

fn metadata_to_entry(path: &Path, meta: &fs::Metadata) -> VfsEntry {
    let entry_type = if meta.is_dir() {
        EntryType::Directory
    } else if meta.is_symlink() {
        EntryType::Symlink
    } else if meta.is_file() {
        EntryType::File
    } else {
        EntryType::Special
    };

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    #[cfg(unix)]
    let (permissions, owner, group) = {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        (
            Some(meta.permissions().mode()),
            Some(meta.uid().to_string()),
            Some(meta.gid().to_string()),
        )
    };

    #[cfg(not(unix))]
    let (permissions, owner, group) = (None, None, None);

    let link_target = if meta.is_symlink() {
        fs::read_link(path).ok().map(VfsPath::local)
    } else {
        None
    };

    VfsEntry {
        name,
        path: VfsPath::local(path),
        entry_type,
        size: meta.len(),
        modified: meta.modified().ok(),
        permissions,
        owner,
        group,
        link_target,
    }
}

#[async_trait::async_trait]
impl VfsProvider for LocalVfsProvider {
    fn scheme(&self) -> &str {
        "file"
    }

    fn handles(&self, path: &VfsPath) -> bool {
        path.scheme == "file"
    }

    async fn read_dir(&self, path: &VfsPath) -> Result<Vec<VfsEntry>, VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let mut entries = Vec::new();
            for entry in fs::read_dir(&local)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                entries.push(metadata_to_entry(&entry.path(), &meta));
            }
            Ok(entries)
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn stat(&self, path: &VfsPath) -> Result<VfsEntry, VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let meta = fs::metadata(&local)?;
            Ok(metadata_to_entry(&local, &meta))
        })
        .await
        .map_err(|e| VfsError::Other(e.to_string()))?
    }

    async fn open_read(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>, VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        let file = tokio::fs::File::open(local).await?;
        Ok(Box::new(file))
    }

    async fn open_write(
        &self,
        path: &VfsPath,
    ) -> Result<Box<dyn AsyncWrite + Unpin + Send>, VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        let file = tokio::fs::File::create(local).await?;
        Ok(Box::new(file))
    }

    async fn create_dir(&self, path: &VfsPath) -> Result<(), VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        tokio::fs::create_dir_all(local).await?;
        Ok(())
    }

    async fn remove_file(&self, path: &VfsPath) -> Result<(), VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        tokio::fs::remove_file(local).await?;
        Ok(())
    }

    async fn remove_dir(&self, path: &VfsPath, recursive: bool) -> Result<(), VfsError> {
        let local = Self::to_local_path(path)?.to_path_buf();
        if recursive {
            tokio::fs::remove_dir_all(local).await?;
        } else {
            tokio::fs::remove_dir(local).await?;
        }
        Ok(())
    }

    async fn rename(&self, from: &VfsPath, to: &VfsPath) -> Result<(), VfsError> {
        let from_local = Self::to_local_path(from)?.to_path_buf();
        let to_local = Self::to_local_path(to)?.to_path_buf();
        tokio::fs::rename(from_local, to_local).await?;
        Ok(())
    }

    async fn set_permissions(&self, path: &VfsPath, mode: u32) -> Result<(), VfsError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let local = Self::to_local_path(path)?.to_path_buf();
            tokio::task::spawn_blocking(move || {
                let perms = fs::Permissions::from_mode(mode);
                fs::set_permissions(&local, perms)?;
                Ok(())
            })
            .await
            .map_err(|e| VfsError::Other(e.to_string()))?
        }
        #[cfg(not(unix))]
        {
            let _ = (path, mode);
            Err(VfsError::Unsupported)
        }
    }
}
