mod gif;
mod jpeg;
mod png;
mod quant;
mod svg;

use super::settings::ImageOptimSettings;

pub use gif::optimize_gif;
pub use jpeg::optimize_jpeg;
pub use png::optimize_png;
pub use svg::optimize_svg;

/// 单次优化运行时选项（由持久化设置派生）
#[derive(Debug, Clone)]
pub struct OptimizeOptions {
    pub strip_metadata: bool,
    /// 主要影响 PNG（oxipng preset）；SVG 高强度时选更激进 jobs
    pub effort: u8,
    pub lossy: bool,
    pub jpeg_max_quality: u8,
    pub png_min_quality: u8,
    pub gif_quality: u8,
}

impl OptimizeOptions {
    pub fn from_settings(settings: &ImageOptimSettings, lossy_override: Option<bool>) -> Self {
        let s = settings.clone().sanitized();
        Self {
            strip_metadata: s.strip_metadata,
            effort: s.effort,
            lossy: lossy_override.unwrap_or(s.lossy_enabled),
            jpeg_max_quality: s.jpeg_max_quality,
            png_min_quality: s.png_min_quality,
            gif_quality: s.gif_quality,
        }
    }
}

/// 取严格小于原文件的最小候选
pub fn pick_smallest(original: &[u8], candidates: impl IntoIterator<Item = Vec<u8>>) -> Option<Vec<u8>> {
    candidates
        .into_iter()
        .filter(|c| !c.is_empty() && c.len() < original.len())
        .min_by_key(|c| c.len())
}

/// 有候选则取最小；若曾失败且无更小结果则返回错误（避免误报「已最优」）
pub fn finish_candidates(
    original: &[u8],
    candidates: Vec<Vec<u8>>,
    last_err: Option<String>,
) -> Result<Option<Vec<u8>>, String> {
    if let Some(best) = pick_smallest(original, candidates) {
        return Ok(Some(best));
    }
    if let Some(err) = last_err {
        return Err(err);
    }
    Ok(None)
}
