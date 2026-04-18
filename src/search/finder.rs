use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use crate::vfs::{VfsPath, VfsRouter};

pub async fn search_files(
    vfs: Arc<VfsRouter>,
    directory: VfsPath,
    pattern: String,
    content_pattern: String,
    result_tx: mpsc::UnboundedSender<VfsPath>,
) -> Result<(), String> {
    search_recursive(&vfs, &directory, &pattern, &content_pattern, &result_tx).await
}

fn search_recursive<'a>(
    vfs: &'a VfsRouter,
    dir: &'a VfsPath,
    pattern: &'a str,
    content_pattern: &'a str,
    result_tx: &'a mpsc::UnboundedSender<VfsPath>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        let entries = match vfs.read_dir(dir).await {
            Ok(e) => e,
            Err(_) => return Ok(()), // Skip unreadable dirs
        };

        for entry in &entries {
            if entry.is_dir() {
                search_recursive(vfs, &entry.path, pattern, content_pattern, result_tx).await?;
            } else {
                if matches_glob(&entry.name, pattern) {
                    if content_pattern.is_empty() {
                        let _ = result_tx.send(entry.path.clone());
                    } else {
                        // Check file content
                        if let Ok(mut reader) = vfs.open_read(&entry.path).await {
                            let mut buf = vec![0u8; entry.size.min(1024 * 1024) as usize];
                            if let Ok(n) = reader.read(&mut buf).await {
                                let text = String::from_utf8_lossy(&buf[..n]);
                                if text.contains(content_pattern) {
                                    let _ = result_tx.send(entry.path.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    })
}

/// Simple glob matching (supports * and ?)
fn matches_glob(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let name = name.to_lowercase();
    let pattern = pattern.to_lowercase();

    glob_match(&name, &pattern)
}

fn glob_match(s: &str, p: &str) -> bool {
    let s: Vec<char> = s.chars().collect();
    let p: Vec<char> = p.chars().collect();
    let (mut si, mut pi) = (0, 0);
    let (mut star_pi, mut star_si) = (None::<usize>, 0);

    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            si += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star_pi = Some(pi);
            star_si = si;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_si += 1;
            si = star_si;
        } else {
            return false;
        }
    }

    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }

    pi == p.len()
}
