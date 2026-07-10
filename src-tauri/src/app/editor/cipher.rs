use crate::app::editor::passphrase_store::FilePassphraseStore;
use crate::app::editor::settings::{derive_key_from_passphrase, load_passphrase};
use super::format::{self, EncryptionKind};

pub const ERR_DECRYPT_PASSPHRASE_REQUIRED: &str = "DECRYPT_PASSPHRASE_REQUIRED";

pub fn decrypt_passphrase_required(path: &std::path::Path) -> String {
    format!("{ERR_DECRYPT_PASSPHRASE_REQUIRED}:{}", path.display())
}

pub fn encrypt_bytes_with_passphrase(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::Rng;

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
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

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

fn utf8_from_bytes(plain: Vec<u8>) -> Result<String, String> {
    String::from_utf8(plain).map_err(|_| "解密内容不是有效 UTF-8 文本".to_string())
}

fn read_payload_bytes(path: &std::path::Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|err| format!("读取文件失败：{err}"))
}

fn passphrase_for_kind(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &std::path::Path,
    kind: EncryptionKind,
) -> Result<String, String> {
    match kind {
        EncryptionKind::Default => load_passphrase(app),
        EncryptionKind::Custom => store
            .get(&super::tree::path_to_string(path))
            .ok_or_else(|| decrypt_passphrase_required(path)),
    }
}

pub fn try_decrypt_with_passphrase(
    path: &std::path::Path,
    passphrase: &str,
) -> Result<Vec<u8>, String> {
    let bytes = read_payload_bytes(path)?;
    decrypt_bytes_with_passphrase(passphrase, &bytes)
}

pub fn read_encrypted_payload(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &std::path::Path,
) -> Result<String, String> {
    let kind = format::encryption_kind(path)
        .ok_or_else(|| format!("不是加密文件：{}", path.display()))?;
    let bytes = read_payload_bytes(path)?;

    match kind {
        EncryptionKind::Default => {
            let passphrase = load_passphrase(app)?;
            match decrypt_bytes_with_passphrase(&passphrase, &bytes) {
                Ok(plain) => utf8_from_bytes(plain),
                Err(_) => Err(decrypt_passphrase_required(path)),
            }
        }
        EncryptionKind::Custom => {
            let passphrase = passphrase_for_kind(app, store, path, kind)?;
            utf8_from_bytes(decrypt_bytes_with_passphrase(&passphrase, &bytes)?)
        }
    }
}

pub fn read_file_content(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &std::path::Path,
) -> Result<String, String> {
    if format::is_encrypted_path(path) {
        return read_encrypted_payload(app, store, path);
    }

    if !format::is_editable_path(path) {
        return Err(format!("无法在编辑器中打开：{}", path.display()));
    }

    std::fs::read_to_string(path).map_err(|err| format!("读取文件失败：{err}"))
}

pub fn write_file_content(
    app: &tauri::AppHandle,
    store: &FilePassphraseStore,
    path: &std::path::Path,
    content: &str,
) -> Result<(), String> {
    format::ensure_writable_path(path)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("创建目录失败：{err}"))?;
    }

    if let Some(kind) = format::encryption_kind(path) {
        let passphrase = passphrase_for_kind(app, store, path, kind)?;
        let encrypted = encrypt_bytes_with_passphrase(&passphrase, content.as_bytes())?;
        std::fs::write(path, encrypted).map_err(|err| format!("写入文件失败：{err}"))?;
        return Ok(());
    }

    std::fs::write(path, content).map_err(|err| format!("写入文件失败：{err}"))
}
