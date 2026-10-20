use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptimSettings {
    /// 剥离 EXIF / 注释等可选元数据（默认开）
    #[serde(default = "default_true")]
    pub strip_metadata: bool,
    /// 压缩强度 0–6，越高越慢越小（默认 2）
    #[serde(default = "default_effort")]
    pub effort: u8,
    /// 有损压缩总开关（默认关）
    #[serde(default)]
    pub lossy_enabled: bool,
    /// JPEG 有损质量上限；100 表示不走有损重编码
    #[serde(default = "default_quality")]
    pub jpeg_max_quality: u8,
    /// PNG 有损调色板质量下限（映射为色数/采样强度）
    #[serde(default = "default_quality")]
    pub png_min_quality: u8,
    /// GIF 有损质量
    #[serde(default = "default_quality")]
    pub gif_quality: u8,
}

fn default_true() -> bool {
    true
}

fn default_effort() -> u8 {
    2
}

fn default_quality() -> u8 {
    80
}

impl Default for ImageOptimSettings {
    fn default() -> Self {
        Self {
            strip_metadata: default_true(),
            effort: default_effort(),
            lossy_enabled: false,
            jpeg_max_quality: default_quality(),
            png_min_quality: default_quality(),
            gif_quality: default_quality(),
        }
    }
}

impl ImageOptimSettings {
    pub fn sanitized(mut self) -> Self {
        self.effort = self.effort.min(6);
        self.jpeg_max_quality = self.jpeg_max_quality.clamp(40, 100);
        self.png_min_quality = self.png_min_quality.clamp(40, 100);
        self.gif_quality = self.gif_quality.clamp(40, 100);
        self
    }
}
