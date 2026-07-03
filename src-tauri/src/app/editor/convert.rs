use std::fs;
use std::path::Path;

use super::cipher;
use super::format;
use super::tree;

pub fn convert_to_encrypted(app: &tauri::AppHandle, path: &Path) -> Result<String, String> {
    if format::is_encrypted_path(path) {
        return Err("该文件已经是加密格式".to_string());
    }
    if !format::is_editable_path(path) {
        return Err(format!("无法转换该文件：{}", path.display()));
    }

    let content = cipher::read_file_content(app, path)?;
    let target = format::encrypted_path_from_plain(path);
    if target.exists() {
        return Err(format!("加密目标已存在：{}", target.display()));
    }

    cipher::write_file_content(app, &target, &content)?;
    fs::remove_file(path).map_err(|err| format!("删除原文件失败：{err}"))?;

    Ok(tree::path_to_string(&target))
}

pub fn convert_to_plain(app: &tauri::AppHandle, path: &Path) -> Result<String, String> {
    if !format::is_encrypted_path(path) {
        return Err("该文件不是加密格式".to_string());
    }

    let target = format::plain_path_from_encrypted(path)
        .ok_or_else(|| format!("无法解析加密文件路径：{}", path.display()))?;
    if target.exists() {
        return Err(format!("普通文件目标已存在：{}", target.display()));
    }

    let content = cipher::read_file_content(app, path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建目录失败：{err}"))?;
    }
    fs::write(&target, content).map_err(|err| format!("写入普通文件失败：{err}"))?;
    fs::remove_file(path).map_err(|err| format!("删除加密文件失败：{err}"))?;

    Ok(tree::path_to_string(&target))
}
