use serde::Serialize;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

use super::cipher;
use super::convert;
use super::format;
use super::tree::{self, EditorTreeNode};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInspect {
    pub path: String,
    pub encrypted: bool,
    pub language: String,
    pub editable: bool,
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
pub fn editor_list_directory(path: String) -> Result<Vec<EditorTreeNode>, String> {
    tree::list_directory(PathBuf::from(path.trim()).as_path())
}

#[tauri::command]
pub fn editor_inspect_file(path: String) -> Result<FileInspect, String> {
    let path_buf = PathBuf::from(path.trim());
    Ok(FileInspect {
        path: tree::path_to_string(&path_buf),
        encrypted: format::is_encrypted_path(&path_buf),
        language: format::language_from_path(&path_buf),
        editable: format::is_encrypted_path(&path_buf) || format::is_editable_path(&path_buf),
    })
}

#[tauri::command]
pub fn editor_read_file(app: tauri::AppHandle, path: String) -> Result<String, String> {
    cipher::read_file_content(&app, PathBuf::from(path.trim()).as_path())
}

#[tauri::command]
pub fn editor_write_file(app: tauri::AppHandle, path: String, content: String) -> Result<(), String> {
    cipher::write_file_content(&app, PathBuf::from(path.trim()).as_path(), &content)
}

#[tauri::command]
pub fn editor_create_file(
    app: tauri::AppHandle,
    directory: String,
    file_name: Option<String>,
    encrypted: Option<bool>,
    content: Option<String>,
) -> Result<String, String> {
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
    cipher::write_file_content(&app, &path, content.as_deref().unwrap_or(""))?;
    Ok(tree::path_to_string(&path))
}

#[tauri::command]
pub fn editor_create_directory(directory: String, folder_name: String) -> Result<String, String> {
    tree::create_directory(PathBuf::from(directory.trim()).as_path(), &folder_name)
}

#[tauri::command]
pub fn editor_delete_path(path: String) -> Result<(), String> {
    tree::delete_path(PathBuf::from(path.trim()).as_path())
}

#[tauri::command]
pub fn editor_rename_path(path: String, new_name: String) -> Result<String, String> {
    tree::rename_path(PathBuf::from(path.trim()).as_path(), &new_name)
}

#[tauri::command]
pub fn editor_convert_to_encrypted(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, String> {
    convert::convert_to_encrypted(
        &app,
        PathBuf::from(path.trim()).as_path(),
    )
}

#[tauri::command]
pub fn editor_convert_to_plain(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, String> {
    convert::convert_to_plain(
        &app,
        PathBuf::from(path.trim()).as_path(),
    )
}
