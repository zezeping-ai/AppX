use tauri::{AppHandle, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};
use crate::app::code_snippets::{PaletteItem, SnippetRegistry};

use super::{copy_palette_item, hide_palette_window, insert_palette_item};

#[tauri::command]
pub fn code_snippets_list_palette_items(
    state: State<'_, AppLockSessionState>,
    registry: State<'_, SnippetRegistry>,
) -> Result<Vec<PaletteItem>, String> {
    ensure_unlocked(&state)?;
    Ok(registry.palette_items())
}

#[tauri::command]
pub fn code_snippets_hide_palette(app: AppHandle) -> Result<(), String> {
    hide_palette_window(&app)
}

#[tauri::command]
pub fn code_snippets_copy_palette_item(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    id: i64,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
    copy_palette_item(&app, id, &registry)
}

#[tauri::command]
pub fn code_snippets_insert_palette_item(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    id: i64,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
    insert_palette_item(&app, id, &registry)
}
