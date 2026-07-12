use serde::{Deserialize, Serialize};

use crate::app::clipboard::rich::RichFormats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PayloadKind {
    Inline,
    Blob,
    FileRef,
}

impl PayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Blob => "blob",
            Self::FileRef => "file_ref",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "inline" => Some(Self::Inline),
            "blob" => Some(Self::Blob),
            "file_ref" => Some(Self::FileRef),
            _ => None,
        }
    }
}

/// 剪贴板读取后的待入库载荷（ingest 管道 → history 持久化）
#[derive(Debug, Clone)]
pub struct CapturedPayload {
    pub kind: PayloadKind,
    pub text: Option<String>,
    pub file_paths: Option<Vec<String>>,
    pub image_bytes: Option<Vec<u8>>,
    /// arboard 返回的原始 RGBA 像素尺寸（非 PNG/JPEG 文件头时使用）
    pub image_dimensions: Option<(u32, u32)>,
    pub rich_formats: Option<RichFormats>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentType {
    Text,
    Link,
    Image,
    File,
    Code,
    Color,
    Json,
}

impl ContentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Link => "link",
            Self::Image => "image",
            Self::File => "file",
            Self::Code => "code",
            Self::Color => "color",
            Self::Json => "json",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "text" => Some(Self::Text),
            "link" => Some(Self::Link),
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            "code" => Some(Self::Code),
            "color" => Some(Self::Color),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMeta {
    #[serde(default)]
    pub has_rich_format: bool,
}

impl PayloadMeta {
    pub fn parse(raw: &str) -> Self {
        serde_json::from_str(raw).unwrap_or_default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_rich(formats: Option<&RichFormats>) -> Self {
        Self {
            has_rich_format: formats.is_some_and(|value| value.has_content()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentBadge {
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub content_type: ContentType,
    pub preview: String,
    pub source_app_bundle: Option<String>,
    pub source_app_name: Option<String>,
    pub source_app_icon_url: Option<String>,
    pub group_key: String,
    pub pinned: bool,
    pub created_at: String,
    pub accent_color: String,
    pub char_count: Option<i64>,
    pub tags: Vec<String>,
    pub badges: Vec<ContentBadge>,
    pub thumb_url: Option<String>,
    pub relative_time: String,
    pub has_rich_format: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItemsQuery {
    pub keyword: Option<String>,
    pub content_type: Option<ContentType>,
    pub group_key: Option<String>,
    pub source_app_bundle: Option<String>,
    pub pinned_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub prefer_cache: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplyAction {
    Paste,
    Copy,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MutateOp {
    Pin,
    Unpin,
    Delete,
    ClearUnpinned,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItemsResult {
    pub items: Vec<ItemSummary>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContentResult {
    pub content_type: ContentType,
    pub payload_kind: PayloadKind,
    pub text: Option<String>,
    pub file_paths: Option<Vec<String>>,
    pub has_blob: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ApplyFormat {
    #[default]
    Plain,
    Rich,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyItemInput {
    pub id: i64,
    pub action: ApplyAction,
    #[serde(default)]
    pub format: Option<ApplyFormat>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutateItemsInput {
    pub op: MutateOp,
    pub ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantStatus {
    pub monitoring_active: bool,
    pub palette_active: bool,
    pub palette_shortcut: String,
    pub total_count: u64,
    pub unpinned_count: u64,
    pub pinned_count: u64,
    pub blob_bytes: u64,
    pub cache_revision: u64,
}
