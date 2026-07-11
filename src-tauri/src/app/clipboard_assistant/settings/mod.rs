use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::AppHandle;

use crate::app::settings::storage;

pub mod model;

pub use model::{normalize_palette_layout, ClipboardAssistantSettings, SaveClipboardAssistantSettingsInput};

pub const SETTINGS_FILE: &str = "clipboard-assistant-settings.json";
const CONTEXT: &str = "剪切助手设置";

static FEATURES_ENABLED: AtomicBool = AtomicBool::new(true);
static MONITORING_ENABLED: AtomicBool = AtomicBool::new(true);
static PALETTE_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn read_settings(app: &AppHandle) -> Result<ClipboardAssistantSettings, String> {
    let mut settings: ClipboardAssistantSettings =
        storage::read_json_settings(app, SETTINGS_FILE, CONTEXT)?.unwrap_or_default();
    settings.palette_layout = model::normalize_palette_layout(&settings.palette_layout);
    Ok(settings)
}

pub fn write_settings(app: &AppHandle, settings: &ClipboardAssistantSettings) -> Result<(), String> {
    storage::write_json_settings(app, SETTINGS_FILE, settings, CONTEXT)
}

pub fn load_runtime_flags(app: &AppHandle) {
    apply_runtime_flags(&read_settings(app).unwrap_or_default());
}

pub fn apply_runtime_flags(settings: &ClipboardAssistantSettings) {
    FEATURES_ENABLED.store(settings.enabled, Ordering::Relaxed);
    MONITORING_ENABLED.store(settings.monitoring_enabled, Ordering::Relaxed);
    PALETTE_ENABLED.store(settings.palette_enabled, Ordering::Relaxed);
}

pub fn is_features_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed)
}

pub fn is_monitoring_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed) && MONITORING_ENABLED.load(Ordering::Relaxed)
}

pub fn is_palette_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed) && PALETTE_ENABLED.load(Ordering::Relaxed)
}

pub type SettingsSnapshot = Arc<ClipboardAssistantSettings>;

pub fn snapshot(app: &AppHandle) -> SettingsSnapshot {
    Arc::new(read_settings(app).unwrap_or_default())
}
