use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::formats::{
    optimize_gif, optimize_jpeg, optimize_png, optimize_svg, OptimizeOptions,
};
use super::model::OptimizeResult;
use super::settings::ImageOptimSettings;

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "svg"];
const MAX_EXPAND_FILES: usize = 10_000;
const MAX_EXPAND_DEPTH: u32 = 32;

pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x)))
        .unwrap_or(false)
}

pub fn expand_paths(paths: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut visited = HashSet::new();
    for raw in paths {
        let path = PathBuf::from(raw.trim());
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            if is_image_path(&path) {
                push_unique(&mut out, path)?;
            }
            continue;
        }
        if path.is_dir() {
            collect_dir(&path, &mut out, &mut visited, 0)?;
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn push_unique(out: &mut Vec<String>, path: PathBuf) -> Result<(), String> {
    if out.len() >= MAX_EXPAND_FILES {
        return Err(format!("图片数量超过上限（{MAX_EXPAND_FILES}）"));
    }
    out.push(path.to_string_lossy().to_string());
    Ok(())
}

fn collect_dir(
    dir: &Path,
    out: &mut Vec<String>,
    visited: &mut HashSet<PathBuf>,
    depth: u32,
) -> Result<(), String> {
    if depth > MAX_EXPAND_DEPTH {
        return Ok(());
    }
    let canonical = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !visited.insert(canonical) {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败：{e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败：{e}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        // 跳过符号链接目录，避免环与意外外溢
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_dir(&path, out, visited, depth + 1)?;
        } else if meta.is_file() && is_image_path(&path) {
            push_unique(out, path)?;
        }
    }
    Ok(())
}

pub fn optimize_file(
    path: &Path,
    settings: &ImageOptimSettings,
    lossy_override: Option<bool>,
) -> OptimizeResult {
    let path_str = path.to_string_lossy().to_string();
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => return OptimizeResult::failed(path_str, 0, format!("读取失败：{e}")),
    };
    let before = data.len() as u64;
    if before == 0 {
        return OptimizeResult::failed(path_str, 0, "空文件");
    }

    // 提前探测可写，避免 CPU 白跑
    if let Err(e) = probe_writable(path) {
        return OptimizeResult::failed(path_str, before, e);
    }

    let opts = OptimizeOptions::from_settings(settings, lossy_override);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let result = match ext.as_str() {
        "png" => optimize_png(&data, &opts),
        "jpg" | "jpeg" => optimize_jpeg(&data, &opts),
        "gif" => optimize_gif(&data, &opts),
        "svg" => optimize_svg(&data, &opts),
        _ => Err(format!("不支持的格式：{ext}")),
    };

    match result {
        Ok(Some(new_data)) => match atomic_write(path, &new_data) {
            Ok(()) => OptimizeResult::optimized(path_str, before, new_data.len() as u64),
            Err(e) => OptimizeResult::failed(path_str, before, e),
        },
        Ok(None) => OptimizeResult::already_optimal(path_str, before),
        Err(e) => OptimizeResult::failed(path_str, before, e),
    }
}

fn probe_writable(path: &Path) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .open(path)
        .map(|_| ())
        .map_err(|e| format!("文件不可写：{e}"))
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    let tmp = parent.join(format!(".{file_name}.appx-optim.tmp"));

    let perms = fs::metadata(path).ok().map(|m| m.permissions());

    {
        let mut file = File::create(&tmp).map_err(|e| format!("创建临时文件失败：{e}"))?;
        file.write_all(data)
            .map_err(|e| format!("写入临时文件失败：{e}"))?;
        file.sync_all()
            .map_err(|e| format!("同步临时文件失败：{e}"))?;
    }

    if let Some(perms) = perms {
        let _ = fs::set_permissions(&tmp, perms);
    }

    replace_file(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("替换原文件失败：{e}")
    })?;
    Ok(())
}

fn replace_file(tmp: &Path, dest: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        // Windows：rename 无法覆盖已存在目标
        let _ = fs::remove_file(dest);
        fs::rename(tmp, dest)
    }
    #[cfg(not(windows))]
    {
        fs::rename(tmp, dest)
    }
}
