use tauri::{AppHandle, Manager};

use crate::app::app_lock::is_session_locked;
use crate::app::clipboard_assistant::{history, ingest, palette, settings};
use crate::app::runtime;

pub fn register_hooks() {
    runtime::register_on_locked(on_locked);
    runtime::register_on_unlocked(on_unlocked);
}

fn on_locked(app: &AppHandle) -> Result<(), String> {
    ingest::stop_monitoring();
    palette::hide_palette_window(app)?;
    if let Some(state) = app.try_state::<std::sync::Arc<super::ClipboardAssistantState>>() {
        if state.settings()?.clear_on_lock {
            history::clear_unpinned(state.inner())?;
        }
    }
    Ok(())
}

fn on_unlocked(app: &AppHandle) -> Result<(), String> {
    if !is_session_locked(app) && settings::is_monitoring_enabled() {
        ingest::restart_monitoring(app)?;
    }
    palette::ensure_window_ready(app);
    crate::app::code_snippets::refresh_runtime(app)
}
