//! AppX 专有加密文本格式 `.txt.x`
//!
//! 文件布局：
//! [4B magic "APPX"][1B version][12B nonce][ciphertext + 16B GCM tag]
//! 密钥由设置中的加密口令派生（SHA-256）。

mod cipher;
mod commands;
mod convert;
mod format;
mod settings;
mod tree;

pub use commands::*;
pub use settings::*;
