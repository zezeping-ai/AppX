//! 触发时：查库 → 按目标 App 选密文 → 解密为明文（命中 registry 缓存则跳过 Argon2）。

use base64::Engine;
use tauri::{AppHandle, Manager};

use crate::app::crypto::decrypt_bytes_with_passphrase;
use crate::app::platform;
use crate::app::security::load_default_passphrase;

use super::db;
use super::registry::SnippetRegistry;

/// 解析 snippet 最终粘贴明文：当前前台 App 命中变体优先，否则默认 content。
pub fn resolve_plain_content(app: &AppHandle, id: i64) -> Result<String, String> {
    resolve_plain_content_for_app(app, id, platform::frontmost_app().0.as_deref())
}

/// 指定目标 app_id 匹配变体（命令面板用打开前捕获的焦点）。
pub fn resolve_plain_content_for_app(
    app: &AppHandle,
    id: i64,
    target_app_id: Option<&str>,
) -> Result<String, String> {
    let app_key = app_cache_key(target_app_id);
    if let Some(registry) = app.try_state::<SnippetRegistry>() {
        if let Some(hit) = registry.cached_plaintext(id, &app_key) {
            return Ok(hit);
        }
    }

    let stored = db::load_snippet_payload(app, id)?;
    let cipher_b64 = select_cipher(&stored.content, &stored.variants, target_app_id);
    let plain = decrypt_cipher_b64(app, &cipher_b64)?;

    if let Some(registry) = app.try_state::<SnippetRegistry>() {
        registry.store_plaintext(id, &app_key, plain.clone());
    }
    Ok(plain)
}

fn app_cache_key(target_app_id: Option<&str>) -> String {
    target_app_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string()
}

fn select_cipher(
    default_content: &str,
    variants: &[super::model::AppVariantRow],
    target_app_id: Option<&str>,
) -> String {
    let Some(front_id) = target_app_id.map(str::trim).filter(|s| !s.is_empty()) else {
        return default_content.to_string();
    };
    if let Some(hit) = variants.iter().find(|v| v.app_id == front_id) {
        if !hit.content.trim().is_empty() {
            return hit.content.clone();
        }
    }
    default_content.to_string()
}

fn decrypt_cipher_b64(app: &AppHandle, cipher_b64: &str) -> Result<String, String> {
    let passphrase = load_default_passphrase(app)?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(cipher_b64.trim())
        .map_err(|err| format!("密文无效：{err}"))?;
    let plain = decrypt_bytes_with_passphrase(&passphrase, &payload)?;
    String::from_utf8(plain).map_err(|err| format!("解密结果无效：{err}"))
}
