//! 工作区内容搜索（Cmd+Shift+F）

use serde::Serialize;
use std::path::Path;
use tauri::AppHandle;

use super::cipher;
use super::format;
use super::passphrase_store::FilePassphraseStore;
use super::tree;

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_MATCHES: usize = 2_000;
const MAX_MATCHES_PER_FILE: usize = 100;
const MAX_FILES_WITH_MATCHES: usize = 200;
const MAX_FILES_SCANNED: usize = 20_000;
const PREVIEW_MAX_CHARS: usize = 200;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchMatch {
    /// 1-based
    pub line: u32,
    /// 1-based，匹配起始列
    pub column: u32,
    /// 行预览（可能截断）
    pub preview: String,
    /// 预览内匹配起点（0-based，UTF-16 友好用字符下标）
    pub match_start: u32,
    pub match_end: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchFile {
    pub path: String,
    pub relative_path: String,
    pub name: String,
    pub language: String,
    pub matches: Vec<WorkspaceSearchMatch>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSearchResult {
    pub files: Vec<WorkspaceSearchFile>,
    pub match_count: u32,
    pub files_scanned: u32,
    pub truncated: bool,
    pub skipped_locked: u32,
}

pub fn search_workspace(
    app: &AppHandle,
    store: &FilePassphraseStore,
    workspace_root: &Path,
    query: &str,
    case_sensitive: bool,
) -> Result<WorkspaceSearchResult, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(WorkspaceSearchResult {
            files: Vec::new(),
            match_count: 0,
            files_scanned: 0,
            truncated: false,
            skipped_locked: 0,
        });
    }
    if !workspace_root.is_dir() {
        return Err(format!("不是有效目录：{}", workspace_root.display()));
    }

    let mut files = Vec::new();
    let mut match_count = 0usize;
    let mut files_scanned = 0u32;
    let mut skipped_locked = 0u32;
    let mut truncated = false;

    for result in tree::workspace_walk_builder(workspace_root).build() {
        if truncated {
            break;
        }
        let entry = result.map_err(|err| format!("遍历工作区失败：{err}"))?;
        let path = entry.path();
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.is_empty() || name == ".DS_Store" {
            continue;
        }
        if should_skip_path(path) {
            continue;
        }

        files_scanned += 1;
        if files_scanned as usize > MAX_FILES_SCANNED {
            truncated = true;
            break;
        }

        let content = match read_searchable_content(app, store, path) {
            Ok(Some(text)) => text,
            Ok(None) => continue,
            Err(err) if err.starts_with(cipher::ERR_DECRYPT_PASSPHRASE_REQUIRED) => {
                skipped_locked += 1;
                continue;
            }
            Err(_) => continue,
        };

        let relative = match relative_path(workspace_root, path) {
            Some(rel) => rel,
            None => continue,
        };

        let file_matches = find_in_content(&content, query, case_sensitive);
        if file_matches.is_empty() {
            continue;
        }

        match_count += file_matches.len();
        files.push(WorkspaceSearchFile {
            path: tree::path_to_string(path),
            relative_path: relative,
            name,
            language: format::language_from_path(path),
            matches: file_matches,
        });

        if match_count >= MAX_MATCHES || files.len() >= MAX_FILES_WITH_MATCHES {
            truncated = true;
            break;
        }
    }

    Ok(WorkspaceSearchResult {
        files,
        match_count: match_count as u32,
        files_scanned,
        truncated,
        skipped_locked,
    })
}

fn should_skip_path(path: &Path) -> bool {
    if format::is_encrypted_path(path) {
        return false;
    }
    if let Some(ext) = format::plain_extension(path) {
        // 复用 format 里的二进制扩展名判断
        const BINARY: &[&str] = &[
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", "pdf", "zip", "gz", "tar",
            "7z", "rar", "dmg", "exe", "dll", "so", "dylib", "wasm", "mp3", "mp4", "mov", "avi",
            "mkv", "woff", "woff2", "ttf", "otf", "eot", "bin", "dat", "doc", "docx", "docm",
            "xls", "xlsx", "xlsm", "ppt", "pptx", "pptm", "odt", "ods", "odp", "rtf",
        ];
        if BINARY.contains(&ext.as_str()) {
            return true;
        }
    }
    false
}

fn relative_path(workspace_root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(workspace_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }
    Some(
        relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

fn read_searchable_content(
    app: &AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
) -> Result<Option<String>, String> {
    let meta = std::fs::metadata(path).map_err(|err| format!("读取元数据失败：{err}"))?;
    if meta.len() > MAX_FILE_BYTES {
        return Ok(None);
    }

    if format::is_encrypted_path(path) {
        let text = cipher::read_file_content(app, store, path)?;
        return Ok(Some(text));
    }

    if !format::is_supported_editor_path(path) {
        return Ok(None);
    }

    let bytes = std::fs::read(path).map_err(|err| format!("读取文件失败：{err}"))?;
    if bytes.contains(&0) {
        return Ok(None);
    }
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None),
    }
}

fn find_in_content(
    content: &str,
    query: &str,
    case_sensitive: bool,
) -> Vec<WorkspaceSearchMatch> {
    let mut out = Vec::new();
    if query.is_empty() {
        return out;
    }

    for (line_idx, line) in content.lines().enumerate() {
        if out.len() >= MAX_MATCHES_PER_FILE {
            break;
        }
        for (start_char, end_char) in line_match_ranges(line, query, case_sensitive) {
            if out.len() >= MAX_MATCHES_PER_FILE {
                break;
            }
            let (preview, match_start, match_end) = build_preview(line, start_char, end_char);
            out.push(WorkspaceSearchMatch {
                line: (line_idx + 1) as u32,
                column: (start_char + 1) as u32,
                preview,
                match_start: match_start as u32,
                match_end: match_end as u32,
            });
        }
    }
    out
}

/// 返回行内匹配的字符下标区间 `[start, end)`
fn line_match_ranges(line: &str, query: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
    let line_chars: Vec<char> = line.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();
    let qlen = query_chars.len();
    if qlen == 0 || line_chars.len() < qlen {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let last = line_chars.len() - qlen;
    let mut i = 0usize;
    while i <= last {
        let matched = (0..qlen).all(|j| {
            let a = line_chars[i + j];
            let b = query_chars[j];
            if case_sensitive {
                a == b
            } else {
                a.to_lowercase().eq(b.to_lowercase())
            }
        });
        if matched {
            ranges.push((i, i + qlen));
            i += qlen;
        } else {
            i += 1;
        }
    }
    ranges
}

fn build_preview(line: &str, start_char: usize, end_char: usize) -> (String, usize, usize) {
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= PREVIEW_MAX_CHARS {
        return (line.to_string(), start_char, end_char);
    }

    let window = PREVIEW_MAX_CHARS;
    let match_mid = (start_char + end_char) / 2;
    let mut from = match_mid.saturating_sub(window / 2);
    if from + window > chars.len() {
        from = chars.len().saturating_sub(window);
    }
    let to = (from + window).min(chars.len());
    let slice: String = chars[from..to].iter().collect();
    let has_prefix = from > 0;
    let has_suffix = to < chars.len();
    let preview = format!(
        "{}{}{}",
        if has_prefix { "…" } else { "" },
        slice,
        if has_suffix { "…" } else { "" },
    );
    let prefix_len = if has_prefix { 1 } else { 0 };
    (
        preview,
        start_char.saturating_sub(from) + prefix_len,
        end_char.saturating_sub(from) + prefix_len,
    )
}

#[cfg(test)]
mod tests {
    use super::{build_preview, find_in_content};

    #[test]
    fn finds_case_insensitive_matches() {
        let content = "Hello World\nhello again\n";
        let matches = find_in_content(content, "hello", false);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[1].line, 2);
    }

    #[test]
    fn finds_case_sensitive_matches() {
        let content = "Hello\nhello\n";
        let matches = find_in_content(content, "Hello", true);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, 1);
    }

    #[test]
    fn preview_marks_match_range() {
        let (preview, start, end) = build_preview("abc DEF ghi", 4, 7);
        let chars: Vec<char> = preview.chars().collect();
        let hit: String = chars[start..end].iter().collect();
        assert_eq!(hit, "DEF");
    }
}
