//! Enhanced Authentication Handlers with Advanced Token Management
//!
//! This module provides enhanced authentication handlers that integrate
//! the advanced token management system with the existing auth infrastructure.

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    auth::{self},
    db::DbPool,
    models::{RefreshTokenRequest, User},
    token_management::{TokenManager, EnhancedClaims},
    ip_controls,
    mfa,
    schema::users,
};
use diesel::prelude::*;
use std::sync::Arc;

/// Enhanced login request with MFA support
#[derive(Deserialize)]
pub struct EnhancedLoginRequest {
    pub username: String,
    pub password: String,
    pub mfa_code: Option<String>,
}

/// Enhanced login response with session information
#[derive(Serialize)]
pub struct EnhancedLoginResponse {
    pub success: bool,
    pub message: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub session_id: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
}

/// Enhanced refresh token response
#[derive(Serialize)]
pub struct EnhancedRefreshResponse {
    pub success: bool,
    pub message: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

/// Enhanced logout request
#[derive(Deserialize)]
pub struct EnhancedLogoutRequest {
    pub revoke_all_sessions: Option<bool>,
    pub reason: Option<String>,
}

/// Enhanced login handler with advanced token management
pub async fn enhanced_login(
    _req: HttpRequest,
    user_data: web::Json<EnhancedLoginRequest>,
    db_pool: web::Data<DbPool>,
    token_manager: web::Data<Arc<TokenManager>>,
) -> ActixResult<HttpResponse> {
    // Extract client information
    let ip_address = ip_controls::extract_client_ip(&_req)
        .map(|ip| ip.to_string());
    let user_agent = _req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Sanitize username
    let sanitized_username = match auth::sanitize_username(&user_data.username) {
        Ok(username) => username,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(EnhancedLoginResponse {
                success: false,
                message: e,
                access_token: None,
                refresh_token: None,
                session_id: None,
                expires_in: None,
                token_type: "Bearer".to_string(),
            }));
        }
    };
    
    // Get database connection
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Failed to get database connection: {}", e);
            return Ok(HttpResponse::InternalServerError().json(EnhancedLoginResponse {
                success: false,
                message: "Database connection error".to_string(),
                access_token: None,
                refresh_token: None,
                session_id: None,
                expires_in: None,
                token_type: "Bearer".to_string(),
            }));
        }
    };
    
    // Find user in database
    let user_result = users::table
        .filter(
            users::username.eq(&sanitized_username)
            .or(users::email.eq(&sanitized_username))
        )
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;
    
    match user_result {
        Some(user) => {
            // Verify password
            if !auth::verify_password(&user_data.password, &user.password_hash) {
                return Ok(HttpResponse::Unauthorized().json(EnhancedLoginResponse {
                    success: false,
                    message: "Invalid credentials".to_string(),
                    access_token: None,
                    refresh_token: None,
                    session_id: None,
                    expires_in: None,
                    token_type: "Bearer".to_string(),
                }));
            }
            
            // Check if MFA is required (check if user has mfa_secret)
            if let Some(ref mfa_secret) = user.mfa_secret {
                if let Some(ref mfa_code) = user_data.mfa_code {
                    if !mfa::verify_totp_code(mfa_secret, mfa_code) {
                        return Ok(HttpResponse::Unauthorized().json(EnhancedLoginResponse {
                            success: false,
                            message: "Invalid MFA code".to_string(),
                            access_token: None,
                            refresh_token: None,
                            session_id: None,
                            expires_in: None,
                            token_type: "Bearer".to_string(),
                        }));
                    }
                } else {
                    return Ok(HttpResponse::Unauthorized().json(EnhancedLoginResponse {
                        success: false,
                        message: "MFA code required".to_string(),
                        access_token: None,
                        refresh_token: None,
                        session_id: None,
                        expires_in: None,
                        token_type: "Bearer".to_string(),
                    }));
                }
            }
            
            // Generate session ID
            let session_id = Uuid::new_v4().to_string();
            
            // Generate enhanced token pair
            match token_manager.generate_enhanced_token_pair(
                user.id,
                session_id.clone(),
                None, // device_id - could be extracted from headers
                ip_address.clone(),
                user_agent.clone(),
            ) {
                Ok(token_pair) => {
                    Ok(HttpResponse::Ok().json(EnhancedLoginResponse {
                        success: true,
                        message: "Login successful".to_string(),
                        access_token: Some(token_pair.access_token),
                        refresh_token: Some(token_pair.refresh_token),
                        session_id: Some(session_id),
                        expires_in: Some(900), // 15 minutes
                        token_type: "Bearer".to_string(),
                    }))
                }
                Err(e) => {
                    log::error!("Token generation failed: {:?}", e);
                    Ok(HttpResponse::InternalServerError().json(EnhancedLoginResponse {
                        success: false,
                        message: "Token generation failed".to_string(),
                        access_token: None,
                        refresh_token: None,
                        session_id: None,
                        expires_in: None,
                        token_type: "Bearer".to_string(),
                    }))
                }
            }
        }
        None => {
            log::warn!("Authentication failed for user {}", user_data.username);
            Ok(HttpResponse::Unauthorized().json(EnhancedLoginResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                access_token: None,
                refresh_token: None,
                session_id: None,
                expires_in: None,
                token_type: "Bearer".to_string(),
            }))
        }
    }
}

/// Enhanced refresh token handler
pub async fn enhanced_refresh_token(
    _req: HttpRequest,
    refresh_data: web::Json<RefreshTokenRequest>,
    token_manager: web::Data<Arc<TokenManager>>,
) -> ActixResult<HttpResponse> {
    let ip_address = ip_controls::extract_client_ip(&_req)
        .map(|ip| ip.to_string());
    let user_agent = _req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    match token_manager.refresh_token_pair(
        &refresh_data.refresh_token,
        None, // device_id
        ip_address,
        user_agent,
    ) {
        Ok(token_pair) => {
            Ok(HttpResponse::Ok().json(EnhancedRefreshResponse {
                success: true,
                message: "Token refreshed successfully".to_string(),
                access_token: Some(token_pair.access_token),
                refresh_token: Some(token_pair.refresh_token),
                expires_in: Some(900), // 15 minutes
            }))
        }
        Err(e) => {
            log::warn!("Token refresh failed: {:?}", e);
            Ok(HttpResponse::Unauthorized().json(EnhancedRefreshResponse {
                success: false,
                message: "Token refresh failed".to_string(),
                access_token: None,
                refresh_token: None,
                expires_in: None,
            }))
        }
    }
}

/// Enhanced logout handler with session management
pub async fn enhanced_logout(
    _req: HttpRequest,
    logout_data: web::Json<EnhancedLogoutRequest>,
    token_manager: web::Data<Arc<TokenManager>>,
    _claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    let reason = logout_data.reason.clone().unwrap_or_else(|| "User logout".to_string());
    
    if logout_data.revoke_all_sessions.unwrap_or(false) {
        // Revoke all user sessions
        token_manager.revoke_all_user_tokens(_claims.sub, reason);
        log::info!("All sessions revoked for user: {}", _claims.sub);
    } else {
        // Revoke current session only
        let _ = token_manager.revoke_session(&_claims.session_id, reason);
        log::info!("Session {} revoked for user: {}", _claims.session_id, _claims.sub);
    }
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Logout successful"
    })))
}

/// Enhanced token validation middleware
pub async fn enhanced_verify_auth(
    req: HttpRequest,
    token_manager: web::Data<Arc<TokenManager>>,
) -> ActixResult<HttpResponse> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                match token_manager.validate_enhanced_token(token) {
                    Ok(claims) => {
                        Ok(HttpResponse::Ok().json(serde_json::json!({
                            "valid": true,
                            "user_id": claims.sub,
                            "session_id": claims.session_id,
                            "expires_at": claims.exp,
                            "token_type": claims.token_type,
                            "scope": claims.scope
                        })))
                    }
                    Err(e) => {
                        log::warn!("Token validation failed: {:?}", e);
                        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                            "valid": false,
                            "error": "Invalid or expired token"
                        })))
                    }
                }
            } else {
                Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "valid": false,
                    "error": "Invalid authorization header format"
                })))
            }
        } else {
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "valid": false,
                "error": "Invalid authorization header"
            })))
        }
    } else {
        Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "valid": false,
            "error": "No authorization header provided"
        })))
    }
}

/// Get user sessions handler
pub async fn get_user_sessions(
    token_manager: web::Data<Arc<TokenManager>>,
    claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    let sessions = token_manager.get_user_sessions(claims.sub);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "sessions": sessions
    })))
}

/// Token statistics handler (admin only)
pub async fn get_token_statistics(
    token_manager: web::Data<Arc<TokenManager>>,
    _claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    // TODO: Add admin role check
    let stats = token_manager.get_token_statistics();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "statistics": stats
    })))
}