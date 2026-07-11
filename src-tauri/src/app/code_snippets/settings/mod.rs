mod model;
mod storage;

pub use model::*;
pub use storage::{read_code_snippet_settings, write_code_snippet_settings};

use std::sync::atomic::{AtomicBool, Ordering};

static FEATURES_ENABLED: AtomicBool = AtomicBool::new(true);
static INLINE_EXPANSION_ENABLED: AtomicBool = AtomicBool::new(true);
static SHORTCUTS_ENABLED: AtomicBool = AtomicBool::new(true);
static PALETTE_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn load_runtime_flags(app: &tauri::AppHandle) {
    let settings = read_code_snippet_settings(app).unwrap_or_default();
    apply_runtime_flags(&settings);
}

pub fn apply_runtime_flags(settings: &CodeSnippetSettings) {
    FEATURES_ENABLED.store(settings.enabled, Ordering::Relaxed);
    INLINE_EXPANSION_ENABLED.store(settings.inline_expansion_enabled, Ordering::Relaxed);
    SHORTCUTS_ENABLED.store(settings.shortcuts_enabled, Ordering::Relaxed);
    PALETTE_ENABLED.store(settings.palette_enabled, Ordering::Relaxed);
    crate::app::code_snippets::expansion::apply_expansion_trigger(&settings.inline_expansion_trigger);
}

pub fn is_inline_expansion_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed) && INLINE_EXPANSION_ENABLED.load(Ordering::Relaxed)
}

pub fn is_shortcuts_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed) && SHORTCUTS_ENABLED.load(Ordering::Relaxed)
}

pub fn is_palette_enabled() -> bool {
    FEATURES_ENABLED.load(Ordering::Relaxed) && PALETTE_ENABLED.load(Ordering::Relaxed)
}
