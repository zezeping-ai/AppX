//! AppX 专有加密文本格式：`.txt.x`（默认口令）、`.txt.x0`（独立口令）
//!
//! 加密算法见 `app::crypto`。

mod cipher;
mod commands;
mod convert;
mod format;
mod open_paths;
mod passphrase_store;
mod search;
mod settings;
mod tree;

pub use commands::*;
pub use open_paths::{enqueue_open_paths, paths_from_args, PendingOpenPaths};
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
pub use open_paths::paths_from_urls;
pub use passphrase_store::FilePassphraseStore;
pub use settings::*;
