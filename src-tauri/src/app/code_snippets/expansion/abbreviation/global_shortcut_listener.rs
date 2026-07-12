//! Windows / Linux X11：通过 global-shortcut 监听展开触发键。

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::app::app_lock::is_session_locked;
use crate::app::code_snippets::settings::{is_inline_expansion_enabled, read_code_snippet_settings};
use crate::app::platform;

use super::{
    set_listener_active, try_expand_on_trigger,
};
use super::trigger::current_trigger_shortcut;

pub fn start_listener(app: AppHandle) -> Result<(), String> {
    refresh_trigger(&app)
}

pub fn refresh_trigger(app: &AppHandle) -> Result<(), String> {
    let settings = read_code_snippet_settings(app).unwrap_or_default();
    if !settings.enabled || !settings.inline_expansion_enabled {
        set_listener_active(false);
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    if !platform::is_x11_session() {
        log::warn!("[code_snippets] :abbrev expansion requires Linux X11 session");
        set_listener_active(false);
        return Ok(());
    }

    let trigger = current_trigger_shortcut();
    let parsed = Shortcut::try_from(trigger.as_str())
        .map_err(|err| format!("展开触发键 `{trigger}` 无效：{err}"))?;
    let own_app_id = app.config().identifier.clone();

    app.global_shortcut()
        .on_shortcut(parsed, move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            if is_session_locked(app) || !is_inline_expansion_enabled() {
                return;
            }
            if platform::is_own_app_foreground(&own_app_id) {
                return;
            }
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                try_expand_on_trigger(app);
            }));
        })
        .map_err(|err| format!("注册展开触发键 `{trigger}` 失败：{err}"))?;

    set_listener_active(true);
    log::info!("[code_snippets] global shortcut listener active (`{trigger}`)");
    Ok(())
}
