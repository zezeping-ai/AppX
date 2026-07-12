//! 系统剪贴板唯一入口；剪切助手历史监听在此挂接。

pub mod rich;
pub mod image;
pub mod files;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use arboard::Clipboard;

static RECORD_SUPPRESSED: AtomicBool = AtomicBool::new(false);
static PASTEBOARD_LOCK: Mutex<()> = Mutex::new(());

/// macOS `NSPasteboard` is not thread-safe; serialize every read/write.
pub fn with_pasteboard_lock<T>(f: impl FnOnce() -> T) -> T {
    let guard = PASTEBOARD_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let result = f();
    drop(guard);
    result
}

pub fn is_record_suppressed() -> bool {
    RECORD_SUPPRESSED.load(Ordering::Relaxed)
}

pub fn with_record_suppressed<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    RECORD_SUPPRESSED.store(true, Ordering::Relaxed);
    let result = f();
    RECORD_SUPPRESSED.store(false, Ordering::Relaxed);
    result
}

/// 用户主动拷贝：写入剪贴板（抑制历史回写）。
pub fn set_text_persist(text: &str) -> Result<(), String> {
    with_pasteboard_lock(|| {
        with_record_suppressed(|| {
            Clipboard::new()
                .map_err(|err| format!("无法访问剪贴板：{err}"))?
                .set_text(text.to_string())
                .map_err(|err| format!("写入剪贴板失败：{err}"))
        })
    })
}

/// 插入时的临时写入：返回原剪贴板内容供恢复。
pub fn set_text_transient(text: &str) -> Result<Option<String>, String> {
    with_pasteboard_lock(|| {
        with_record_suppressed(|| {
            let mut clipboard =
                Clipboard::new().map_err(|err| format!("无法访问剪贴板：{err}"))?;
            let original = clipboard.get_text().ok();
            clipboard
                .set_text(text.to_string())
                .map_err(|err| format!("写入剪贴板失败：{err}"))?;
            Ok(original)
        })
    })
}

/// 恢复剪贴板内容（插入副作用结束后调用）。
pub fn restore_text(text: &str) {
    let _ = with_pasteboard_lock(|| {
        with_record_suppressed(|| {
            Clipboard::new()
                .map_err(|err| format!("无法访问剪贴板：{err}"))?
                .set_text(text.to_string())
                .map_err(|err| format!("写入剪贴板失败：{err}"))
        })
    });
}
