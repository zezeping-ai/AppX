use serde::{Deserialize, Serialize};

use crate::app::crypto::DEFAULT_PASSPHRASE;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    #[serde(default = "default_encryption_passphrase")]
    pub default_encryption_passphrase: String,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            default_encryption_passphrase: default_encryption_passphrase(),
        }
    }
}

fn default_encryption_passphrase() -> String {
    DEFAULT_PASSPHRASE.to_string()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettingsView {
    pub default_encryption_passphrase: String,
    /// 全局口令是否允许修改（当前固定为 false）
    pub default_encryption_passphrase_editable: bool,
}

impl SecuritySettingsView {
    pub fn from_settings(settings: &SecuritySettings) -> Self {
        Self {
            default_encryption_passphrase: settings.default_encryption_passphrase.clone(),
            default_encryption_passphrase_editable: false,
        }
    }
}
