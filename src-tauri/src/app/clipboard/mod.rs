//! 系统剪贴板唯一入口；剪切助手历史监听在此挂接。

pub mod rich;
pub mod image;
pub mod files;
pub mod transient;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use arboard::Clipboard;

/// 嵌套安全：临时粘贴会跨多次写剪贴板，深度归零才恢复监听。
static RECORD_SUPPRESS_DEPTH: AtomicU32 = AtomicU32::new(0);
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
    RECORD_SUPPRESS_DEPTH.load(Ordering::Relaxed) > 0
}

fn suppress_recording_enter() {
    RECORD_SUPPRESS_DEPTH.fetch_add(1, Ordering::Relaxed);
}

fn suppress_recording_leave() {
    let prev = RECORD_SUPPRESS_DEPTH.fetch_sub(1, Ordering::Relaxed);
    debug_assert!(prev > 0, "record suppress depth underflow");
}

/// RAII：持有期间剪切助手不入库（可嵌套）。
pub struct RecordSuppressGuard;

impl Drop for RecordSuppressGuard {
    fn drop(&mut self) {
        suppress_recording_leave();
    }
}

pub fn suppress_recording() -> RecordSuppressGuard {
    suppress_recording_enter();
    RecordSuppressGuard
}

pub fn with_record_suppressed<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = suppress_recording();
    f()
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
