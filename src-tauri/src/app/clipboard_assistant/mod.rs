mod access;
mod app_icon;
mod cache;
mod cleanup;
mod commands;
mod db;
mod enricher;
mod fts;
mod history;
mod ingest;
mod model;
mod palette;
mod palette_geometry;
mod payload;
mod protocol;
mod session;
pub mod settings;
mod state;
mod thumb;

use std::sync::Arc;

use tauri::{AppHandle, Manager};

pub use commands::*;
pub use protocol::register_protocols;

use crate::app::app_lock::is_session_locked;

use state::ClipboardAssistantState;

pub fn setup(app: &AppHandle) -> Result<(), String> {
    session::register_hooks();
    settings::load_runtime_flags(app);

    let conn = db::open_db(&db::db_path(app)?)?;
    let _ = db::refresh_text_previews(&conn);
    let blobs = db::blobs_dir(app)?;
    std::fs::create_dir_all(&blobs).map_err(|e| format!("创建 blob 目录失败：{e}"))?;
    let icons = db::icons_dir(app)?;
    std::fs::create_dir_all(&icons).map_err(|e| format!("创建图标目录失败：{e}"))?;

    let snap = settings::snapshot(app);
    let state = Arc::new(ClipboardAssistantState::new(app.clone(), conn, blobs, snap));
    commands::sweep_on_startup(&state)?;
    history::warm_cache(&state)?;

    app.manage(state.clone());

    if !is_session_locked(app) && settings::is_monitoring_enabled() {
        ingest::start_monitoring(app.clone(), state)?;
    }

    Ok(())
}

pub fn register_shortcut(app: &AppHandle) -> Result<(), String> {
    palette::register_shortcut(app)
}

pub fn refresh_runtime(app: &AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<Arc<ClipboardAssistantState>>() {
        state.reload_settings()?;
        settings::load_runtime_flags(app);
        if is_session_locked(app) {
            ingest::stop_monitoring();
        } else {
            ingest::restart_monitoring(app)?;
        }
    }
    Ok(())
}

/// 解锁后或设置变更时，由前端触发同步运行时（监听 / 快捷键链）。
pub fn sync_runtime(app: &AppHandle) -> Result<(), String> {
    refresh_runtime(app)?;
    crate::app::code_snippets::refresh_runtime(app)
}
