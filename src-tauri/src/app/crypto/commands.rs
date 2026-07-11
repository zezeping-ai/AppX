use base64::Engine;
use tauri::{AppHandle, State};

use crate::app::app_lock::{ensure_unlocked, AppLockSessionState};
use crate::app::crypto::{decrypt_bytes_with_passphrase, encrypt_bytes_with_passphrase};
use crate::app::security::load_default_passphrase;

#[tauri::command]
pub fn crypto_encrypt_text(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    plaintext: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    let passphrase = load_default_passphrase(&app)?;
    let encrypted = encrypt_bytes_with_passphrase(&passphrase, plaintext.as_bytes())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(encrypted))
}

#[tauri::command]
pub fn crypto_decrypt_text(
    app: AppHandle,
    state: State<'_, AppLockSessionState>,
    ciphertext_b64: String,
) -> Result<String, String> {
    ensure_unlocked(&state)?;
    let passphrase = load_default_passphrase(&app)?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64.trim())
        .map_err(|err| format!("密文 Base64 无效：{err}"))?;
    let plain = decrypt_bytes_with_passphrase(&passphrase, &payload)?;
    String::from_utf8(plain).map_err(|err| format!("解密结果不是有效 UTF-8：{err}"))
}
