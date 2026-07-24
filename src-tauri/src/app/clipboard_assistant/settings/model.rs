use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardAssistantSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub monitoring_enabled: bool,
    #[serde(default = "default_true")]
    pub palette_enabled: bool,
    #[serde(default = "default_palette_shortcut")]
    pub palette_shortcut: String,
    #[serde(default = "default_max_history")]
    pub max_history_items: u32,
    #[serde(default = "default_palette_layout")]
    pub palette_layout: String,
    #[serde(default = "default_palette_anchor")]
    pub palette_anchor: String,
    #[serde(default = "default_u32_960")]
    pub palette_width: u32,
    #[serde(default = "default_u32_320")]
    pub palette_height: u32,
    #[serde(default = "default_u32_16")]
    pub palette_edge_margin: u32,
    #[serde(default = "default_true")]
    pub remember_window_position: bool,
    #[serde(default = "default_true")]
    pub auto_hide_on_paste: bool,
    #[serde(default = "default_true")]
    pub auto_hide_on_click_outside: bool,
    #[serde(default = "default_true")]
    pub open_search_on_show: bool,
    #[serde(default = "default_dedupe")]
    pub dedupe_mode: String,
    #[serde(default = "default_u32_500")]
    pub palette_max_items: u32,
    #[serde(default = "default_true")]
    pub show_source_app_icon: bool,
    #[serde(default = "default_true")]
    pub auto_sweep_orphans_on_startup: bool,
    #[serde(default = "default_text_inline_threshold")]
    pub text_inline_threshold: u32,
    #[serde(default = "default_max_text_bytes")]
    pub max_text_bytes: u32,
    #[serde(default = "default_max_image_blob_bytes")]
    pub max_image_blob_bytes: u32,
    #[serde(default = "default_max_image_blob_hard_bytes")]
    pub max_image_blob_hard_bytes: u32,
    #[serde(default = "default_true")]
    pub compress_oversized_images: bool,
    #[serde(default)]
    pub excluded_apps: Vec<String>,
    #[serde(default)]
    pub clear_on_lock: bool,
    #[serde(default = "default_true")]
    pub copy_sound_enabled: bool,
    #[serde(default = "default_true")]
    pub paste_sound_enabled: bool,
    /// 空字符串表示使用内置默认音效
    #[serde(default)]
    pub copy_sound_path: String,
    /// 空字符串表示使用内置默认音效
    #[serde(default)]
    pub paste_sound_path: String,
}

fn default_true() -> bool {
    true
}
fn default_palette_shortcut() -> String {
    "CommandOrControl+Shift+V".to_string()
}
fn default_max_history() -> u32 {
    500
}
fn default_palette_layout() -> String {
    "bottomPanel".to_string()
}

/// 兼容旧版布局值（bottomStrip / bottomList → bottomPanel）
pub fn normalize_palette_layout(layout: &str) -> String {
    match layout {
        "bottomStrip" | "bottomList" => "bottomPanel".to_string(),
        "topPanel" | "bottomPanel" | "leftPanel" | "rightPanel" => layout.to_string(),
        _ => "bottomPanel".to_string(),
    }
}
fn default_palette_anchor() -> String {
    "activeMonitor".to_string()
}
fn default_u32_960() -> u32 {
    960
}
fn default_u32_320() -> u32 {
    320
}
fn default_u32_16() -> u32 {
    16
}
fn default_u32_500() -> u32 {
    500
}
fn default_dedupe() -> String {
    "consecutive".to_string()
}
fn default_text_inline_threshold() -> u32 {
    64 * 1024
}
fn default_max_text_bytes() -> u32 {
    5 * 1024 * 1024
}
fn default_max_image_blob_bytes() -> u32 {
    30 * 1024 * 1024
}
fn default_max_image_blob_hard_bytes() -> u32 {
    100 * 1024 * 1024
}

impl Default for ClipboardAssistantSettings {
    fn default() -> Self {
        serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveClipboardAssistantSettingsInput {
    pub enabled: bool,
    pub monitoring_enabled: bool,
    pub palette_enabled: bool,
    pub palette_shortcut: String,
    pub max_history_items: u32,
    pub palette_layout: String,
    pub palette_anchor: String,
    pub palette_width: u32,
    pub palette_height: u32,
    pub palette_edge_margin: u32,
    pub remember_window_position: bool,
    pub auto_hide_on_paste: bool,
    pub auto_hide_on_click_outside: bool,
    pub open_search_on_show: bool,
    pub dedupe_mode: String,
    pub palette_max_items: u32,
    pub show_source_app_icon: bool,
    pub auto_sweep_orphans_on_startup: bool,
    pub text_inline_threshold: u32,
    pub max_text_bytes: u32,
    pub max_image_blob_bytes: u32,
    pub max_image_blob_hard_bytes: u32,
    pub compress_oversized_images: bool,
    pub excluded_apps: Vec<String>,
    pub clear_on_lock: bool,
    pub copy_sound_enabled: bool,
    pub paste_sound_enabled: bool,
    pub copy_sound_path: String,
    pub paste_sound_path: String,
}
