use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetSettings {
    /// 总开关：同时控制缩写展开、全局快捷键与快捷键命令面板
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 是否启用 `:缩写` + 触发键 全局展开
    #[serde(default = "default_inline_expansion_enabled")]
    pub inline_expansion_enabled: bool,
    /// 缩写展开触发快捷键（Tauri global-shortcut 格式，默认 F12）
    #[serde(default = "default_inline_expansion_trigger")]
    pub inline_expansion_trigger: String,
    /// 是否启用各代码段配置的全局快捷键
    #[serde(default = "default_shortcuts_enabled")]
    pub shortcuts_enabled: bool,
    /// 是否启用快捷键命令面板
    #[serde(default = "default_palette_enabled")]
    pub palette_enabled: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_inline_expansion_enabled() -> bool {
    true
}

fn default_shortcuts_enabled() -> bool {
    true
}

fn default_palette_enabled() -> bool {
    true
}

fn default_inline_expansion_trigger() -> String {
    "F12".to_string()
}

impl Default for CodeSnippetSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            inline_expansion_enabled: default_inline_expansion_enabled(),
            inline_expansion_trigger: default_inline_expansion_trigger(),
            shortcuts_enabled: default_shortcuts_enabled(),
            palette_enabled: default_palette_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetSettingsView {
    pub enabled: bool,
    pub inline_expansion_enabled: bool,
    pub inline_expansion_trigger: String,
    pub shortcuts_enabled: bool,
    pub palette_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCodeSnippetSettingsInput {
    pub enabled: bool,
    pub inline_expansion_enabled: bool,
    pub inline_expansion_trigger: String,
    pub shortcuts_enabled: bool,
    pub palette_enabled: bool,
}

impl From<&CodeSnippetSettings> for CodeSnippetSettingsView {
    fn from(value: &CodeSnippetSettings) -> Self {
        Self {
            enabled: value.enabled,
            inline_expansion_enabled: value.inline_expansion_enabled,
            inline_expansion_trigger: value.inline_expansion_trigger.clone(),
            shortcuts_enabled: value.shortcuts_enabled,
            palette_enabled: value.palette_enabled,
        }
    }
}
