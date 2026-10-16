use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::app_lock::is_session_locked;
use crate::app::code_snippets::{is_shortcuts_enabled, registry::SnippetRegistry};
use crate::app::text_delivery::insert_at_focus;

/// 仅注册本域 snippet 快捷键；清空全部快捷键由 `global_shortcut::refresh_all` 统一负责。
pub fn refresh_shortcuts(app: &AppHandle) -> Result<(), String> {
    let Some(registry) = app.try_state::<SnippetRegistry>() else {
        return Ok(());
    };

    if !is_shortcuts_enabled() {
        return Ok(());
    }

    let shortcut_api = app.global_shortcut();
    for (shortcut_text, entry) in registry.snapshot().shortcuts {
        let parsed = parse_shortcut(&shortcut_text)?;
        let content = Arc::new(entry.content);
        let handler_content = content.clone();

        shortcut_api
            .on_shortcut(parsed, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if is_session_locked(app) || !is_shortcuts_enabled() {
                        return;
                    }
                    insert_at_focus(&handler_content);
                }
            })
            .map_err(|err| format!("注册快捷键 `{shortcut_text}` 失败：{err}"))?;
    }

    Ok(())
}

fn parse_shortcut(raw: &str) -> Result<Shortcut, String> {
    Shortcut::try_from(raw).map_err(|err| format!("快捷键 `{raw}` 无效：{err}"))
}
