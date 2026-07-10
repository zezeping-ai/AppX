use base64::Engine;
use tauri::AppHandle;

use crate::app::crypto::{decrypt_bytes_with_passphrase, encrypt_bytes_with_passphrase};
use crate::app::security::load_default_passphrase;

#[tauri::command]
pub fn crypto_encrypt_text(app: AppHandle, plaintext: String) -> Result<String, String> {
    let passphrase = load_default_passphrase(&app)?;
    let encrypted = encrypt_bytes_with_passphrase(&passphrase, plaintext.as_bytes())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(encrypted))
}

#[tauri::command]
pub fn crypto_decrypt_text(app: AppHandle, ciphertext_b64: String) -> Result<String, String> {
    let passphrase = load_default_passphrase(&app)?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64.trim())
        .map_err(|err| format!("密文 Base64 无效：{err}"))?;
    let plain = decrypt_bytes_with_passphrase(&passphrase, &payload)?;
    String::from_utf8(plain).map_err(|err| format!("解密结果不是有效 UTF-8：{err}"))
}
