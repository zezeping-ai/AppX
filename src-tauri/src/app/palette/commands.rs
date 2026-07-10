use tauri::{AppHandle, State};

use crate::app::code_snippets::{PaletteItem, SnippetRegistry};

use super::{copy_palette_item, hide_palette_window, insert_palette_item};

#[tauri::command]
pub fn code_snippets_list_palette_items(
    registry: State<'_, SnippetRegistry>,
) -> Vec<PaletteItem> {
    registry.palette_items()
}

#[tauri::command]
pub fn code_snippets_hide_palette(app: AppHandle) -> Result<(), String> {
    hide_palette_window(&app)
}

#[tauri::command]
pub fn code_snippets_copy_palette_item(
    app: AppHandle,
    id: i64,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    copy_palette_item(&app, id, &registry)
}

#[tauri::command]
pub fn code_snippets_insert_palette_item(
    app: AppHandle,
    id: i64,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    insert_palette_item(&app, id, &registry)
}
