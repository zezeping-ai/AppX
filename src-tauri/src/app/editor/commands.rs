use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::app::app_lock::AppLockSessionState;
use crate::app::editor::passphrase_store::FilePassphraseStore;

use super::cipher;
use super::convert;
use super::format;
use super::tree::{self, EditorTreeNode};

fn ensure_unlocked(state: &AppLockSessionState) -> Result<(), String> {
    if state.is_locked()? {
        return Err("应用已锁定，请先解锁".to_string());
    }
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInspect {
    pub path: String,
    pub encrypted: bool,
    pub custom_encrypted: bool,
    pub language: String,
    pub editable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockEncryptedFileResult {
    pub path: String,
    pub content: String,
}

#[tauri::command]
pub async fn editor_pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .set_title("选择工作文件夹")
        .blocking_pick_folder();

    Ok(picked.map(|path| path.to_string()))
}

#[tauri::command]
pub async fn editor_pick_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .set_title("打开文件")
        .blocking_pick_file();

    Ok(picked.map(|path| path.to_string()))
}

#[tauri::command]
pub fn editor_list_directory(
    state: State<'_, AppLockSessionState>,
    path: String,
) -> Result<Vec<EditorTreeNode>, String> {
    ensure_unlocked(&state)?;
    tree::list_directory(PathBuf::from(path.trim()).as_path())
}

fn parse_path(path: String) -> PathBuf {
    PathBuf::from(path.trim())
}

fn build_file_inspect(path: &Path) -> FileInspect {
    FileInspect {
        path: tree::path_to_string(path),
        encrypted: format::is_encrypted_path(path),
        custom_encrypted: format::is_custom_encrypted_path(path),
        language: format::language_from_path(path),
        editable: format::is_encrypted_path(path) || format::is_editable_path(path),
    }
}

#[tauri::command]
pub fn editor_inspect_file(
    state: State<'_, AppLockSessionState>,
    path: String,
) -> Result<FileInspect, String> {
    ensure_unlocked(&state)?;
    Ok(build_file_inspect(&parse_path(path)))
}

#[tauri::command]
pub fn editor_read_file(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    cipher::read_file_content(&app, &passphrase_store, &parse_path(path))
}

#[tauri::command]
pub fn editor_write_file(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
    content: String,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
    cipher::write_file_content(&app, &passphrase_store, &parse_path(path), &content)
}

#[tauri::command]
pub fn editor_unlock_encrypted_file(
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
    passphrase: String,
) -> Result<UnlockEncryptedFileResult, String> {
    ensure_unlocked(&state)?;
    let path_buf = parse_path(path);
    let (next_path, content) =
        convert::unlock_encrypted_file(&passphrase_store, &path_buf, &passphrase)?;
    Ok(UnlockEncryptedFileResult {
        path: next_path,
        content,
    })
}

#[tauri::command]
pub fn editor_create_file(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    directory: String,
    file_name: Option<String>,
    encrypted: Option<bool>,
    content: Option<String>,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    let use_encrypted = encrypted.unwrap_or(false);
    let name = file_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| tree::default_new_file_name(use_encrypted));

    let path = tree::join_path(directory.trim(), &name);
    if path.exists() {
        return Err(format!("文件已存在：{}", path.display()));
    }

    format::ensure_writable_path(&path)?;
    cipher::write_file_content(
        &app,
        &passphrase_store,
        &path,
        content.as_deref().unwrap_or(""),
    )?;
    Ok(tree::path_to_string(&path))
}

#[tauri::command]
pub fn editor_create_directory(
    state: State<'_, AppLockSessionState>,
    directory: String,
    folder_name: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    tree::create_directory(PathBuf::from(directory.trim()).as_path(), &folder_name)
}

#[tauri::command]
pub fn editor_delete_path(
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
    let path_buf = parse_path(path);
    tree::delete_path(&path_buf)?;
    passphrase_store.remove(&tree::path_to_string(&path_buf));
    Ok(())
}

#[tauri::command]
pub fn editor_rename_path(
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
    new_name: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    let path_buf = parse_path(path);
    let old_key = tree::path_to_string(&path_buf);
    let new_path = tree::rename_path(&path_buf, &new_name)?;
    passphrase_store.rename_key(&old_key, &new_path);
    Ok(new_path)
}

#[tauri::command]
pub fn editor_convert_to_encrypted(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    convert::convert_to_encrypted(&app, &passphrase_store, &parse_path(path))
}

#[tauri::command]
pub fn editor_convert_to_custom_encrypted(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
    passphrase: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    convert::convert_to_custom_encrypted(&app, &passphrase_store, &parse_path(path), &passphrase)
}

#[tauri::command]
pub fn editor_convert_custom_to_default_encrypted(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    convert::convert_custom_to_default_encrypted(&app, &passphrase_store, &parse_path(path))
}

#[tauri::command]
pub fn editor_convert_to_plain(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    passphrase_store: State<'_, FilePassphraseStore>,
    path: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    convert::convert_to_plain(&app, &passphrase_store, &parse_path(path))
}
