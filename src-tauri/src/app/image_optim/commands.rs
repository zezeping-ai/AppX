use std::path::PathBuf;

use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};

use super::model::{OptimizeFileInput, OptimizeResult};
use super::pipeline;
use super::settings::read_image_optim_settings;

#[tauri::command]
pub async fn image_optim_pick_files(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<Vec<String>, String> {
    ensure_unlocked(&state)?;
    let picked = app
        .dialog()
        .file()
        .set_title("选择图片")
        .add_filter("图片", &["png", "jpg", "jpeg", "gif", "svg"])
        .blocking_pick_files();

    Ok(picked
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string())
        .collect())
}

#[tauri::command]
pub async fn image_optim_pick_folder(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<Option<String>, String> {
    ensure_unlocked(&state)?;
    let picked = app
        .dialog()
        .file()
        .set_title("选择图片文件夹")
        .blocking_pick_folder();
    Ok(picked.map(|p| p.to_string()))
}

#[tauri::command]
pub fn image_optim_expand_paths(
    state: State<'_, AppLockSessionState>,
    paths: Vec<String>,
) -> Result<Vec<String>, String> {
    ensure_unlocked(&state)?;
    pipeline::expand_paths(&paths)
}

#[tauri::command]
pub async fn image_optim_optimize_file(
    app: tauri::AppHandle,
    state: State<'_, AppLockSessionState>,
    input: OptimizeFileInput,
) -> Result<OptimizeResult, String> {
    ensure_unlocked(&state)?;
    let settings = read_image_optim_settings(&app)?;
    let path = PathBuf::from(input.path.trim());
    let lossy = input.lossy;

    // CPU 密集：放到阻塞线程，避免卡住 IPC
    tauri::async_runtime::spawn_blocking(move || {
        if !path.is_file() {
            return OptimizeResult::failed(path.to_string_lossy().to_string(), 0, "文件不存在");
        }
        pipeline::optimize_file(&path, &settings, lossy)
    })
    .await
    .map_err(|e| format!("优化任务失败：{e}"))
}
