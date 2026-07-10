use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::format;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditorTreeNode {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub encrypted: Option<bool>,
    pub custom_encrypted: Option<bool>,
    pub language: Option<String>,
    pub children: Option<Vec<EditorTreeNode>>,
}

pub fn list_directory(root: &Path) -> Result<Vec<EditorTreeNode>, String> {
    if !root.is_dir() {
        return Err(format!("不是有效目录：{}", root.display()));
    }

    let mut entries: Vec<_> = fs::read_dir(root)
        .map_err(|err| format!("读取目录失败：{err}"))?
        .filter_map(Result::ok)
        .collect();

    entries.sort_by(compare_dir_entries);

    let mut nodes = Vec::new();
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".DS_Store" {
            continue;
        }

        if path.is_dir() {
            nodes.push(EditorTreeNode {
                name,
                path: path_to_string(&path),
                kind: "directory".to_string(),
                encrypted: None,
                custom_encrypted: None,
                language: None,
                children: Some(list_directory(&path)?),
            });
            continue;
        }

        if !should_list_file(&path) {
            continue;
        }

        nodes.push(EditorTreeNode {
            name,
            path: path_to_string(&path),
            kind: "file".to_string(),
            encrypted: Some(format::is_encrypted_path(&path)),
            custom_encrypted: Some(format::is_custom_encrypted_path(&path)),
            language: Some(format::language_from_path(&path)),
            children: None,
        });
    }

    Ok(nodes)
}

/// VS Code 默认：文件夹在前，同类型按名称不区分大小写 + 数字自然序。
fn compare_dir_entries(a: &fs::DirEntry, b: &fs::DirEntry) -> std::cmp::Ordering {
    let a_is_dir = a.path().is_dir();
    let b_is_dir = b.path().is_dir();
    match (a_is_dir, b_is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => compare_file_names(
            &a.file_name().to_string_lossy(),
            &b.file_name().to_string_lossy(),
        ),
    }
}

fn compare_file_names(a: &str, b: &str) -> std::cmp::Ordering {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut i = 0;
    let mut j = 0;

    while i < a_chars.len() && j < b_chars.len() {
        let ac = a_chars[i];
        let bc = b_chars[j];
        let a_digit = ac.is_ascii_digit();
        let b_digit = bc.is_ascii_digit();

        if a_digit && b_digit {
            let (a_num, a_end) = parse_number_run(&a_chars, i);
            let (b_num, b_end) = parse_number_run(&b_chars, j);
            if a_num != b_num {
                return a_num.cmp(&b_num);
            }
            i = a_end;
            j = b_end;
            continue;
        }

        if a_digit != b_digit {
            return if a_digit {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }

        let ord = compare_chars_case_insensitive(ac, bc);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }

    a_chars.len().cmp(&b_chars.len())
}

fn compare_chars_case_insensitive(a: char, b: char) -> std::cmp::Ordering {
    let a_lower: String = a.to_lowercase().collect();
    let b_lower: String = b.to_lowercase().collect();
    a_lower.cmp(&b_lower).then_with(|| a.cmp(&b))
}

fn parse_number_run(chars: &[char], start: usize) -> (u64, usize) {
    let mut i = start;
    let mut value = 0u64;
    while i < chars.len() && chars[i].is_ascii_digit() {
        value = value
            .saturating_mul(10)
            .saturating_add(chars[i].to_digit(10).unwrap_or(0) as u64);
        i += 1;
    }
    (value, i)
}

#[cfg(test)]
mod tests {
    use super::compare_file_names;

    #[test]
    fn names_sort_case_insensitive_with_natural_numbers() {
        assert_eq!(compare_file_names("Alpha", "beta"), std::cmp::Ordering::Less);
        assert_eq!(compare_file_names("file2", "file10"), std::cmp::Ordering::Less);
        assert_eq!(compare_file_names("a", "A"), std::cmp::Ordering::Greater);
    }
}

fn should_list_file(path: &Path) -> bool {
    if format::is_encrypted_path(path) || format::is_editable_path(path) {
        return true;
    }

    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if name.starts_with('.') {
        return name == ".gitignore" || name == ".env" || name == ".editorconfig";
    }

    true
}

pub fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn default_new_file_name(encrypted: bool) -> String {
    if encrypted {
        "untitled.txt.x".to_string()
    } else {
        "untitled.txt".to_string()
    }
}

pub fn join_path(dir: &str, name: &str) -> PathBuf {
    PathBuf::from(dir).join(name)
}

fn validate_entry_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("名称不能为空".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("无效的名称".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("名称不能包含路径分隔符".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn create_directory(parent: &Path, name: &str) -> Result<String, String> {
    if !parent.is_dir() {
        return Err(format!("不是有效目录：{}", parent.display()));
    }
    let name = validate_entry_name(name)?;
    let path = parent.join(&name);
    if path.exists() {
        return Err(format!("已存在：{}", path.display()));
    }
    fs::create_dir(&path).map_err(|err| format!("创建文件夹失败：{err}"))?;
    Ok(path_to_string(&path))
}

pub fn delete_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路径不存在：{}", path.display()));
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|err| format!("删除文件夹失败：{err}"))?;
    } else {
        fs::remove_file(path).map_err(|err| format!("删除文件失败：{err}"))?;
    }
    Ok(())
}

pub fn rename_path(path: &Path, new_name: &str) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("路径不存在：{}", path.display()));
    }
    let new_name = validate_entry_name(new_name)?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法解析父目录".to_string())?;
    let new_path = parent.join(&new_name);
    if new_path == path {
        return Ok(path_to_string(path));
    }
    if new_path.exists() {
        return Err(format!("已存在：{}", new_path.display()));
    }
    if path.is_file() {
        format::ensure_writable_path(&new_path)?;
    }
    fs::rename(path, &new_path).map_err(|err| format!("重命名失败：{err}"))?;
    Ok(path_to_string(&new_path))
}
