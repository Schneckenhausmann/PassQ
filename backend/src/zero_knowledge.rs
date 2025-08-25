//! Zero-Knowledge Architecture Module
//! Ensures server never sees passwords or sensitive data in plaintext

use ring::rand::{SecureRandom, SystemRandom};
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use log;
use base64::{Engine as _, engine::general_purpose};

/// Client-side encryption parameters
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptionParams {
    pub salt: String,           // Base64 encoded salt for key derivation
    pub iterations: u32,        // PBKDF2 iterations
    pub nonce: String,          // Base64 encoded nonce for encryption
}

/// Encrypted data structure sent from client
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedData {
    pub data: String,           // Base64 encoded encrypted data
    pub params: EncryptionParams,
}

/// Zero-knowledge user registration data
#[derive(Serialize, Deserialize)]
pub struct ZkUserRegistration {
    pub username: String,
    pub password_hash: String,  // Client-side hashed password
    pub salt: String,           // Salt used for password hashing
    pub verification_key: String, // Key for server-side verification
}

/// Zero-knowledge password entry
#[derive(Serialize, Deserialize)]
pub struct ZkPasswordEntry {
    pub website: EncryptedData,
    pub username: EncryptedData,
    pub password: EncryptedData,
    pub notes: Option<EncryptedData>,
    pub folder_id: Option<uuid::Uuid>,
}

/// Zero-knowledge authentication challenge
#[derive(Serialize, Deserialize)]
pub struct ZkAuthChallenge {
    pub challenge: String,      // Server-generated challenge
    pub salt: String,           // User's salt for key derivation
}

/// Zero-knowledge authentication response
#[derive(Serialize, Deserialize)]
pub struct ZkAuthResponse {
    pub username: String,
    pub challenge_response: String, // HMAC of challenge with derived key
}

#[allow(dead_code)]
pub struct ZeroKnowledgeManager {
    system_random: SystemRandom,
}

#[allow(dead_code)]
impl ZeroKnowledgeManager {
    pub fn new() -> Self {
        ZeroKnowledgeManager {
            system_random: SystemRandom::new(),
        }
    }

    /// Generate a cryptographically secure salt
    pub fn generate_salt(&self) -> Result<Vec<u8>, String> {
        let mut salt = [0u8; 32];
        self.system_random.fill(&mut salt)
            .map_err(|e| format!("Failed to generate salt: {}", e))?;
        Ok(salt.to_vec())
    }

    /// Generate a cryptographically secure nonce
    pub fn generate_nonce(&self) -> Result<Vec<u8>, String> {
        let mut nonce = [0u8; 12]; // 96-bit nonce for AES-GCM
        self.system_random.fill(&mut nonce)
            .map_err(|e| format!("Failed to generate nonce: {}", e))?;
        Ok(nonce.to_vec())
    }

    /// Generate authentication challenge
    pub fn generate_auth_challenge(&self) -> Result<String, String> {
        let mut challenge = [0u8; 32];
        self.system_random.fill(&mut challenge)
            .map_err(|e| format!("Failed to generate challenge: {}", e))?;
        Ok(general_purpose::STANDARD.encode(challenge))
    }

    /// Derive key from password using PBKDF2
    /// This should be done on the client side, but we provide server-side validation
    pub fn derive_key_from_password(
        password: &str,
        salt: &[u8],
        iterations: u32,
    ) -> Result<Vec<u8>, String> {
        let mut key = [0u8; 32]; // 256-bit key
        let iterations = NonZeroU32::new(iterations)
            .ok_or("Iterations must be greater than 0")?;
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt,
            password.as_bytes(),
            &mut key,
        );
        
        Ok(key.to_vec())
    }

    /// Verify encrypted data integrity without decrypting
    pub fn verify_encrypted_data(&self, encrypted_data: &EncryptedData) -> Result<bool, String> {
        // Decode base64 data
        let data = general_purpose::STANDARD.decode(&encrypted_data.data)
            .map_err(|e| format!("Invalid base64 data: {}", e))?;
        
        // Verify minimum length (nonce + tag + at least 1 byte of data)
        if data.len() < 12 + 16 + 1 {
            return Ok(false);
        }
        
        // Verify nonce format
        general_purpose::STANDARD.decode(&encrypted_data.params.nonce)
            .map_err(|e| format!("Invalid nonce format: {}", e))?;
        
        // Verify salt format
        general_purpose::STANDARD.decode(&encrypted_data.params.salt)
            .map_err(|e| format!("Invalid salt format: {}", e))?;
        
        // Verify iterations are within acceptable range
        if encrypted_data.params.iterations < 10000 || encrypted_data.params.iterations > 1000000 {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Validate zero-knowledge authentication response
    pub fn validate_auth_response(
        &self,
        challenge: &str,
        response: &ZkAuthResponse,
        stored_verification_key: &str,
    ) -> Result<bool, String> {
        use ring::hmac;
        
        // Decode the stored verification key
        let verification_key = general_purpose::STANDARD.decode(stored_verification_key)
            .map_err(|e| format!("Invalid verification key: {}", e))?;
        
        // Create HMAC key
        let key = hmac::Key::new(hmac::HMAC_SHA256, &verification_key);
        
        // Compute expected HMAC
        let _expected_hmac = hmac::sign(&key, challenge.as_bytes());
        
        // Decode the provided response
        let provided_response = general_purpose::STANDARD.decode(&response.challenge_response)
            .map_err(|e| format!("Invalid challenge response: {}", e))?;
        
        // Verify HMAC
        hmac::verify(&key, challenge.as_bytes(), &provided_response)
            .map_err(|_| "Authentication failed".to_string())?;
        
        Ok(true)
    }

    /// Generate client-side encryption parameters
    pub fn generate_encryption_params(&self) -> Result<EncryptionParams, String> {
        let salt = self.generate_salt()?;
        let nonce = self.generate_nonce()?;
        
        Ok(EncryptionParams {
            salt: general_purpose::STANDARD.encode(salt),
            iterations: 100000, // OWASP recommended minimum
            nonce: general_purpose::STANDARD.encode(nonce),
        })
    }

    /// Validate password strength on server side
    pub fn validate_password_strength(password_hash: &str) -> Result<(), String> {
        // Since we only receive hashed passwords, we validate the hash format
        let decoded = general_purpose::STANDARD.decode(password_hash)
            .map_err(|_| "Invalid password hash format")?;
        
        // Ensure hash is the expected length (SHA-256 = 32 bytes)
        if decoded.len() != 32 {
            return Err("Invalid password hash length".to_string());
        }
        
        Ok(())
    }

    /// Create verification key from client-derived key
    /// This allows server to verify user identity without knowing the password
    pub fn create_verification_key(client_key: &[u8]) -> Result<String, String> {
        use ring::digest;
        
        // Hash the client key to create a verification key
        let verification_key = digest::digest(&digest::SHA256, client_key);
        Ok(general_purpose::STANDARD.encode(verification_key.as_ref()))
    }

    /// Audit log entry for zero-knowledge operations
    pub fn log_zk_operation(&self, operation: &str, username: &str, success: bool) {
        if success {
            log::info!("ZK Operation successful: {} for user: {}", operation, username);
        } else {
            log::warn!("ZK Operation failed: {} for user: {}", operation, username);
        }
    }
}

/// Client-side encryption utilities (for reference/documentation)
/// These functions should be implemented in JavaScript on the client side
pub mod client_side_reference {
    //! Reference implementation for client-side encryption
    //! This code should be implemented in JavaScript in the frontend
    
    /// JavaScript reference for client-side password hashing
    #[allow(dead_code)]
    pub const CLIENT_PASSWORD_HASHING: &str = r#"
    // Client-side password hashing (JavaScript)
    async function hashPassword(password, salt, iterations = 100000) {
        const encoder = new TextEncoder();
        const passwordBuffer = encoder.encode(password);
        const saltBuffer = new Uint8Array(atob(salt).split('').map(c => c.charCodeAt(0)));
        
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            passwordBuffer,
            { name: 'PBKDF2' },
            false,
            ['deriveBits']
        );
        
        const derivedBits = await crypto.subtle.deriveBits(
            {
                name: 'PBKDF2',
                salt: saltBuffer,
                iterations: iterations,
                hash: 'SHA-256'
            },
            keyMaterial,
            256
        );
        
        return btoa(String.fromCharCode(...new Uint8Array(derivedBits)));
    }
    "#;
    
    /// JavaScript reference for client-side data encryption
    #[allow(dead_code)]
    pub const CLIENT_DATA_ENCRYPTION: &str = r#"
    // Client-side data encryption (JavaScript)
    async function encryptData(data, password, salt, iterations = 100000) {
        const encoder = new TextEncoder();
        const dataBuffer = encoder.encode(data);
        
        // Derive key from password
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            encoder.encode(password),
            { name: 'PBKDF2' },
            false,
            ['deriveBits']
        );
        
        const saltBuffer = new Uint8Array(atob(salt).split('').map(c => c.charCodeAt(0)));
        const derivedKey = await crypto.subtle.deriveBits(
            {
                name: 'PBKDF2',
                salt: saltBuffer,
                iterations: iterations,
                hash: 'SHA-256'
            },
            keyMaterial,
            256
        );
        
        // Import derived key for AES-GCM
        const cryptoKey = await crypto.subtle.importKey(
            'raw',
            derivedKey,
            { name: 'AES-GCM' },
            false,
            ['encrypt']
        );
        
        // Generate nonce
        const nonce = crypto.getRandomValues(new Uint8Array(12));
        
        // Encrypt data
        const encryptedData = await crypto.subtle.encrypt(
            {
                name: 'AES-GCM',
                iv: nonce
            },
            cryptoKey,
            dataBuffer
        );
        
        return {
            data: btoa(String.fromCharCode(...new Uint8Array(encryptedData))),
            params: {
                salt: salt,
                iterations: iterations,
                nonce: btoa(String.fromCharCode(...nonce))
            }
        };
    }
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_generation() {
        let zk_manager = ZeroKnowledgeManager::new();
        let salt = zk_manager.generate_salt().unwrap();
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn test_nonce_generation() {
        let zk_manager = ZeroKnowledgeManager::new();
        let nonce = zk_manager.generate_nonce().unwrap();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt_32_bytes_long_exactly";
        let iterations = 100000;
        
        let key = ZeroKnowledgeManager::derive_key_from_password(password, salt, iterations).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_verification_key_creation() {
        let client_key = b"test_client_key_32_bytes_long_ex";
        let verification_key = ZeroKnowledgeManager::create_verification_key(client_key).unwrap();
        assert!(!verification_key.is_empty());
    }

    #[test]
    fn test_encryption_params_generation() {
        let zk_manager = ZeroKnowledgeManager::new();
        let params = zk_manager.generate_encryption_params().unwrap();
        
        assert!(!params.salt.is_empty());
        assert!(!params.nonce.is_empty());
        assert_eq!(params.iterations, 100000);
    }
}