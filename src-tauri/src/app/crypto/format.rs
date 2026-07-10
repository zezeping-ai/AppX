/// AppX 加密文件头布局：
/// [4B magic "APPX"][1B version][12B nonce][ciphertext + 16B GCM tag]
pub const MAGIC: &[u8; 4] = b"APPX";
pub const VERSION: u8 = 1;
pub const NONCE_LEN: usize = 12;
pub const HEADER_LEN: usize = MAGIC.len() + 1 + NONCE_LEN;
