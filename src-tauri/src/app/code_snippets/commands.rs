use serde::Serialize;
use tauri::{AppHandle, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};
use crate::app::platform;

use super::expansion;
use super::model::{SnippetEntry, SnippetSyncItem};
use super::registry::SnippetRegistry;
use super::settings::{
    apply_runtime_flags, read_code_snippet_settings, write_code_snippet_settings,
    CodeSnippetSettingsView, SaveCodeSnippetSettingsInput,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontmostAppView {
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub is_own: bool,
}

#[tauri::command]
pub fn code_snippets_set_expansion_paused(paused: bool) {
    expansion::set_expansion_paused(paused);
}

/// 同步轻量索引（缩写 / 快捷键 → id）；正文触发时再查库。
#[tauri::command]
pub fn code_snippets_sync(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    snippets: Vec<SnippetSyncItem>,
    registry: State<'_, SnippetRegistry>,
) -> Result<(), String> {
    ensure_unlocked(&state)?;
    let mut entries = Vec::with_capacity(snippets.len());

    for item in snippets {
        let abbreviation = item.abbreviation.trim().to_lowercase();
        if abbreviation.is_empty() {
            continue;
        }

        let shortcut = item
            .shortcut
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        entries.push(SnippetEntry {
            id: item.id,
            name: item.name.trim().to_string(),
            abbreviation,
            shortcut,
            group: normalize_snippet_group(&item.group),
        });
    }

    registry.replace(entries);
    super::refresh_runtime(&app)?;
    Ok(())
}

#[tauri::command]
pub fn code_snippets_get_settings(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<CodeSnippetSettingsView, String> {
    ensure_unlocked(&state)?;
    Ok(CodeSnippetSettingsView::from(&read_code_snippet_settings(&app)?))
}

#[tauri::command]
pub fn code_snippets_save_settings(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    input: SaveCodeSnippetSettingsInput,
) -> Result<CodeSnippetSettingsView, String> {
    ensure_unlocked(&state)?;
    let mut settings = read_code_snippet_settings(&app)?;
    settings.enabled = input.enabled;
    settings.inline_expansion_enabled = input.inline_expansion_enabled;
    settings.inline_expansion_trigger = normalize_trigger_shortcut(&input.inline_expansion_trigger);
    settings.shortcuts_enabled = input.shortcuts_enabled;
    settings.palette_enabled = input.palette_enabled;
    write_code_snippet_settings(&app, &settings)?;
    apply_runtime_flags(&settings);
    super::refresh_runtime(&app)?;
    Ok(CodeSnippetSettingsView::from(&settings))
}

/// 表单「选择目标应用」：返回当前前台 App（含是否为本进程）。
#[tauri::command]
pub fn code_snippets_get_frontmost_app(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
) -> Result<FrontmostAppView, String> {
    ensure_unlocked(&state)?;
    let own_id = app.config().identifier.clone();
    let (app_id, app_name) = platform::frontmost_app();
    let is_own = platform::is_own_app_foreground(&own_id);
    Ok(FrontmostAppView {
        app_id,
        app_name,
        is_own,
    })
}

fn normalize_snippet_group(raw: &str) -> String {
    let group = raw.trim();
    if group.is_empty() {
        "general".to_string()
    } else {
        group.to_string()
    }
}

fn normalize_trigger_shortcut(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "F12".to_string();
    }
    if tauri_plugin_global_shortcut::Shortcut::try_from(trimmed).is_ok() {
        trimmed.to_string()
    } else {
        "F12".to_string()
    }
}
