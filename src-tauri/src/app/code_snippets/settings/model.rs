use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetSettings {
    /// 是否启用 `:缩写;` 全局展开
    #[serde(default = "default_inline_expansion_enabled")]
    pub inline_expansion_enabled: bool,
}

fn default_inline_expansion_enabled() -> bool {
    true
}

impl Default for CodeSnippetSettings {
    fn default() -> Self {
        Self {
            inline_expansion_enabled: default_inline_expansion_enabled(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippetSettingsView {
    pub inline_expansion_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCodeSnippetSettingsInput {
    pub inline_expansion_enabled: bool,
}

impl From<&CodeSnippetSettings> for CodeSnippetSettingsView {
    fn from(value: &CodeSnippetSettings) -> Self {
        Self {
            inline_expansion_enabled: value.inline_expansion_enabled,
        }
    }
}
