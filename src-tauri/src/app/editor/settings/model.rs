use serde::{Deserialize, Serialize};

pub const DEFAULT_PASSPHRASE: &str = "zezeping";

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
    #[serde(default = "default_passphrase")]
    pub passphrase: String,
}

impl Default for EditorEncryptionSettings {
    fn default() -> Self {
        Self {
            passphrase: default_passphrase(),
        }
    }
}

fn default_passphrase() -> String {
    DEFAULT_PASSPHRASE.to_string()
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
}

/// 旧版 `settings.json` 结构，仅用于迁移
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LegacyAppSettings {
    #[serde(default = "default_passphrase")]
    pub(super) encryption_passphrase: String,
}
