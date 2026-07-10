use std::fs;
use std::path::{Path, PathBuf};

use super::cipher;
use super::format;
use super::passphrase_store::FilePassphraseStore;
use super::tree;

fn trim_passphrase(passphrase: &str) -> Result<&str, String> {
    let trimmed = passphrase.trim();
    if trimmed.is_empty() {
        return Err("口令不能为空".to_string());
    }
    Ok(trimmed)
}

fn ensure_target_available(source: &Path, target: &Path) -> Result<(), String> {
    if target != source && target.exists() {
        return Err(format!("目标已存在：{}", target.display()));
    }
    Ok(())
}

fn read_plaintext_source(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
) -> Result<String, String> {
    if format::is_encrypted_path(path) {
        return cipher::read_encrypted_payload(app, store, path);
    }
    if !format::is_editable_path(path) {
        return Err(format!("无法转换该文件：{}", path.display()));
    }
    cipher::read_file_content(app, store, path)
}

fn replace_encrypted_file(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    source: &Path,
    target: &Path,
    content: &str,
) -> Result<String, String> {
    ensure_target_available(source, target)?;
    cipher::write_file_content(app, store, target, content)?;
    if target == source {
        return Ok(tree::path_to_string(target));
    }

    let source_key = tree::path_to_string(source);
    fs::remove_file(source).map_err(|err| format!("删除原文件失败：{err}"))?;
    store.remove(&source_key);
    Ok(tree::path_to_string(target))
}

pub fn convert_to_encrypted(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
) -> Result<String, String> {
    if format::is_encrypted_path(path) {
        return Err("该文件已经是加密格式".to_string());
    }

    let content = read_plaintext_source(app, store, path)?;
    let target = format::default_encrypted_path_from_plain(path);
    replace_encrypted_file(app, store, path, &target, &content)
}

pub fn convert_to_custom_encrypted(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
    passphrase: &str,
) -> Result<String, String> {
    let trimmed = trim_passphrase(passphrase)?;

    let content = read_plaintext_source(app, store, path)?;
    let target = format::custom_encrypt_target(path);
    ensure_target_available(path, &target)?;

    let target_key = tree::path_to_string(&target);
    store.set(&target_key, trimmed.to_string());
    cipher::write_file_content(app, store, &target, &content)?;

    if target != path {
        fs::remove_file(path).map_err(|err| format!("删除原文件失败：{err}"))?;
        store.remove(&tree::path_to_string(path));
    }

    Ok(target_key)
}

pub fn convert_custom_to_default_encrypted(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
) -> Result<String, String> {
    if !format::is_custom_encrypted_path(path) {
        return Err("该文件不是独立口令加密格式 (.x0)".to_string());
    }

    let content = cipher::read_encrypted_payload(app, store, path)?;
    let target = format::default_encrypted_path_from_custom(path);
    replace_encrypted_file(app, store, path, &target, &content)
}

pub fn convert_to_plain(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &Path,
) -> Result<String, String> {
    if !format::is_encrypted_path(path) {
        return Err("该文件不是加密格式".to_string());
    }

    let target = format::plain_path_from_encrypted(path)
        .ok_or_else(|| format!("无法解析加密文件路径：{}", path.display()))?;
    ensure_target_available(path, &target)?;

    let content = cipher::read_encrypted_payload(app, store, path)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("创建目录失败：{err}"))?;
    }
    fs::write(&target, content).map_err(|err| format!("写入普通文件失败：{err}"))?;
    fs::remove_file(path).map_err(|err| format!("删除加密文件失败：{err}"))?;
    store.remove(&tree::path_to_string(path));

    Ok(tree::path_to_string(&target))
}

pub fn unlock_encrypted_file(
    store: &FilePassphraseStore,
    path: &Path,
    passphrase: &str,
) -> Result<(String, String), String> {
    let trimmed = trim_passphrase(passphrase)?;
    if !format::is_encrypted_path(path) {
        return Err("该文件不是加密格式".to_string());
    }

    let plain = cipher::try_decrypt_with_passphrase(path, trimmed)?;
    let content = String::from_utf8(plain).map_err(|_| "解密内容不是有效 UTF-8 文本".to_string())?;

    let final_path: PathBuf = if format::is_default_encrypted_path(path) {
        let target = format::custom_encrypted_path_from_default(path);
        ensure_target_available(path, &target)?;
        fs::rename(path, &target).map_err(|err| format!("重命名为 .x0 失败：{err}"))?;
        target
    } else {
        path.to_path_buf()
    };

    let path_key = tree::path_to_string(&final_path);
    store.set(&path_key, trimmed.to_string());
    Ok((path_key, content))
}
