use std::path::Path;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::access::ensure_usable;
use super::cleanup::{blob_dir_size, sweep_orphan_blobs};
use super::history::{clear_unpinned, get_content, list_items, toggle_pin};
use super::model::{
    ApplyItemInput, AssistantStatus, ListItemsQuery, ListItemsResult, MutateItemsInput, MutateOp,
    ApplyFormat,
};
use super::palette::save_palette_geometry;
use super::palette_geometry::PaletteGeometry;
use super::payload::apply_item;
use super::palette::hide_palette_window;
use super::settings::{
    apply_runtime_flags, is_monitoring_enabled, is_palette_enabled, normalize_palette_layout,
    write_settings, ClipboardAssistantSettings, SaveClipboardAssistantSettingsInput,
};
use super::state::ClipboardAssistantState;

#[tauri::command]
pub fn clipboard_assistant_list_items(
    app: AppHandle,
    state: State<'_, Arc<ClipboardAssistantState>>,
    query: ListItemsQuery,
) -> Result<ListItemsResult, String> {
    ensure_usable(&app)?;
    list_items(state.inner(), query)
}

#[tauri::command]
pub fn clipboard_assistant_apply_item(
    app: AppHandle,
    state: State<'_, Arc<ClipboardAssistantState>>,
    input: ApplyItemInput,
) -> Result<(), String> {
    ensure_usable(&app)?;
    apply_item(
        &app,
        state.inner(),
        input.id,
        input.action,
        input.format.unwrap_or(ApplyFormat::Plain),
    )
}

#[tauri::command]
pub fn clipboard_assistant_mutate_items(
    app: AppHandle,
    state: State<'_, Arc<ClipboardAssistantState>>,
    input: MutateItemsInput,
) -> Result<(), String> {
    ensure_usable(&app)?;
    let st = state.inner();
    match input.op {
        MutateOp::Pin => {
            for id in input.ids.unwrap_or_default() {
                let pinned: i64 = {
                    let conn = st.db.lock().map_err(|_| "数据库锁失败".to_string())?;
                    conn.query_row(
                        "SELECT pinned FROM clipboard_items WHERE id = ?1",
                        rusqlite::params![id],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap_or(0)
                };
                if pinned == 0 {
                    toggle_pin(st, id)?;
                }
            }
        }
        MutateOp::Unpin => {
            for id in input.ids.unwrap_or_default() {
                let pinned: i64 = {
                    let conn = st.db.lock().map_err(|_| "数据库锁失败".to_string())?;
                    conn.query_row(
                        "SELECT pinned FROM clipboard_items WHERE id = ?1",
                        rusqlite::params![id],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap_or(0)
                };
                if pinned == 1 {
                    toggle_pin(st, id)?;
                }
            }
        }
        MutateOp::Delete => {
            super::cleanup::remove_ids(st, &input.ids.unwrap_or_default())?;
        }
        MutateOp::ClearUnpinned => clear_unpinned(st)?,
    }
    Ok(())
}

#[tauri::command]
pub fn clipboard_assistant_get_content(
    app: AppHandle,
    state: State<'_, Arc<ClipboardAssistantState>>,
    id: i64,
) -> Result<super::model::GetContentResult, String> {
    ensure_usable(&app)?;
    get_content(state.inner(), id)
}

#[tauri::command]
pub fn clipboard_assistant_get_settings(
    state: State<'_, Arc<ClipboardAssistantState>>,
) -> Result<ClipboardAssistantSettings, String> {
    Ok((*state.inner().settings()?).clone())
}

#[tauri::command]
pub fn clipboard_assistant_save_settings(
    app: AppHandle,
    state: State<'_, Arc<ClipboardAssistantState>>,
    input: SaveClipboardAssistantSettingsInput,
) -> Result<(), String> {
    let previous = state.inner().settings()?.palette_layout.clone();
    let settings = ClipboardAssistantSettings {
        enabled: input.enabled,
        monitoring_enabled: input.monitoring_enabled,
        palette_enabled: input.palette_enabled,
        palette_shortcut: input.palette_shortcut,
        max_history_items: input.max_history_items,
        palette_layout: normalize_palette_layout(&input.palette_layout),
        palette_anchor: input.palette_anchor,
        palette_width: input.palette_width,
        palette_height: input.palette_height,
        palette_edge_margin: input.palette_edge_margin,
        remember_window_position: input.remember_window_position,
        auto_hide_on_paste: input.auto_hide_on_paste,
        auto_hide_on_click_outside: input.auto_hide_on_click_outside,
        open_search_on_show: input.open_search_on_show,
        dedupe_mode: input.dedupe_mode,
        palette_max_items: input.palette_max_items,
        show_source_app_icon: input.show_source_app_icon,
        auto_sweep_orphans_on_startup: input.auto_sweep_orphans_on_startup,
        text_inline_threshold: input.text_inline_threshold,
        max_text_bytes: input.max_text_bytes,
        max_image_blob_bytes: input.max_image_blob_bytes,
        max_image_blob_hard_bytes: input.max_image_blob_hard_bytes,
        compress_oversized_images: input.compress_oversized_images,
        excluded_apps: input.excluded_apps,
        clear_on_lock: input.clear_on_lock,
        copy_sound_enabled: input.copy_sound_enabled,
        paste_sound_enabled: input.paste_sound_enabled,
        copy_sound_path: input.copy_sound_path.trim().to_string(),
        paste_sound_path: input.paste_sound_path.trim().to_string(),
    };
    write_settings(&app, &settings)?;
    state.inner().reload_settings()?;
    apply_runtime_flags(&settings);
    if previous != settings.palette_layout {
        super::palette_geometry::invalidate_geometry_for_layout(&app, settings.palette_layout.as_str())?;
    }
    super::palette::refresh_palette_geometry(&app)?;
    super::sync_runtime(&app)?;
    Ok(())
}

#[tauri::command]
pub fn clipboard_assistant_save_palette_geometry(
    app: AppHandle,
    geometry: PaletteGeometry,
) -> Result<(), String> {
    save_palette_geometry(&app, geometry)
}

#[tauri::command]
pub fn clipboard_assistant_sync_runtime(app: AppHandle) -> Result<(), String> {
    super::sync_runtime(&app)
}

#[tauri::command]
pub fn clipboard_assistant_get_status(
    state: State<'_, Arc<ClipboardAssistantState>>,
) -> Result<AssistantStatus, String> {
    let st = state.inner();
    let settings = st.settings()?;
    let cache_revision = st
        .cache
        .read()
        .map(|cache| cache.revision())
        .unwrap_or(0);
    Ok(AssistantStatus {
        monitoring_active: is_monitoring_enabled(),
        palette_active: is_palette_enabled(),
        palette_shortcut: settings.palette_shortcut.clone(),
        total_count: st.total_count.load(std::sync::atomic::Ordering::Relaxed),
        unpinned_count: st.unpinned_count.load(std::sync::atomic::Ordering::Relaxed),
        pinned_count: st.pinned_count.load(std::sync::atomic::Ordering::Relaxed),
        blob_bytes: blob_dir_size(&st.blobs_dir),
        cache_revision,
    })
}

#[tauri::command]
pub fn clipboard_assistant_hide_palette(app: AppHandle) -> Result<(), String> {
    hide_palette_window(&app)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundFileData {
    pub mime_type: String,
    pub base64: String,
}

fn sound_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") | Some("aac") => "audio/mp4",
        Some("flac") => "audio/flac",
        _ => "audio/mpeg",
    }
}

#[tauri::command]
pub async fn clipboard_assistant_pick_sound_file(app: AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .set_title("选择音效文件")
        .add_filter("音频", &["mp3", "wav", "ogg", "m4a", "aac", "flac"])
        .blocking_pick_file();

    Ok(picked.map(|path| path.to_string()))
}

#[tauri::command]
pub fn clipboard_assistant_read_sound_file(path: String) -> Result<SoundFileData, String> {
    let path = Path::new(path.trim());
    if !path.is_file() {
        return Err("音效文件不存在".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("读取音效失败：{e}"))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("音效文件过大（上限 2MB）".into());
    }
    Ok(SoundFileData {
        mime_type: sound_mime(path).to_string(),
        base64: BASE64.encode(bytes),
    })
}

/// 试听或调试播放；`force` 忽略开关。
#[tauri::command]
pub fn clipboard_assistant_play_sound(
    app: AppHandle,
    kind: String,
    path: Option<String>,
    force: Option<bool>,
) -> Result<(), String> {
    let kind = super::sounds::SoundKind::parse(kind.trim())
        .ok_or_else(|| "无效音效类型".to_string())?;
    super::sounds::play(
        &app,
        kind,
        path.as_deref(),
        force.unwrap_or(false),
    );
    Ok(())
}

pub fn sweep_on_startup(state: &ClipboardAssistantState) -> Result<(), String> {
    if !state.settings()?.auto_sweep_orphans_on_startup {
        return Ok(());
    }
    let conn = state.db.lock().map_err(|_| "数据库锁失败".to_string())?;
    let _ = sweep_orphan_blobs(&state.blobs_dir, &conn);
    Ok(())
}
