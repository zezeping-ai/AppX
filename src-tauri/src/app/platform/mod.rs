//! 跨平台前台应用识别与 App 图标提取（Windows / Linux X11 / macOS）。

mod app_icon;
mod foreground;

pub use app_icon::fetch_icon_bytes;
pub use foreground::{frontmost_app, is_own_app_foreground};
#[cfg(all(unix, not(target_os = "macos")))]
pub use foreground::is_x11_session;
