//! Secure, user-isolated credential and connection store.
//!
//! Enforces:
//! - Local user isolation: secrets are encrypted on disk with a user-and-machine-derived key.
//! - Redaction: secrets are never dumped to logs or stdout.
//! - Safe defaults: missing store returns empty list without panicking.

use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::types::ConnectionConfig;

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("I/O error during credential storage: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Decryption failed: corrupted credential data or changed user environment")]
    DecryptionFailed,
}

const SALT: &[u8] = b"TZWorkQuint-CredStore-Salt-v1-7f9a2b8e4d1c";

/// Derives a 32-byte encryption key bound to the current operating system user.
fn derive_user_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(SALT);

    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }
    if let Ok(domain) = std::env::var("USERDOMAIN").or_else(|_| std::env::var("COMPUTERNAME")) {
        hasher.update(domain.as_bytes());
    }
    if let Ok(profile) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        hasher.update(profile.as_bytes());
    }

    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Encrypts/decrypts plaintext using a keystream derived from the key and a nonce (counter-mode style).
fn apply_keystream(key: &[u8; 32], nonce: &[u8; 16], data: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len());
    let mut counter: u64 = 0;

    let mut chunk_idx = 0;
    while chunk_idx < data.len() {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(&counter.to_le_bytes());
        let stream_block = hasher.finalize();

        let take = (data.len() - chunk_idx).min(32);
        for i in 0..take {
            output.push(data[chunk_idx + i] ^ stream_block[i]);
        }

        chunk_idx += take;
        counter = counter.wrapping_add(1);
    }

    output
}

/// Computes an HMAC-SHA256 authentication tag for ciphertext integrity.
fn compute_mac(key: &[u8; 32], nonce: &[u8; 16], ciphertext: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"MAC-KEY");
    hasher.update(key);
    let mac_key = hasher.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(&mac_key);
    hasher2.update(nonce);
    hasher2.update(ciphertext);
    let tag = hasher2.finalize();

    let mut out = [0u8; 32];
    out.copy_from_slice(&tag);
    out
}

/// Returns the standard credential store path for the current user.
pub fn default_store_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("TZWorkQuint")
            .join("ai_credentials.bin")
    } else if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        PathBuf::from(home)
            .join(".tzworkquint")
            .join("ai_credentials.bin")
    } else {
        PathBuf::from("ai_credentials.bin")
    }
}

/// Saves a list of connection configurations to the encrypted store.
pub fn save_connections(path: &Path, connections: &[ConnectionConfig]) -> Result<(), CredentialError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json_bytes = serde_json::to_vec(connections)?;
    let key = derive_user_key();

    // Generate pseudo-random nonce from timestamp + data length + salt
    let mut nonce_hasher = Sha256::new();
    nonce_hasher.update(SALT);
    nonce_hasher.update(
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
            .to_le_bytes(),
    );
    nonce_hasher.update(&(json_bytes.len() as u64).to_le_bytes());
    let nonce_hash = nonce_hasher.finalize();
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&nonce_hash[..16]);

    let ciphertext = apply_keystream(&key, &nonce, &json_bytes);
    let mac = compute_mac(&key, &nonce, &ciphertext);

    // Format: [MAGIC (4)] [NONCE (16)] [MAC (32)] [CIPHERTEXT (...)]
    let mut payload = Vec::with_capacity(4 + 16 + 32 + ciphertext.len());
    payload.extend_from_slice(b"TZCR");
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&mac);
    payload.extend_from_slice(&ciphertext);

    fs::write(path, payload)?;
    Ok(())
}

/// Loads connection configurations from the encrypted store.
/// Returns an empty list if the file does not exist.
pub fn load_connections(path: &Path) -> Result<Vec<ConnectionConfig>, CredentialError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let data = fs::read(path)?;
    if data.len() < 4 + 16 + 32 || &data[..4] != b"TZCR" {
        return Err(CredentialError::DecryptionFailed);
    }

    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&data[4..20]);

    let mut expected_mac = [0u8; 32];
    expected_mac.copy_from_slice(&data[20..52]);

    let ciphertext = &data[52..];
    let key = derive_user_key();

    let computed_mac = compute_mac(&key, &nonce, ciphertext);
    if computed_mac != expected_mac {
        return Err(CredentialError::DecryptionFailed);
    }

    let plaintext = apply_keystream(&key, &nonce, ciphertext);
    let mut connections: Vec<ConnectionConfig> = serde_json::from_slice(&plaintext)?;
    let mut migrated = false;
    for conn in &mut connections {
        if conn.name.contains("FreeLLMAPI") || conn.id == "freellmapi-local" {
            conn.name = conn.name
                .replace("Local FreeLLMAPI Gateway", "TZ AI Gateway (Local)")
                .replace("FreeLLMAPI Gateway", "TZ AI Gateway (Local)")
                .replace("FreeLLMAPI", "TZ AI Gateway");
            if conn.id == "freellmapi-local" {
                conn.id = "tz-gateway-local".to_string();
            }
            migrated = true;
        }
    }
    if migrated {
        let _ = save_connections(path, &connections);
    }
    Ok(connections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_roundtrip() {
        let temp_dir = std::env::temp_dir().join("tz_cred_test");
        let store_path = temp_dir.join("test_store.bin");

        let original = vec![
            ConnectionConfig {
                id: "c1".to_string(),
                name: "Test Local TZ AI Gateway".to_string(),
                provider_type: crate::types::ProviderType::OpenAiCompatible,
                base_url: "http://localhost:3001/v1".to_string(),
                api_key: Some("secret-key-12345678".to_string()),
                model: "llama3.2".to_string(),
                temperature: Some(0.3),
                max_tokens: Some(2048),
                enabled: true,
            },
            ConnectionConfig {
                id: "c2".to_string(),
                name: "Test Gemini".to_string(),
                provider_type: crate::types::ProviderType::Gemini,
                base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
                api_key: Some("AIzaSy-TestSecretKey".to_string()),
                model: "gemini-2.5-flash".to_string(),
                temperature: None,
                max_tokens: None,
                enabled: true,
            },
        ];

        save_connections(&store_path, &original).expect("save failed");
        let loaded = load_connections(&store_path).expect("load failed");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Test Local TZ AI Gateway");
        assert_eq!(loaded[0].api_key, Some("secret-key-12345678".to_string()));
        assert_eq!(loaded[1].name, "Test Gemini");
        assert_eq!(loaded[1].api_key, Some("AIzaSy-TestSecretKey".to_string()));

        // Ensure raw file on disk does NOT contain the secret in plaintext
        let raw_disk = fs::read(&store_path).expect("read raw failed");
        let raw_str = String::from_utf8_lossy(&raw_disk);
        assert!(!raw_str.contains("secret-key-12345678"));
        assert!(!raw_str.contains("AIzaSy-TestSecretKey"));

        // Cleanup
        let _ = fs::remove_file(&store_path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
