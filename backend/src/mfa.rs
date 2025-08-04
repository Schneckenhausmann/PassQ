//! Multi-Factor Authentication module

use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;
use log;

/// Generates a TOTP secret for a user
pub fn generate_totp_secret(_user_id: Uuid) -> String {
    let secret = Secret::generate_secret(); // Not a Result, no expect
    secret.to_string()
}

/// Generates a QR code provisioning URL for the authenticator app
pub fn generate_qr_code_url(user_id: Uuid, secret: &str) -> String {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,                          // digits
        1,                          // skew
        30,                         // step
        secret.as_bytes().to_vec(),
        Some("MyApp".to_string()),  // issuer
        format!("user_{}", user_id) // account name
    ).expect("Failed to create TOTP instance");

    totp.get_url()
}

/// Generates a TOTP code from a secret
pub fn generate_totp_code(secret: &str) -> Result<String, String> {
    if secret.is_empty() {
        return Err("Empty TOTP secret provided".to_string());
    }

    // Validate secret format - check if it's a valid base32 string
    if secret.len() < 16 || secret.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Err("Invalid TOTP secret format".to_string());
    }

    match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
        Some("MyApp".to_string()),
        "account".to_string(),
    ) {
        Ok(totp) => {
            match totp.generate_current() {
                Ok(code) => Ok(code),
                Err(e) => {
                    log::error!("Failed to generate TOTP code: {}", e);
                    Err("Failed to generate TOTP code".to_string())
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create TOTP instance: {}", e);
            Err("Failed to create TOTP instance".to_string())
        }
    }
}

/// Verifies a TOTP code
pub fn verify_totp_code(secret: &str, code: &str) -> bool {
    if secret.is_empty() || code.is_empty() {
        log::warn!("Empty TOTP secret or code provided");
        return false;
    }

    // Validate secret format - check if it's a valid base32 string
    if secret.len() < 16 || secret.chars().any(|c| !c.is_ascii_alphanumeric()) {
        log::warn!("Invalid TOTP secret format");
        return false;
    }

    match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.as_bytes().to_vec(),
        Some("MyApp".to_string()),
        "account".to_string(),
    ) {
        Ok(totp) => {
            match totp.check_current(code) {
                Ok(result) => {
                    log::info!("TOTP verification successful");
                    result
                }
                Err(e) => {
                    log::warn!("TOTP verification failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create TOTP instance: {}", e);
            false
        }
    }
}
