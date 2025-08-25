//! Cryptography module for encryption/decryption

use ring::aead;
use ring::rand::{SecureRandom, SystemRandom};
use std::env;
use log;
use crate::key_management::get_key_manager;

/// Generates a random key for encryption using enterprise key management
#[allow(dead_code)]
pub async fn generate_key_async() -> Result<aead::LessSafeKey, String> {
    let key_manager = get_key_manager()?;
    log::info!("Using key provider: {}", key_manager.get_provider_info());
    key_manager.get_encryption_key().await
}

/// Generates a random key for encryption (synchronous version for backward compatibility)
pub fn generate_key() -> Result<aead::LessSafeKey, String> {
    let key_material = match env::var("ENCRYPTION_KEY") {
        Ok(key) => {
            log::info!("Encryption key loaded successfully from environment");
            key
        }
        Err(_) => {
            log::error!("Environment variable ENCRYPTION_KEY is not set");
            return Err("ENCRYPTION_KEY environment variable must be set".to_string());
        }
    };

    let key_bytes = key_material.as_bytes();

    if key_bytes.len() != 32 {
        log::error!("ENCRYPTION_KEY must be exactly 32 bytes for AES-256-GCM, got {} bytes", key_bytes.len());
        return Err("ENCRYPTION_KEY must be exactly 32 bytes for AES-256-GCM".to_string());
    }

    match aead::UnboundKey::new(&aead::AES_256_GCM, key_bytes) {
        Ok(unbound_key) => {
            log::info!("Encryption key generated successfully");
            Ok(aead::LessSafeKey::new(unbound_key))
        }
        Err(e) => {
            log::error!("Failed to create encryption key: {}", e);
            Err(format!("Failed to create encryption key: {}", e))
        }
    }
}

/// Encrypts data using AES-256-GCM
/// Returns nonce + encrypted_data + tag
pub fn encrypt(mut data: Vec<u8>, key: &aead::LessSafeKey) -> Result<Vec<u8>, String> {
    log::debug!("Encrypting {} bytes of data", data.len());
    
    let mut nonce_bytes = [0u8; 12];
    let system_random = SystemRandom::new();
    system_random.fill(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate random nonce: {}", e))?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
    
    match key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut data) {
        Ok(_) => {
            log::debug!("Encryption successful");
            // Prepend nonce to encrypted data for storage
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&data);
            Ok(result)
        }
        Err(e) => {
            log::error!("Encryption failed: {}", e);
            Err(format!("Encryption failed: {}", e))
        }
    }
}

/// Decrypts data using AES-256-GCM
/// Expects nonce + encrypted_data + tag format
pub fn decrypt(encrypted: Vec<u8>, key: &aead::LessSafeKey) -> Result<Vec<u8>, String> {
    log::debug!("Decrypting {} bytes of data", encrypted.len());
    
    if encrypted.len() < 12 {
        return Err("Encrypted data too short to contain nonce".to_string());
    }
    
    // Extract nonce from the beginning of encrypted data
    let nonce_bytes: [u8; 12] = encrypted[0..12].try_into()
        .map_err(|_| "Failed to extract nonce from encrypted data".to_string())?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
    
    // Remove nonce from encrypted data
    let mut ciphertext = encrypted[12..].to_vec();
    
    match key.open_in_place(nonce, aead::Aad::empty(), &mut ciphertext) {
        Ok(decrypted_data) => {
            log::debug!("Decryption successful");
            Ok(decrypted_data.to_vec())
        }
        Err(e) => {
            log::error!("Decryption failed: {}", e);
            Err(format!("Decryption failed: {}", e))
        }
    }
}

/// Encrypts a password string
pub fn encrypt_password(password: &str) -> Result<Vec<u8>, String> {
    let key = generate_key()?;
    encrypt(password.as_bytes().to_vec(), &key)
}

/// Encrypts a password string (async version for enterprise key management)
#[allow(dead_code)]
pub async fn encrypt_password_async(password: &str) -> Result<Vec<u8>, String> {
    let key = generate_key_async().await?;
    encrypt(password.as_bytes().to_vec(), &key)
}

/// Decrypts a password from binary data
pub fn decrypt_password(encrypted_password: &[u8]) -> Result<String, String> {
    let key = generate_key()?;
    let decrypted_data = decrypt(encrypted_password.to_vec(), &key)?;
    String::from_utf8(decrypted_data)
        .map_err(|e| format!("Failed to convert decrypted data to string: {}", e))
}

/// Decrypts a password from binary data (async version for enterprise key management)
#[allow(dead_code)]
pub async fn decrypt_password_async(encrypted_password: &[u8]) -> Result<String, String> {
    let key = generate_key_async().await?;
    let decrypted_data = decrypt(encrypted_password.to_vec(), &key)?;
    String::from_utf8(decrypted_data)
        .map_err(|e| format!("Failed to convert decrypted data to string: {}", e))
}

/// Encrypts metadata (website URL or username)
pub fn encrypt_metadata(metadata: &str) -> Result<Vec<u8>, String> {
    let key = generate_key()?;
    encrypt(metadata.as_bytes().to_vec(), &key)
}

/// Encrypts metadata (async version for enterprise key management)
#[allow(dead_code)]
pub async fn encrypt_metadata_async(metadata: &str) -> Result<Vec<u8>, String> {
    let key = generate_key_async().await?;
    encrypt(metadata.as_bytes().to_vec(), &key)
}

/// Decrypts metadata from binary data
pub fn decrypt_metadata(encrypted_metadata: &[u8]) -> Result<String, String> {
    let key = generate_key()?;
    let decrypted_data = decrypt(encrypted_metadata.to_vec(), &key)?;
    String::from_utf8(decrypted_data)
        .map_err(|e| format!("Failed to convert decrypted metadata to string: {}", e))
}

/// Decrypts metadata from binary data (async version for enterprise key management)
#[allow(dead_code)]
pub async fn decrypt_metadata_async(encrypted_metadata: &[u8]) -> Result<String, String> {
    let key = generate_key_async().await?;
    let decrypted_data = decrypt(encrypted_metadata.to_vec(), &key)?;
    String::from_utf8(decrypted_data)
        .map_err(|e| format!("Failed to convert decrypted metadata to string: {}", e))
}
