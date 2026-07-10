use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetSyncItem {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub shortcut: Option<String>,
    /// Base64(AppX AES-GCM 密文)
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SnippetEntry {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub shortcut: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteItem {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
}
