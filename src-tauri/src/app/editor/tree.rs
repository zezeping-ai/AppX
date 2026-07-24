use ignore::{IncrementalIgnore, WalkBuilder};
use serde::Serialize;
use std::cmp::Ordering;
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
    /// 点文件 / 点目录（如 `.env`、`.git`）
    pub hidden: bool,
    /// 被 `.gitignore` / `.git/info/exclude` / 全局 excludes 忽略
    pub ignored: bool,
    /// 目录：未加载时为 `None`（前端按需拉取）；已加载为空目录时为 `Some([])`。
    /// 文件：始终为 `None`。
    pub children: Option<Vec<EditorTreeNode>>,
}

/// 仅列出一层目录内容（懒加载）。目录节点 `children = None` 表示尚未展开加载。
/// `workspace_root` 用于构建 ignore 匹配器；未提供时仅标记 hidden。
pub fn list_directory(
    root: &Path,
    workspace_root: Option<&Path>,
) -> Result<Vec<EditorTreeNode>, String> {
    if !root.is_dir() {
        return Err(format!("不是有效目录：{}", root.display()));
    }

    let mut ignore = workspace_root.and_then(build_ignore_matcher);

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

        let hidden = is_hidden_name(&name);
        let is_dir = path.is_dir();
        let ignored = ignore
            .as_mut()
            .zip(workspace_root)
            .map(|(matcher, ws)| is_ignored(matcher, ws, &path, is_dir))
            .unwrap_or(false);

        if is_dir {
            nodes.push(EditorTreeNode {
                name,
                path: path_to_string(&path),
                kind: "directory".to_string(),
                encrypted: None,
                custom_encrypted: None,
                language: None,
                hidden,
                ignored,
                children: None,
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
            hidden,
            ignored,
            children: None,
        });
    }

    Ok(nodes)
}

fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') && name != "." && name != ".."
}

/// BurntSushi `ignore`（ripgrep 同款）：`WalkBuilder::build_matchers` → `IncrementalIgnore`
/// 支持嵌套 `.gitignore`、`.git/info/exclude`、全局 excludes，以及祖先目录规则。
fn build_ignore_matcher(workspace_root: &Path) -> Option<IncrementalIgnore> {
    if !workspace_root.is_dir() {
        return None;
    }
    let mut builder = WalkBuilder::new(workspace_root);
    builder
        .hidden(false)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        // 工作区未必是 git 仓库，仍尊重目录内 `.gitignore`
        .require_git(false);
    builder.build_matchers().into_iter().next()
}

fn is_ignored(
    matcher: &mut IncrementalIgnore,
    workspace_root: &Path,
    path: &Path,
    is_dir: bool,
) -> bool {
    let relative = match path.strip_prefix(workspace_root) {
        Ok(rel) if !rel.as_os_str().is_empty() => rel,
        _ => return false,
    };
    matcher.matched(relative, is_dir).is_ignore()
}

/// VS Code 默认：文件夹在前，同类型按名称不区分大小写 + 数字自然序。
fn compare_dir_entries(a: &fs::DirEntry, b: &fs::DirEntry) -> Ordering {
    let a_is_dir = a.path().is_dir();
    let b_is_dir = b.path().is_dir();
    match (a_is_dir, b_is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => compare_file_names(
            &a.file_name().to_string_lossy(),
            &b.file_name().to_string_lossy(),
        ),
    }
}

/// 全序自然排序：数字段按数值比较；相等时再比数字串长度；其余按不区分大小写字符比较。
fn compare_file_names(a: &str, b: &str) -> Ordering {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut i = 0;
    let mut j = 0;

    while i < a_chars.len() && j < b_chars.len() {
        let ac = a_chars[i];
        let bc = b_chars[j];

        if ac.is_ascii_digit() && bc.is_ascii_digit() {
            let (a_num, a_end) = parse_number_run(&a_chars, i);
            let (b_num, b_end) = parse_number_run(&b_chars, j);
            // 数值相同仍比较位数，保证 "1" 与 "01" 有确定全序，避免 sort panic
            match a_num
                .cmp(&b_num)
                .then_with(|| (a_end - i).cmp(&(b_end - j)))
            {
                Ordering::Equal => {
                    i = a_end;
                    j = b_end;
                }
                ord => return ord,
            }
            continue;
        }

        let ord = compare_chars_case_insensitive(ac, bc);
        if ord != Ordering::Equal {
            return ord;
        }
        i += 1;
        j += 1;
    }

    a_chars
        .len()
        .cmp(&b_chars.len())
        .then_with(|| a.cmp(b))
}

fn compare_chars_case_insensitive(a: char, b: char) -> Ordering {
    // ASCII 快路径，避免 Unicode 大小写折叠多字符破坏直觉的同时保持全序
    if a.is_ascii() && b.is_ascii() {
        return a
            .to_ascii_lowercase()
            .cmp(&b.to_ascii_lowercase())
            .then_with(|| a.cmp(&b));
    }
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

#[cfg(test)]
mod tests {
    use super::{
        build_ignore_matcher, compare_file_names, is_hidden_name, is_ignored, list_directory,
    };
    use std::cmp::Ordering;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "appx-{}-{}",
            tag,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn names_sort_case_insensitive_with_natural_numbers() {
        assert_eq!(compare_file_names("Alpha", "beta"), Ordering::Less);
        assert_eq!(compare_file_names("file2", "file10"), Ordering::Less);
        assert_eq!(compare_file_names("a", "A"), Ordering::Greater);
    }

    #[test]
    fn natural_sort_is_total_order_for_common_project_names() {
        let mut names = vec![
            "file10.ts",
            "file2.ts",
            "file.ts",
            "file01.ts",
            "File3.ts",
            "1a",
            "a1",
            "a01",
            "README.md",
            "package.json",
            "src",
            ".env",
        ];
        names.sort_by(|a, b| compare_file_names(a, b));
        let mut again = names.clone();
        again.sort_by(|a, b| compare_file_names(a, b));
        assert_eq!(names, again);
        assert_eq!(compare_file_names("file01.ts", "file1.ts"), Ordering::Greater);
    }

    #[test]
    fn hidden_names_start_with_dot() {
        assert!(is_hidden_name(".env"));
        assert!(is_hidden_name(".git"));
        assert!(!is_hidden_name("src"));
    }

    #[test]
    fn incremental_ignore_respects_gitignore_directories() {
        let dir = temp_dir("gitignore");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        fs::write(dir.join(".gitignore"), "node_modules/\ndist\n*.log\n").unwrap();
        fs::write(dir.join("src/app.ts"), "").unwrap();
        fs::write(dir.join("debug.log"), "").unwrap();

        let mut matcher = build_ignore_matcher(&dir).expect("matcher");
        assert!(is_ignored(&mut matcher, &dir, &dir.join("node_modules"), true));
        assert!(is_ignored(
            &mut matcher,
            &dir,
            &dir.join("node_modules/pkg"),
            true
        ));
        assert!(is_ignored(&mut matcher, &dir, &dir.join("debug.log"), false));
        assert!(!is_ignored(&mut matcher, &dir, &dir.join("src/app.ts"), false));

        let nodes = list_directory(&dir, Some(&dir)).unwrap();
        let node_modules = nodes.iter().find(|n| n.name == "node_modules").unwrap();
        assert!(node_modules.ignored);
        let src = nodes.iter().find(|n| n.name == "src").unwrap();
        assert!(!src.ignored);
        let log = nodes.iter().find(|n| n.name == "debug.log").unwrap();
        assert!(log.ignored);

        let _ = fs::remove_dir_all(&dir);
    }
}
