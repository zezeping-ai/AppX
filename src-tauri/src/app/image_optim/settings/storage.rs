use super::model::ImageOptimSettings;
use crate::app::settings::storage;

const SETTINGS_FILE: &str = "image_optim.json";
const CONTEXT: &str = "图片优化设置";

pub fn read_image_optim_settings(app: &tauri::AppHandle) -> Result<ImageOptimSettings, String> {
    Ok(
        storage::read_json_settings::<ImageOptimSettings>(app, SETTINGS_FILE, CONTEXT)?
            .unwrap_or_default()
            .sanitized(),
    )
}

pub fn write_image_optim_settings(
    app: &tauri::AppHandle,
    settings: &ImageOptimSettings,
) -> Result<(), String> {
    storage::write_json_settings(app, SETTINGS_FILE, settings, CONTEXT)
}
