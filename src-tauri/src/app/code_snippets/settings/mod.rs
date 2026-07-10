mod model;
mod storage;

pub use model::*;
pub use storage::{read_code_snippet_settings, write_code_snippet_settings};

use std::sync::atomic::{AtomicBool, Ordering};

static INLINE_EXPANSION_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn load_runtime_flags(app: &tauri::AppHandle) {
    let enabled = read_code_snippet_settings(app)
        .map(|settings| settings.inline_expansion_enabled)
        .unwrap_or(true);
    INLINE_EXPANSION_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn set_inline_expansion_enabled(enabled: bool) {
    INLINE_EXPANSION_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_inline_expansion_enabled() -> bool {
    INLINE_EXPANSION_ENABLED.load(Ordering::Relaxed)
}
