use argon2::Argon2;
use sha2::{Digest, Sha256};

pub const KEY_LEN: usize = 32;

/// 应用级默认加密口令
pub const DEFAULT_PASSPHRASE: &str = "zezeping";

/// v1 兼容：无盐 SHA-256（仅解密旧文件）
pub fn derive_key_sha256(passphrase: &str) -> [u8; KEY_LEN] {
    let digest = Sha256::digest(passphrase.as_bytes());
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}

/// v2：Argon2id 派生 32 字节密钥（salt 由调用方提供/生成）
pub fn derive_key_argon2(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    if salt.len() < 8 {
        return Err("盐值过短".to_string());
    }
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|err| format!("密钥派生失败：{err}"))?;
    Ok(key)
}
