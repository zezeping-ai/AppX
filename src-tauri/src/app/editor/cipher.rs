use crate::app::editor::settings::load_cipher_key;
use super::format::{self, HEADER_LEN, MAGIC, VERSION};
pub fn encrypt_bytes(app: &tauri::AppHandle, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::Rng;

    let key = load_cipher_key(app)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| format!("初始化加密器失败：{err}"))?;

    let mut nonce_bytes = [0u8; format::NONCE_LEN];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|err| format!("加密失败：{err}"))?;

    let mut output = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    output.extend_from_slice(MAGIC);
    output.push(VERSION);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_bytes(app: &tauri::AppHandle, payload: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    if payload.len() < HEADER_LEN {
        return Err("文件过短，不是有效的 AppX 加密格式".to_string());
    }
    if &payload[..4] != MAGIC {
        return Err("文件头无效，不是 AppX 加密格式".to_string());
    }
    if payload[4] != VERSION {
        return Err(format!("不支持的格式版本：{}", payload[4]));
    }

    let key = load_cipher_key(app)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|err| format!("初始化解密器失败：{err}"))?;
    let nonce = Nonce::from_slice(&payload[5..HEADER_LEN]);
    let ciphertext = &payload[HEADER_LEN..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "解密失败：文件损坏、密钥不匹配或不是由本应用创建".to_string())
}

pub fn read_file_content(app: &tauri::AppHandle, path: &std::path::Path) -> Result<String, String> {
    if format::is_encrypted_path(path) {
        let bytes = std::fs::read(path).map_err(|err| format!("读取文件失败：{err}"))?;
        let plain = decrypt_bytes(app, &bytes)?;
        return String::from_utf8(plain).map_err(|_| "解密内容不是有效 UTF-8 文本".to_string());
    }

    if !format::is_editable_path(path) {
        return Err(format!("无法在编辑器中打开：{}", path.display()));
    }

    std::fs::read_to_string(path).map_err(|err| format!("读取文件失败：{err}"))
}

pub fn write_file_content(
    app: &tauri::AppHandle,
    path: &std::path::Path,
    content: &str,
) -> Result<(), String> {
    format::ensure_writable_path(path)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("创建目录失败：{err}"))?;
    }

    if format::is_encrypted_path(path) {
        let encrypted = encrypt_bytes(app, content.as_bytes())?;
        std::fs::write(path, encrypted).map_err(|err| format!("写入文件失败：{err}"))?;
        return Ok(());
    }

    std::fs::write(path, content).map_err(|err| format!("写入文件失败：{err}"))
}
