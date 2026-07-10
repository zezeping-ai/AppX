use super::model::SecuritySettingsView;
use super::storage::read_security_settings;

/// 读取「偏好设置 › 安全」中的默认加密口令
pub fn load_default_passphrase(app: &tauri::AppHandle) -> Result<String, String> {
    let settings = read_security_settings(app)?;
    let trimmed = settings.default_encryption_passphrase.trim();
    if trimmed.is_empty() {
        return Err("默认加密口令未配置".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn security_settings_view(app: &tauri::AppHandle) -> Result<SecuritySettingsView, String> {
    let settings = read_security_settings(app)?;
    Ok(SecuritySettingsView::from_settings(&settings))
}
