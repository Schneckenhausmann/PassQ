//! Authentication module

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::env;
use chrono::{Utc, Duration};
use regex::Regex;
use log;
use ring::rand::{SystemRandom, SecureRandom};

/// Validates and sanitizes a username
pub fn sanitize_username(username: &str) -> Result<String, String> {
    if username.is_empty() {
        log::warn!("Empty username provided");
        return Err("Username cannot be empty".to_string());
    }

    if username.len() > 50 {
        log::warn!("Username too long: {} characters", username.len());
        return Err("Username too long".to_string());
    }

    // Remove any potentially dangerous characters
    let sanitized: String = username
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    if sanitized.is_empty() {
        log::warn!("Username contains only invalid characters: {}", username);
        return Err("Username contains only invalid characters".to_string());
    }

    // Check for patterns that might indicate injection attempts
    let regex = Regex::new(r"^[\w\-]+$").unwrap();
    if !regex.is_match(&sanitized) {
        log::warn!("Username contains invalid characters: {}", username);
        return Err("Username contains invalid characters".to_string());
    }

    log::info!("Username successfully sanitized: {}", username);
    Ok(sanitized)
}

/// Validates a password strength
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.is_empty() {
        log::warn!("Empty password provided");
        return Err("Password cannot be empty".to_string());
    }

    if password.len() < 8 {
        log::warn!("Password too short: {} characters", password.len());
        return Err("Password must be at least 8 characters long".to_string());
    }

    if password.len() > 128 {
        log::warn!("Password too long: {} characters", password.len());
        return Err("Password too long".to_string());
    }

    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        log::warn!("Password missing uppercase letter");
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        log::warn!("Password missing lowercase letter");
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        log::warn!("Password missing digit");
        return Err("Password must contain at least one digit".to_string());
    }

    log::info!("Password validation successful");
    Ok(())
}

/// Sanitizes and validates a website URL
pub fn sanitize_website_url(url: &str) -> Result<String, String> {
    if url.is_empty() {
        return Ok(String::new());
    }

    if url.len() > 2048 {
        log::warn!("Website URL too long: {} characters", url.len());
        return Err("Website URL too long".to_string());
    }

    // Basic URL validation - must start with http:// or https://
    let trimmed = url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        log::warn!("Invalid URL format: {}", url);
        return Err("URL must start with http:// or https://".to_string());
    }

    // Remove any potentially dangerous characters
    let sanitized: String = trimmed
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect();

    log::info!("Website URL successfully sanitized");
    Ok(sanitized)
}

/// Sanitizes notes field
pub fn sanitize_notes(notes: &str) -> Result<String, String> {
    if notes.is_empty() {
        return Ok(String::new());
    }

    if notes.len() > 10000 {
        log::warn!("Notes too long: {} characters", notes.len());
        return Err("Notes too long (max 10000 characters)".to_string());
    }

    // Remove control characters but allow newlines and tabs
    let sanitized: String = notes
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
        .collect();

    log::info!("Notes successfully sanitized");
    Ok(sanitized)
}

/// Sanitizes OTP secret
pub fn sanitize_otp_secret(secret: &str) -> Result<String, String> {
    if secret.is_empty() {
        return Ok(String::new());
    }

    if secret.len() > 256 {
        log::warn!("OTP secret too long: {} characters", secret.len());
        return Err("OTP secret too long".to_string());
    }

    // OTP secrets should only contain base32 characters
    let sanitized: String = secret
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '=')
        .collect();

    if sanitized.is_empty() && !secret.is_empty() {
        log::warn!("OTP secret contains only invalid characters: {}", secret);
        return Err("OTP secret contains invalid characters".to_string());
    }

    log::info!("OTP secret successfully sanitized");
    Ok(sanitized)
}

/// User structure for authentication
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

/// Claims structure for JWT tokens
#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub token_type: String, // "access" or "refresh"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds until access token expires
}

/// Hashes a password with bcrypt
pub fn hash_password(password: &str) -> String {
    log::debug!("Hashing password");
    match hash(password, DEFAULT_COST) {
        Ok(hash) => {
            log::debug!("Password hashed successfully");
            hash
        }
        Err(e) => {
            log::error!("Failed to hash password: {}", e);
            panic!("Failed to hash password: {}", e)
        }
    }
}

/// Verifies a password against its hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    log::debug!("Verifying password against hash");
    match verify(password, hash) {
        Ok(result) => {
            log::debug!("Password verification successful");
            result
        }
        Err(e) => {
            log::error!("Failed to verify password: {}", e);
            false
        }
    }
}
/// Generates a JWT token for a user
#[allow(dead_code)]
pub fn generate_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    log::info!("Generating JWT token for user: {}", user_id);
    
    let expiration = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: user_id,
        exp: expiration.timestamp() as usize,
        token_type: "access".to_string(),
    };

    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => {
            log::debug!("JWT secret loaded successfully");
            secret
        }
        Err(_) => {
            log::error!("Environment variable JWT_SECRET is not set");
            return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
    };

    match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())) {
        Ok(token) => {
            log::info!("JWT token generated successfully for user: {}", user_id);
            Ok(token)
        }
        Err(e) => {
            log::error!("Failed to encode JWT token: {}", e);
            Err(e)
        }
    }
}

/// Decodes a JWT token and validates it
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    log::debug!("Validating JWT token");
    
    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => {
            log::debug!("JWT secret loaded successfully");
            secret
        }
        Err(_) => {
            log::error!("Environment variable JWT_SECRET is not set");
            return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
    };

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            log::info!("JWT token validation successful for user: {:?}", token_data.claims.sub);
            Ok(token_data.claims)
        }
        Err(e) => {
            log::error!("JWT token validation failed: {}", e);
            Err(e)
        }
    }
}

/// Extracts user ID from HTTP request cookies or Authorization header
pub fn extract_user_id_from_request(req: &actix_web::HttpRequest) -> Result<Uuid, String> {
    // First try to get token from cookie
    let token = if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies to find auth_token
            let mut found_token = None;
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(token_value) = cookie.strip_prefix("auth_token=") {
                    found_token = Some(token_value.to_string());
                    break;
                }
            }
            found_token
        } else {
            None
        }
    } else {
        None
    };
    
    // Fallback to Authorization header for backward compatibility
    let token = token.or_else(|| {
        req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string())
    });
    
    let token = token.ok_or_else(|| {
        log::warn!("Missing authentication token in both cookie and Authorization header");
        "Missing authentication token".to_string()
    })?;
    
    let claims = validate_access_token(&token).map_err(|e| {
        log::error!("JWT validation failed: {}", e);
        "Invalid token".to_string()
    })?;
    
    Ok(claims.sub)
}

/// Generate a CSRF token
pub fn generate_csrf_token() -> Result<String, String> {
    let rng = SystemRandom::new();
    let mut token_bytes = [0u8; 32];
    
    rng.fill(&mut token_bytes)
        .map_err(|_| "Failed to generate random bytes".to_string())?;
    
    // Convert to hex string
    let token = token_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    log::info!("CSRF token generated successfully");
    Ok(token)
}

/// Validate a CSRF token
#[allow(dead_code)]
pub fn validate_csrf_token(token: &str, expected_token: &str) -> bool {
    if token.is_empty() || expected_token.is_empty() {
        log::warn!("Empty CSRF token provided");
        return false;
    }
    
    if token.len() != 64 || expected_token.len() != 64 {
        log::warn!("Invalid CSRF token length");
        return false;
    }
    
    // Use constant-time comparison to prevent timing attacks
    let token_bytes = token.as_bytes();
    let expected_bytes = expected_token.as_bytes();
    
    if token_bytes.len() != expected_bytes.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (a, b) in token_bytes.iter().zip(expected_bytes.iter()) {
        result |= a ^ b;
    }
    
    let is_valid = result == 0;
    if is_valid {
        log::info!("CSRF token validation successful");
    } else {
        log::warn!("CSRF token validation failed");
    }
    
    is_valid
}

/// Generates a secure password reset token
pub fn generate_password_reset_token() -> Result<String, String> {
    log::debug!("Generating password reset token");
    
    let rng = SystemRandom::new();
    let mut token_bytes = [0u8; 32]; // 256-bit token
    
    match rng.fill(&mut token_bytes) {
        Ok(_) => {
            // Convert to hex string
            let token = token_bytes.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            
            log::info!("Password reset token generated successfully");
            Ok(token)
        }
        Err(e) => {
            log::error!("Failed to generate password reset token: {:?}", e);
            Err("Failed to generate secure token".to_string())
        }
    }
}

/// Validates if a password reset token is still valid (not expired)
pub fn is_reset_token_valid(expires_at: Option<chrono::NaiveDateTime>) -> bool {
    match expires_at {
        Some(expiry) => {
            let now = Utc::now().naive_utc();
            let is_valid = expiry > now;
            
            if is_valid {
                log::debug!("Password reset token is still valid");
            } else {
                log::warn!("Password reset token has expired");
            }
            
            is_valid
        }
        None => {
            log::warn!("No expiration time set for password reset token");
            false
        }
    }
}

/// Generates expiration time for password reset token (15 minutes from now)
pub fn generate_reset_token_expiry() -> chrono::NaiveDateTime {
    let expiry = Utc::now() + Duration::minutes(15);
    expiry.naive_utc()
}

/// Generates a token pair with short-lived access token and long-lived refresh token
pub fn generate_token_pair(user_id: Uuid) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    log::info!("Generating token pair for user: {}", user_id);
    
    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            log::error!("Environment variable JWT_SECRET is not set");
            return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
    };

    // Generate short-lived access token (15 minutes)
    let access_expiration = Utc::now() + Duration::minutes(15);
    let access_claims = Claims {
        sub: user_id,
        exp: access_expiration.timestamp() as usize,
        token_type: "access".to_string(),
    };

    let access_token = encode(&Header::default(), &access_claims, &EncodingKey::from_secret(secret.as_ref()))?;

    // Generate long-lived refresh token (7 days)
    let refresh_expiration = Utc::now() + Duration::days(7);
    let refresh_claims = Claims {
        sub: user_id,
        exp: refresh_expiration.timestamp() as usize,
        token_type: "refresh".to_string(),
    };

    let refresh_token = encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(secret.as_ref()))?;

    log::info!("Token pair generated successfully for user: {}", user_id);
    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: 900, // 15 minutes in seconds
    })
}

/// Refreshes an access token using a valid refresh token
pub fn refresh_access_token(refresh_token: &str) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    log::debug!("Refreshing access token");
    
    // Validate the refresh token
    let claims = validate_token(refresh_token)?;
    
    // Ensure it's actually a refresh token
    if claims.token_type != "refresh" {
        log::error!("Invalid token type for refresh: {}", claims.token_type);
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    
    // Generate new token pair
    generate_token_pair(claims.sub)
}

/// Validates token and ensures it's an access token
pub fn validate_access_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claims = validate_token(token)?;
    
    if claims.token_type != "access" {
        log::error!("Invalid token type for access: {}", claims.token_type);
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    }
    
    Ok(claims)
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }
    
    Ok(())
}
