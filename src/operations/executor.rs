use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::vfs::{VfsEntry, VfsError, VfsPath, VfsRouter};

use super::{OperationKind, OperationProgress};

pub async fn execute_operation(
    vfs: Arc<VfsRouter>,
    operation: OperationKind,
    progress_tx: mpsc::UnboundedSender<OperationProgress>,
) -> Result<(), String> {
    match operation {
        OperationKind::Copy {
            sources,
            destination,
        } => copy_files(vfs, sources, destination, progress_tx).await,
        OperationKind::Move {
            sources,
            destination,
        } => move_files(vfs, sources, destination, progress_tx).await,
        OperationKind::Delete { targets } => delete_files(vfs, targets, progress_tx).await,
        OperationKind::Mkdir { path } => vfs.create_dir(&path).await.map_err(|e| e.to_string()),
    }
}

async fn copy_files(
    vfs: Arc<VfsRouter>,
    sources: Vec<VfsPath>,
    destination: VfsPath,
    progress_tx: mpsc::UnboundedSender<OperationProgress>,
) -> Result<(), String> {
    // Calculate total size
    let mut total_bytes: u64 = 0;
    let mut file_list: Vec<(VfsPath, VfsPath)> = Vec::new();

    for source in &sources {
        collect_files(&vfs, source, &destination, &mut file_list, &mut total_bytes).await?;
    }

    let mut progress = OperationProgress::new(file_list.len(), total_bytes);
    let _ = progress_tx.send(progress.clone());

    for (src, dst) in &file_list {
        let entry = vfs.stat(src).await.map_err(|e| e.to_string())?;
        progress.current_file = entry.name.clone();
        let _ = progress_tx.send(progress.clone());

        if entry.is_dir() {
            vfs.create_dir(dst).await.map_err(|e| e.to_string())?;
        } else {
            copy_single_file(&vfs, src, dst, &mut progress, &progress_tx).await?;
        }

        progress.files_done += 1;
        let _ = progress_tx.send(progress.clone());
    }

    Ok(())
}

async fn copy_single_file(
    vfs: &VfsRouter,
    src: &VfsPath,
    dst: &VfsPath,
    progress: &mut OperationProgress,
    progress_tx: &mpsc::UnboundedSender<OperationProgress>,
) -> Result<(), String> {
    let mut reader = vfs.open_read(src).await.map_err(|e| e.to_string())?;
    let mut writer = vfs.open_write(dst).await.map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer
    loop {
        let n = reader.read(&mut buf).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .await
            .map_err(|e| e.to_string())?;
        progress.transferred_bytes += n as u64;
        let _ = progress_tx.send(progress.clone());
    }
    writer.flush().await.map_err(|e| e.to_string())?;

    Ok(())
}

fn collect_files<'a>(
    vfs: &'a VfsRouter,
    source: &'a VfsPath,
    dest_dir: &'a VfsPath,
    file_list: &'a mut Vec<(VfsPath, VfsPath)>,
    total_bytes: &'a mut u64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        let entry = vfs.stat(source).await.map_err(|e| e.to_string())?;
        let dest_path = dest_dir.join(&entry.name);

        if entry.is_dir() {
            file_list.push((source.clone(), dest_path.clone()));
            let children = vfs.read_dir(source).await.map_err(|e| e.to_string())?;
            for child in &children {
                collect_files(vfs, &child.path, &dest_path, file_list, total_bytes).await?;
            }
        } else {
            *total_bytes += entry.size;
            file_list.push((source.clone(), dest_path));
        }
        Ok(())
    })
}

async fn move_files(
    vfs: Arc<VfsRouter>,
    sources: Vec<VfsPath>,
    destination: VfsPath,
    progress_tx: mpsc::UnboundedSender<OperationProgress>,
) -> Result<(), String> {
    // Try rename first (same filesystem)
    for source in &sources {
        let entry = vfs.stat(source).await.map_err(|e| e.to_string())?;
        let dest_path = destination.join(&entry.name);
        match vfs.rename(source, &dest_path).await {
            Ok(()) => continue,
            Err(_) => {
                // Fall back to copy + delete
                copy_files(
                    vfs.clone(),
                    vec![source.clone()],
                    destination.clone(),
                    progress_tx.clone(),
                )
                .await?;
                delete_single(&vfs, source).await?;
            }
        }
    }
    Ok(())
}

async fn delete_files(
    vfs: Arc<VfsRouter>,
    targets: Vec<VfsPath>,
    progress_tx: mpsc::UnboundedSender<OperationProgress>,
) -> Result<(), String> {
    let mut progress = OperationProgress::new(targets.len(), 0);
    let _ = progress_tx.send(progress.clone());

    for target in &targets {
        let entry = vfs.stat(target).await.map_err(|e| e.to_string())?;
        progress.current_file = entry.name.clone();
        let _ = progress_tx.send(progress.clone());

        delete_single(&vfs, target).await?;

        progress.files_done += 1;
        let _ = progress_tx.send(progress.clone());
    }
    Ok(())
}

async fn delete_single(vfs: &VfsRouter, path: &VfsPath) -> Result<(), String> {
    let entry = vfs.stat(path).await.map_err(|e| e.to_string())?;
    if entry.is_dir() {
        vfs.remove_dir(path, true).await.map_err(|e| e.to_string())
    } else {
        vfs.remove_file(path).await.map_err(|e| e.to_string())
    }
}
