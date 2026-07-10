//! AppX 专有加密文本格式：`.txt.x`（默认口令）、`.txt.x0`（独立口令）
//!
//! 文件布局：
//! [4B magic "APPX"][1B version][12B nonce][ciphertext + 16B GCM tag]
//! 密钥由加密口令经 SHA-256 派生。

mod cipher;
mod commands;
mod convert;
mod format;
mod passphrase_store;
mod settings;
mod tree;

pub use commands::*;
pub use passphrase_store::FilePassphraseStore;
pub use settings::*;
