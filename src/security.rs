use crate::plugins::Plugin;
use crate::state::AppState;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use std::env;

pub struct SecurityPlugin;

#[async_trait]
impl Plugin for SecurityPlugin {
    fn name(&self) -> &'static str {
        "Security"
    }

    async fn handle(&self, input: &str, _state: &AppState) -> Option<String> {
        let prompt_lower = input.to_lowercase();
        if prompt_lower.starts_with("encrypt ") {
            let text = input[8..].trim();
            match encrypt(text) {
                Ok(cipher) => Some(format!("Encrypted: {cipher}")),
                Err(e) => Some(format!("Encryption failed: {e}")),
            }
        } else if prompt_lower.starts_with("decrypt ") {
            let text = input[8..].trim();
            match decrypt(text) {
                Ok(plain) => Some(format!("Decrypted: {plain}")),
                Err(e) => Some(format!("Decryption failed: {e}")),
            }
        } else {
            None
        }

    }
}

fn get_key() -> [u8; 32] {
    let key_str = std::env::var("JEEBS_SECRET_KEY").unwrap_or_else(|_| "01234567890123456789012345678901".to_string());
    let mut key = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = bytes.len().min(32);
    key[..len].copy_from_slice(&bytes[..len]);
    key
}

pub fn encrypt(data: &str) -> Result<String, String> {
    let key = get_key();
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes()).map_err(|e| format!("{}", e))?;
    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);
    Ok(general_purpose::STANDARD.encode(combined))
}

pub fn decrypt(encrypted_data: &str) -> Result<String, String> {
    let key = get_key();
    let cipher = Aes256Gcm::new(&key.into());

    let decoded = general_purpose::STANDARD.decode(encrypted_data).map_err(|e| format!("{}", e))?;
    if decoded.len() < 12 { return Err("Data too short".to_string()); }
    let nonce = Nonce::from_slice(&decoded[..12]);
    let ciphertext = &decoded[12..];
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| format!("{}", e))?;
    String::from_utf8(plaintext).map_err(|e| format!("{}", e))
}

pub fn generate_password(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
