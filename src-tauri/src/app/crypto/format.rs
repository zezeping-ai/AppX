/// AppX 加密文件头布局
///
/// v1: `[4B magic "APPX"][1B version=1][12B nonce][ciphertext + 16B GCM tag]`
///     密钥 = SHA-256(passphrase)（无盐，兼容旧文件）
///
/// v2: `[4B magic "APPX"][1B version=2][16B salt][12B nonce][ciphertext + 16B GCM tag]`
///     密钥 = Argon2id(passphrase, salt)
pub const MAGIC: &[u8; 4] = b"APPX";
pub const VERSION_V1: u8 = 1;
/// 新加密统一写 v2
pub const VERSION_V2: u8 = 2;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const HEADER_LEN_V1: usize = MAGIC.len() + 1 + NONCE_LEN;
pub const HEADER_LEN_V2: usize = MAGIC.len() + 1 + SALT_LEN + NONCE_LEN;
