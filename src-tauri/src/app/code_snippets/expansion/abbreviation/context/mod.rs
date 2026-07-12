//! 将 macOS context.rs 移入子模块，供跨平台 context 聚合。

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(all(unix, not(target_os = "macos")))]
mod linux_x11;

/// 读取焦点输入框光标前最多 `max_len` 个字符。
pub fn read_text_before_cursor(max_len: usize) -> Option<String> {
    read_text_before_cursor_impl(max_len)
}

#[cfg(target_os = "macos")]
fn read_text_before_cursor_impl(max_len: usize) -> Option<String> {
    macos::read_text_before_cursor(max_len)
}

#[cfg(target_os = "windows")]
fn read_text_before_cursor_impl(max_len: usize) -> Option<String> {
    windows::read_text_before_cursor(max_len)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_text_before_cursor_impl(max_len: usize) -> Option<String> {
    if crate::app::platform::is_x11_session() {
        linux_x11::read_text_before_cursor(max_len)
    } else {
        None
    }
}
