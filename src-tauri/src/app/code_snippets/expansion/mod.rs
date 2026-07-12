mod abbreviation;
mod shortcut;

use std::sync::OnceLock;

use tauri::AppHandle;

static STARTED: OnceLock<()> = OnceLock::new();

pub fn start(app: &AppHandle) -> Result<(), String> {
    if STARTED.set(()).is_ok() {
        abbreviation::start_listener(app.clone())?;
    }
    Ok(())
}

pub fn refresh_shortcuts(app: &AppHandle) -> Result<(), String> {
    shortcut::refresh_shortcuts(app)?;
    abbreviation::refresh_trigger(app)
}

pub fn set_expansion_paused(paused: bool) {
    abbreviation::set_expansion_paused(paused);
}

pub fn apply_expansion_trigger(shortcut: &str) {
    abbreviation::trigger::apply_trigger_shortcut(shortcut);
}

pub fn is_abbreviation_listener_active() -> bool {
    abbreviation::is_listener_active()
}
