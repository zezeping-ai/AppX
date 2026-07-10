use sha2::{Digest, Sha256};

pub const KEY_LEN: usize = 32;

/// 应用级默认加密口令
pub const DEFAULT_PASSPHRASE: &str = "zezeping";

pub fn derive_key_from_passphrase(passphrase: &str) -> [u8; KEY_LEN] {
    let digest = Sha256::digest(passphrase.as_bytes());
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}
