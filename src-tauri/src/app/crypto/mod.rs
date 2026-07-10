//! AppX 公共加密能力（AES-256-GCM，口令经 SHA-256 派生密钥）

mod cipher;
pub mod commands;
mod format;
mod key;

pub use cipher::{decrypt_bytes_with_passphrase, encrypt_bytes_with_passphrase};
pub use key::DEFAULT_PASSPHRASE;
