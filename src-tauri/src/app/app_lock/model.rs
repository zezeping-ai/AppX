use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLockSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_lock_on_startup")]
    pub lock_on_startup: bool,
}

impl Default for AppLockSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            lock_on_startup: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAppLockSettingsInput {
    pub enabled: bool,
    pub lock_on_startup: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLockSettingsView {
    pub enabled: bool,
    pub lock_on_startup: bool,
    pub session_locked: bool,
}

pub fn to_view(settings: &AppLockSettings, session_locked: bool) -> AppLockSettingsView {
    AppLockSettingsView {
        enabled: settings.enabled,
        lock_on_startup: settings.lock_on_startup,
        session_locked,
    }
}

fn default_lock_on_startup() -> bool {
    true
}
