use tauri::AppHandle;

use crate::database;

#[tauri::command]
pub fn database_resolve_path(app: AppHandle) -> Result<String, String> {
    database::db_full_path(&app).map(|path| path.display().to_string())
}

#[tauri::command]
pub fn database_reset_dev(app: AppHandle) -> Result<bool, String> {
    if !cfg!(debug_assertions) {
        return Err("Reset database is only available in development builds.".to_string());
    }

    let path = database::db_full_path(&app)?;
    let mut targets = vec![path.clone()];
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        targets.push(path.with_file_name(format!("{name}-wal")));
        targets.push(path.with_file_name(format!("{name}-shm")));
    }

    for candidate in targets {
        if candidate.exists() {
            std::fs::remove_file(&candidate)
                .map_err(|err| format!("无法删除数据库文件 {}：{err}", candidate.display()))?;
        }
    }

    Ok(true)
}
