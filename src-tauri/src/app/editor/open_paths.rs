//! 系统文件关联 / 双击打开：缓存路径并通知前端。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Url};

/// 前端监听：应用已运行时再次打开文件
const OPEN_FILES_EVENT: &str = "appx:open-files";

#[derive(Default)]
pub struct PendingOpenPaths(pub Mutex<Vec<String>>);

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// 从启动参数解析待打开文件（跳过 flags / 非 file URL）
pub fn paths_from_args(args: impl IntoIterator<Item = String>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for arg in args {
        let trimmed = arg.trim();
        if trimmed.is_empty() || trimmed.starts_with('-') {
            continue;
        }
        if let Ok(url) = Url::parse(trimmed) {
            if let Ok(path) = url.to_file_path() {
                paths.push(path);
            }
            continue;
        }
        paths.push(PathBuf::from(trimmed));
    }
    paths
}

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
pub fn paths_from_urls(urls: impl IntoIterator<Item = Url>) -> Vec<PathBuf> {
    urls.into_iter()
        .filter_map(|url| url.to_file_path().ok())
        .collect()
}

fn normalize_existing_files(paths: Vec<PathBuf>) -> Vec<String> {
    let mut out = Vec::new();
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let text = path_to_string(&path);
        if !out.iter().any(|item| item == &text) {
            out.push(text);
        }
    }
    out
}

/// 写入待打开队列；若 `emit` 为 true 则同时通知前端（热启动）
pub fn enqueue_open_paths(app: &AppHandle, paths: Vec<PathBuf>, emit: bool) {
    let files = normalize_existing_files(paths);
    if files.is_empty() {
        return;
    }

    if let Ok(mut pending) = app.state::<PendingOpenPaths>().0.lock() {
        for path in &files {
            if !pending.iter().any(|item| item == path) {
                pending.push(path.clone());
            }
        }
    }

    if emit {
        let _ = app.emit(OPEN_FILES_EVENT, files);
    }
}

pub fn take_pending(state: &PendingOpenPaths) -> Vec<String> {
    state
        .0
        .lock()
        .map(|mut pending| std::mem::take(&mut *pending))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_flags_and_keeps_plain_paths() {
        let paths = paths_from_args([
            "--flag".into(),
            "-v".into(),
            "/tmp/note.axdoc".into(),
            "file:///tmp/other.axdoc".into(),
        ]);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/tmp/note.axdoc"));
        assert_eq!(paths[1], PathBuf::from("/tmp/other.axdoc"));
    }
}
