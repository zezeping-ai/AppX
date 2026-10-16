//! 解锁 PIN：Argon2id PHC 字符串存储与校验

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

const MIN_PIN_LEN: usize = 4;
const MAX_PIN_LEN: usize = 64;

pub fn validate_pin_format(pin: &str) -> Result<(), String> {
    let trimmed = pin.trim();
    if trimmed.len() < MIN_PIN_LEN {
        return Err(format!("解锁 PIN 至少 {MIN_PIN_LEN} 位"));
    }
    if trimmed.len() > MAX_PIN_LEN {
        return Err(format!("解锁 PIN 最多 {MAX_PIN_LEN} 位"));
    }
    Ok(())
}

pub fn hash_pin(pin: &str) -> Result<String, String> {
    validate_pin_format(pin)?;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.trim().as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|err| format!("PIN 哈希失败：{err}"))
}

pub fn verify_pin(pin: &str, phc_hash: &str) -> Result<(), String> {
    let parsed =
        PasswordHash::new(phc_hash).map_err(|_| "解锁 PIN 配置损坏，请重新设置".to_string())?;
    Argon2::default()
        .verify_password(pin.trim().as_bytes(), &parsed)
        .map_err(|_| "解锁 PIN 不正确".to_string())
}
