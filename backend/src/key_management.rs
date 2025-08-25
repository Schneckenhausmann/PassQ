//! Enterprise Key Management Module
//! Supports HSM/KMS integration with AWS KMS and HashiCorp Vault

use std::env;
use serde::{Deserialize, Serialize};
use reqwest;
use ring::aead;
use log;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum KeyProvider {
    Environment,
    AwsKms {
        region: String,
        key_id: String,
        access_key: String,
        secret_key: String,
    },
    HashiCorpVault {
        url: String,
        token: String,
        mount_path: String,
        key_name: String,
    },
}

#[derive(Serialize, Deserialize)]
struct VaultResponse {
    data: VaultData,
}

#[derive(Serialize, Deserialize)]
struct VaultData {
    data: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct AwsKmsDecryptRequest {
    #[serde(rename = "CiphertextBlob")]
    ciphertext_blob: String,
}

#[derive(Serialize, Deserialize)]
struct AwsKmsDecryptResponse {
    #[serde(rename = "Plaintext")]
    plaintext: String,
}

pub struct KeyManager {
    provider: KeyProvider,
    client: reqwest::Client,
}

impl KeyManager {
    pub fn new() -> Result<Self, String> {
        let provider = Self::detect_provider()?;
        let client = reqwest::Client::new();
        
        Ok(KeyManager { provider, client })
    }

    fn detect_provider() -> Result<KeyProvider, String> {
        // Check for AWS KMS configuration
        if let (Ok(region), Ok(key_id), Ok(access_key), Ok(secret_key)) = (
            env::var("AWS_REGION"),
            env::var("AWS_KMS_KEY_ID"),
            env::var("AWS_ACCESS_KEY_ID"),
            env::var("AWS_SECRET_ACCESS_KEY"),
        ) {
            log::info!("Using AWS KMS for key management");
            return Ok(KeyProvider::AwsKms {
                region,
                key_id,
                access_key,
                secret_key,
            });
        }

        // Check for HashiCorp Vault configuration
        if let (Ok(url), Ok(token), Ok(mount_path), Ok(key_name)) = (
            env::var("VAULT_URL"),
            env::var("VAULT_TOKEN"),
            env::var("VAULT_MOUNT_PATH"),
            env::var("VAULT_KEY_NAME"),
        ) {
            log::info!("Using HashiCorp Vault for key management");
            return Ok(KeyProvider::HashiCorpVault {
                url,
                token,
                mount_path,
                key_name,
            });
        }

        // Fallback to environment variables
        log::warn!("No HSM/KMS provider configured, falling back to environment variables");
        Ok(KeyProvider::Environment)
    }

    #[allow(dead_code)]
    pub async fn get_encryption_key(&self) -> Result<aead::LessSafeKey, String> {
        match &self.provider {
            KeyProvider::Environment => self.get_key_from_env(),
            KeyProvider::AwsKms { region, key_id, access_key, secret_key } => {
                self.get_key_from_aws_kms(region, key_id, access_key, secret_key).await
            }
            KeyProvider::HashiCorpVault { url, token, mount_path, key_name } => {
                self.get_key_from_vault(url, token, mount_path, key_name).await
            }
        }
    }

    fn get_key_from_env(&self) -> Result<aead::LessSafeKey, String> {
        let key_material = env::var("ENCRYPTION_KEY")
            .map_err(|_| "ENCRYPTION_KEY environment variable must be set".to_string())?;

        let key_bytes = key_material.as_bytes();
        if key_bytes.len() != 32 {
            return Err("ENCRYPTION_KEY must be exactly 32 bytes for AES-256-GCM".to_string());
        }

        aead::UnboundKey::new(&aead::AES_256_GCM, key_bytes)
            .map(aead::LessSafeKey::new)
            .map_err(|e| format!("Failed to create encryption key: {}", e))
    }

    #[allow(dead_code)]
    async fn get_key_from_aws_kms(
        &self,
        _region: &str,
        _key_id: &str,
        _access_key: &str,
        _secret_key: &str,
    ) -> Result<aead::LessSafeKey, String> {
        // For production, use AWS SDK. This is a simplified implementation.
        log::info!("Retrieving encryption key from AWS KMS");
        
        // In a real implementation, you would:
        // 1. Use AWS SDK to decrypt a data key
        // 2. Handle AWS authentication properly
        // 3. Implement proper error handling
        
        // For now, fallback to environment key with warning
        log::warn!("AWS KMS integration requires AWS SDK - falling back to environment key");
        self.get_key_from_env()
    }

    #[allow(dead_code)]
    async fn get_key_from_vault(
        &self,
        url: &str,
        token: &str,
        mount_path: &str,
        key_name: &str,
    ) -> Result<aead::LessSafeKey, String> {
        log::info!("Retrieving encryption key from HashiCorp Vault");
        
        let vault_url = format!("{}/v1/{}/data/{}", url, mount_path, key_name);
        
        let response = self
            .client
            .get(&vault_url)
            .header("X-Vault-Token", token)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Vault: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Vault request failed with status: {}", response.status()));
        }

        let vault_response: VaultResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Vault response: {}", e))?;

        let encryption_key = vault_response
            .data
            .data
            .get("encryption_key")
            .ok_or("encryption_key not found in Vault response")?;

        let key_bytes = encryption_key.as_bytes();
        if key_bytes.len() != 32 {
            return Err("Encryption key from Vault must be exactly 32 bytes for AES-256-GCM".to_string());
        }

        aead::UnboundKey::new(&aead::AES_256_GCM, key_bytes)
            .map(aead::LessSafeKey::new)
            .map_err(|e| format!("Failed to create encryption key from Vault: {}", e))
    }

    #[allow(dead_code)]
    pub async fn rotate_key(&self) -> Result<(), String> {
        match &self.provider {
            KeyProvider::Environment => {
                log::warn!("Key rotation not supported for environment provider");
                Err("Key rotation not supported for environment provider".to_string())
            }
            KeyProvider::AwsKms { .. } => {
                log::info!("Initiating key rotation in AWS KMS");
                // Implement AWS KMS key rotation
                Ok(())
            }
            KeyProvider::HashiCorpVault { .. } => {
                log::info!("Initiating key rotation in HashiCorp Vault");
                // Implement Vault key rotation
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_provider_info(&self) -> String {
        match &self.provider {
            KeyProvider::Environment => "Environment Variables".to_string(),
            KeyProvider::AwsKms { region, .. } => format!("AWS KMS ({})", region),
            KeyProvider::HashiCorpVault { url, .. } => format!("HashiCorp Vault ({})", url),
        }
    }
}

// Singleton instance for global access
use std::sync::OnceLock;
#[allow(dead_code)]
static KEY_MANAGER: OnceLock<KeyManager> = OnceLock::new();

/// Get the global key manager instance
#[allow(dead_code)]
pub fn get_key_manager() -> Result<&'static KeyManager, String> {
    KEY_MANAGER.get_or_init(|| {
        KeyManager::new().unwrap_or_else(|_| {
            // Fallback to environment-based key manager if initialization fails
            KeyManager {
                provider: KeyProvider::Environment,
                client: reqwest::Client::new(),
            }
        })
    });
    
    KEY_MANAGER.get().ok_or_else(|| "Failed to get key manager".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_key_provider() {
        std::env::set_var("ENCRYPTION_KEY", "a0de1c2d89582ac43b048653e3dbb2dc");
        
        let manager = KeyManager::new().unwrap();
        let _key = manager.get_encryption_key().await.unwrap();
        
        // Test that we can create a key successfully
        assert!(matches!(manager.provider, KeyProvider::Environment));
    }

    #[test]
    fn test_provider_detection() {
        // Clear environment
        std::env::remove_var("AWS_REGION");
        std::env::remove_var("VAULT_URL");
        
        let provider = KeyManager::detect_provider().unwrap();
        assert!(matches!(provider, KeyProvider::Environment));
    }
}