use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettings {
    #[serde(default)]
    pub encryption: EditorEncryptionSettings,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            encryption: EditorEncryptionSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorEncryptionSettings {
    /// 空字符串表示未单独配置，回退「偏好设置 › 安全」中的默认加密口令
    #[serde(default)]
    pub passphrase: String,
}

impl Default for EditorEncryptionSettings {
    fn default() -> Self {
        Self {
            passphrase: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettingsView {
    pub encryption: EditorEncryptionView,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorEncryptionView {
    pub passphrase: String,
    pub uses_global_passphrase: bool,
}

/// 旧版 `settings.json` 结构，仅用于迁移
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LegacyAppSettings {
    #[serde(default)]
    pub(super) encryption_passphrase: String,
}
