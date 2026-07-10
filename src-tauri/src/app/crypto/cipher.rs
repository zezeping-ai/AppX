use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

use super::format;
use super::key::derive_key_from_passphrase;

pub fn encrypt_bytes_with_passphrase(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = derive_key_from_passphrase(passphrase);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| format!("初始化加密器失败：{err}"))?;

    let mut nonce_bytes = [0u8; format::NONCE_LEN];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|err| format!("加密失败：{err}"))?;

    let mut output = Vec::with_capacity(format::HEADER_LEN + ciphertext.len());
    output.extend_from_slice(format::MAGIC);
    output.push(format::VERSION);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_bytes_with_passphrase(passphrase: &str, payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.len() < format::HEADER_LEN {
        return Err("文件过短，不是有效的 AppX 加密格式".to_string());
    }
    if &payload[..4] != format::MAGIC {
        return Err("文件头无效，不是 AppX 加密格式".to_string());
    }
    if payload[4] != format::VERSION {
        return Err(format!("不支持的格式版本：{}", payload[4]));
    }

    let key = derive_key_from_passphrase(passphrase);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| format!("初始化解密器失败：{err}"))?;
    let nonce = Nonce::from_slice(&payload[5..format::HEADER_LEN]);
    let ciphertext = &payload[format::HEADER_LEN..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "解密失败：文件损坏、密钥不匹配或不是由本应用创建".to_string())
}
