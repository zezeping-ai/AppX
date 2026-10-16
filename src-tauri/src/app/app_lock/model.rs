use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLockSettings {
    #[serde(default)]
    pub enabled: bool,
    /// 应用冷启动时锁定
    #[serde(default = "default_lock_on_startup")]
    pub lock_on_startup: bool,
    /// 从托盘/隐藏状态再次显示主窗口时锁定
    #[serde(default)]
    pub lock_on_window_show: bool,
    /// 解锁 PIN 的 Argon2id PHC 哈希；空表示尚未设置（旧配置兼容）
    #[serde(default)]
    pub unlock_pin_hash: Option<String>,
}

impl Default for AppLockSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            lock_on_startup: true,
            lock_on_window_show: false,
            unlock_pin_hash: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAppLockSettingsInput {
    pub enabled: bool,
    pub lock_on_startup: bool,
    pub lock_on_window_show: bool,
    /// 设置或更换解锁 PIN；`None` 表示不改动已有哈希
    #[serde(default)]
    pub unlock_pin: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockSessionInput {
    pub pin: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppLockSettingsView {
    pub enabled: bool,
    pub lock_on_startup: bool,
    pub lock_on_window_show: bool,
    pub session_locked: bool,
    /// 是否已配置解锁 PIN（后端校验所需）
    pub has_unlock_pin: bool,
}

pub fn to_view(settings: &AppLockSettings, session_locked: bool) -> AppLockSettingsView {
    AppLockSettingsView {
        enabled: settings.enabled,
        lock_on_startup: settings.lock_on_startup,
        lock_on_window_show: settings.lock_on_window_show,
        session_locked,
        has_unlock_pin: settings
            .unlock_pin_hash
            .as_ref()
            .map(|h| !h.is_empty())
            .unwrap_or(false),
    }
}

fn default_lock_on_startup() -> bool {
    true
}
