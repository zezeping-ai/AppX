use serde::{Deserialize, Serialize};

/// 同步到 Rust 的轻量索引（不含密文内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetSyncItem {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub shortcut: Option<String>,
    #[serde(default = "default_snippet_group")]
    pub group: String,
}

fn default_snippet_group() -> String {
    "general".to_string()
}

/// 内存索引条目：仅 id / 元数据，正文触发时再查库解密
#[derive(Debug, Clone)]
pub struct SnippetEntry {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub shortcut: Option<String>,
    pub group: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteItem {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub group: String,
}

/// DB 中 `app_variants` JSON 元素（匹配只用 app_id）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppVariantRow {
    pub app_id: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SnippetStoredPayload {
    pub content: String,
    pub variants: Vec<AppVariantRow>,
}
