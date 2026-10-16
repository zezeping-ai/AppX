//! AppX 公共加密能力（AES-256-GCM；新文件 Argon2id 派生，兼容旧版 SHA-256）

mod cipher;
pub mod commands;
mod format;
mod key;

pub use cipher::{decrypt_bytes_with_passphrase, encrypt_bytes_with_passphrase};
pub use key::DEFAULT_PASSPHRASE;
