//! Authentication module

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::env;
use chrono::{Utc, Duration};
use regex::Regex;
use log;

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
pub fn generate_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    log::info!("Generating JWT token for user: {}", user_id);
    
    let expiration = Utc::now() + Duration::days(7);
    let claims = Claims {
        sub: user_id,
        exp: expiration.timestamp() as usize,
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

/// Extracts user ID from HTTP request Authorization header
pub fn extract_user_id_from_request(req: &actix_web::HttpRequest) -> Result<Uuid, String> {
    let auth_header = req.headers().get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            log::warn!("Missing or invalid Authorization header");
            "Missing or invalid Authorization header".to_string()
        })?;
    
    let claims = validate_token(auth_header).map_err(|e| {
        log::error!("JWT validation failed: {}", e);
        "Invalid token".to_string()
    })?;
    
    Ok(claims.sub)
}
