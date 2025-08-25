//! Advanced Token Management Module
//!
//! This module provides enterprise-grade JWT token management with:
//! - Short-lived access tokens (15 minutes)
//! - Long-lived refresh tokens (7 days)
//! - Token revocation list (blacklist)
//! - Token rotation and refresh mechanisms
//! - Concurrent session management
//! - Token analytics and monitoring

use actix_web::{web, HttpResponse, Result as ActixResult};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::auth::TokenPair;

/// Database connection pool type
type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Token revocation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedToken {
    pub jti: String,           // JWT ID
    pub user_id: Uuid,         // User ID
    pub token_type: String,    // "access" or "refresh"
    pub revoked_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
    pub reason: String,        // Revocation reason
}

/// Active session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: String,
    pub user_id: Uuid,
    pub access_token_jti: String,
    pub refresh_token_jti: String,
    pub created_at: chrono::DateTime<Utc>,
    pub last_activity: chrono::DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
}

/// Token analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalytics {
    pub user_id: Uuid,
    pub event_type: String,    // "issued", "refreshed", "revoked", "expired"
    pub token_type: String,    // "access" or "refresh"
    pub timestamp: chrono::DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
}

/// Enhanced JWT claims with additional security fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedClaims {
    pub sub: Uuid,             // Subject (user ID)
    pub exp: usize,            // Expiration time
    pub iat: usize,            // Issued at
    pub jti: String,           // JWT ID (unique identifier)
    pub token_type: String,    // "access" or "refresh"
    pub session_id: String,    // Session identifier
    pub aud: String,           // Audience
    pub iss: String,           // Issuer
    pub device_id: Option<String>, // Device identifier
    pub scope: Vec<String>,    // Token scope/permissions
}

/// Token refresh request
#[derive(Debug, Deserialize)]
pub struct TokenRefreshRequest {
    pub refresh_token: String,
    pub device_id: Option<String>,
}

/// Token revocation request
#[derive(Debug, Deserialize)]
pub struct TokenRevocationRequest {
    pub token: String,
    pub token_type: String, // "access", "refresh", or "all"
    pub reason: Option<String>,
}

/// Session management request
#[derive(Debug, Deserialize)]
pub struct SessionManagementRequest {
    pub action: String, // "list", "revoke", "revoke_all"
    pub session_id: Option<String>,
}

/// Token manager with in-memory caching and persistence
pub struct TokenManager {
    revoked_tokens: Arc<Mutex<HashMap<String, RevokedToken>>>,
    active_sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
    token_analytics: Arc<Mutex<Vec<TokenAnalytics>>>,
    #[allow(dead_code)]
    db_pool: DbPool,
}

impl TokenManager {
    /// Create a new token manager instance
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            revoked_tokens: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            token_analytics: Arc::new(Mutex::new(Vec::new())),
            db_pool,
        }
    }

    /// Generate enhanced token pair with additional security features
    pub fn generate_enhanced_token_pair(
        &self,
        user_id: Uuid,
        session_id: String,
        device_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenPair, jsonwebtoken::errors::Error> {
        log::info!("Generating enhanced token pair for user: {} session: {}", user_id, session_id);
        
        let secret = env::var("JWT_SECRET")
            .map_err(|_| jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

        let now = Utc::now();
        let access_jti = Uuid::new_v4().to_string();
        let refresh_jti = Uuid::new_v4().to_string();

        // Generate short-lived access token (15 minutes)
        let access_expiration = now + Duration::minutes(15);
        let access_claims = EnhancedClaims {
            sub: user_id,
            exp: access_expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: access_jti.clone(),
            token_type: "access".to_string(),
            session_id: session_id.clone(),
            aud: "passq-api".to_string(),
            iss: "passq-auth".to_string(),
            device_id: device_id.clone(),
            scope: vec!["read".to_string(), "write".to_string()],
        };

        let access_token = encode(&Header::default(), &access_claims, &EncodingKey::from_secret(secret.as_ref()))?;

        // Generate long-lived refresh token (7 days)
        let refresh_expiration = now + Duration::days(7);
        let refresh_claims = EnhancedClaims {
            sub: user_id,
            exp: refresh_expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: refresh_jti.clone(),
            token_type: "refresh".to_string(),
            session_id: session_id.clone(),
            aud: "passq-auth".to_string(),
            iss: "passq-auth".to_string(),
            device_id: device_id.clone(),
            scope: vec!["refresh".to_string()],
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(secret.as_ref()))?;

        // Store active session
        let session = ActiveSession {
            session_id: session_id.clone(),
            user_id,
            access_token_jti: access_jti,
            refresh_token_jti: refresh_jti,
            created_at: now,
            last_activity: now,
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            device_fingerprint: device_id.clone(),
        };

        if let Ok(mut sessions) = self.active_sessions.lock() {
            sessions.insert(session_id.clone(), session);
        }

        // Record analytics
        self.record_token_analytics(TokenAnalytics {
            user_id,
            event_type: "issued".to_string(),
            token_type: "pair".to_string(),
            timestamp: now,
            ip_address,
            user_agent,
            success: true,
        });

        log::info!("Enhanced token pair generated successfully for user: {} session: {}", user_id, session_id);
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: 900, // 15 minutes in seconds
        })
    }

    /// Validate enhanced token with revocation check
    pub fn validate_enhanced_token(&self, token: &str) -> Result<EnhancedClaims, jsonwebtoken::errors::Error> {
        log::debug!("Validating enhanced JWT token");
        
        let secret = env::var("JWT_SECRET")
            .map_err(|_| jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken))?;

        let token_data = decode::<EnhancedClaims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        // Check if token is revoked
        if let Ok(revoked_tokens) = self.revoked_tokens.lock() {
            if revoked_tokens.contains_key(&token_data.claims.jti) {
                log::warn!("Attempted use of revoked token: {}", token_data.claims.jti);
                return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
            }
        }

        // Update session activity
        if let Ok(mut sessions) = self.active_sessions.lock() {
            if let Some(session) = sessions.get_mut(&token_data.claims.session_id) {
                session.last_activity = Utc::now();
            }
        }

        log::info!("Enhanced JWT token validation successful for user: {:?}", token_data.claims.sub);
        Ok(token_data.claims)
    }

    /// Refresh token pair with rotation
    pub fn refresh_token_pair(
        &self,
        refresh_token: &str,
        device_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenPair, Box<dyn std::error::Error>> {
        log::info!("Refreshing token pair");

        // Validate refresh token
        let claims = self.validate_enhanced_token(refresh_token)
            .map_err(|e| {
                log::error!("Refresh token validation failed: {}", e);
                e
            })?;

        // Ensure it's a refresh token
        if claims.token_type != "refresh" {
            log::error!("Invalid token type for refresh: {}", claims.token_type);
            return Err("Invalid token type".into());
        }

        // Revoke old refresh token
        self.revoke_token(
            &claims.jti,
            claims.sub,
            "refresh".to_string(),
            "token_rotation".to_string(),
        );

        // Generate new token pair
        let new_session_id = Uuid::new_v4().to_string();
        let new_token_pair = self.generate_enhanced_token_pair(
            claims.sub,
            new_session_id,
            device_id,
            ip_address.clone(),
            user_agent.clone(),
        )?;

        // Record analytics
        self.record_token_analytics(TokenAnalytics {
            user_id: claims.sub,
            event_type: "refreshed".to_string(),
            token_type: "pair".to_string(),
            timestamp: Utc::now(),
            ip_address,
            user_agent,
            success: true,
        });

        log::info!("Token pair refreshed successfully for user: {}", claims.sub);
        Ok(new_token_pair)
    }

    /// Revoke a specific token
    pub fn revoke_token(
        &self,
        jti: &str,
        user_id: Uuid,
        token_type: String,
        reason: String,
    ) {
        log::info!("Revoking token: {} for user: {} reason: {}", jti, user_id, reason);

        let revoked_token = RevokedToken {
            jti: jti.to_string(),
            user_id,
            token_type: token_type.clone(),
            revoked_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30), // Keep revocation record for 30 days
            reason: reason.clone(),
        };

        if let Ok(mut revoked_tokens) = self.revoked_tokens.lock() {
            revoked_tokens.insert(jti.to_string(), revoked_token);
        }

        // Record analytics
        self.record_token_analytics(TokenAnalytics {
            user_id,
            event_type: "revoked".to_string(),
            token_type,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            success: true,
        });

        log::info!("Token revoked successfully: {}", jti);
    }

    /// Revoke all tokens for a user
    pub fn revoke_all_user_tokens(&self, user_id: Uuid, reason: String) {
        log::info!("Revoking all tokens for user: {} reason: {}", user_id, reason);

        // Revoke all active sessions for the user
        if let Ok(mut sessions) = self.active_sessions.lock() {
            let user_sessions: Vec<_> = sessions
                .iter()
                .filter(|(_, session)| session.user_id == user_id)
                .map(|(session_id, session)| (session_id.clone(), session.clone()))
                .collect();

            for (session_id, session) in user_sessions {
                // Revoke access token
                self.revoke_token(
                    &session.access_token_jti,
                    user_id,
                    "access".to_string(),
                    reason.clone(),
                );

                // Revoke refresh token
                self.revoke_token(
                    &session.refresh_token_jti,
                    user_id,
                    "refresh".to_string(),
                    reason.clone(),
                );

                // Remove session
                sessions.remove(&session_id);
            }
        }

        log::info!("All tokens revoked for user: {}", user_id);
    }

    /// Get active sessions for a user
    pub fn get_user_sessions(&self, user_id: Uuid) -> Vec<ActiveSession> {
        if let Ok(sessions) = self.active_sessions.lock() {
            sessions
                .values()
                .filter(|session| session.user_id == user_id)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Revoke a specific session
    pub fn revoke_session(&self, session_id: &str, reason: String) -> Result<(), String> {
        log::info!("Revoking session: {} reason: {}", session_id, reason);

        if let Ok(mut sessions) = self.active_sessions.lock() {
            if let Some(session) = sessions.remove(session_id) {
                // Revoke associated tokens
                self.revoke_token(
                    &session.access_token_jti,
                    session.user_id,
                    "access".to_string(),
                    reason.clone(),
                );

                self.revoke_token(
                    &session.refresh_token_jti,
                    session.user_id,
                    "refresh".to_string(),
                    reason,
                );

                log::info!("Session revoked successfully: {}", session_id);
                Ok(())
            } else {
                log::warn!("Session not found: {}", session_id);
                Err("Session not found".to_string())
            }
        } else {
            Err("Failed to access sessions".to_string())
        }
    }

    /// Clean up expired tokens and sessions
    pub fn cleanup_expired_tokens(&self) {
        log::info!("Cleaning up expired tokens and sessions");
        let now = Utc::now();

        // Clean up expired revoked tokens
        if let Ok(mut revoked_tokens) = self.revoked_tokens.lock() {
            revoked_tokens.retain(|_, token| token.expires_at > now);
        }

        // Clean up expired sessions (inactive for more than 30 days)
        if let Ok(mut sessions) = self.active_sessions.lock() {
            let cutoff = now - Duration::days(30);
            sessions.retain(|_, session| session.last_activity > cutoff);
        }

        // Clean up old analytics (keep only last 90 days)
        if let Ok(mut analytics) = self.token_analytics.lock() {
            let cutoff = now - Duration::days(90);
            analytics.retain(|entry| entry.timestamp > cutoff);
        }

        log::info!("Token cleanup completed");
    }

    /// Record token analytics
    fn record_token_analytics(&self, analytics: TokenAnalytics) {
        if let Ok(mut analytics_vec) = self.token_analytics.lock() {
            analytics_vec.push(analytics);
        }
    }

    /// Get token analytics for a user
    pub fn get_user_analytics(&self, user_id: Uuid, days: i64) -> Vec<TokenAnalytics> {
        let cutoff = Utc::now() - Duration::days(days);
        
        if let Ok(analytics) = self.token_analytics.lock() {
            analytics
                .iter()
                .filter(|entry| entry.user_id == user_id && entry.timestamp > cutoff)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get system-wide token statistics
    pub fn get_token_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();

        if let Ok(sessions) = self.active_sessions.lock() {
            stats.insert("active_sessions".to_string(), sessions.len() as u64);
        }

        if let Ok(revoked_tokens) = self.revoked_tokens.lock() {
            stats.insert("revoked_tokens".to_string(), revoked_tokens.len() as u64);
        }

        if let Ok(analytics) = self.token_analytics.lock() {
            stats.insert("total_events".to_string(), analytics.len() as u64);
            
            let recent_events = analytics
                .iter()
                .filter(|entry| entry.timestamp > Utc::now() - Duration::hours(24))
                .count();
            stats.insert("events_24h".to_string(), recent_events as u64);
        }

        stats
    }
}

/// Token refresh endpoint
pub async fn refresh_token(
    token_manager: web::Data<Arc<TokenManager>>,
    req: web::Json<TokenRefreshRequest>,
) -> ActixResult<HttpResponse> {
    log::info!("Token refresh request received");

    match token_manager.refresh_token_pair(
        &req.refresh_token,
        req.device_id.clone(),
        None, // IP address would be extracted from request headers
        None, // User agent would be extracted from request headers
    ) {
        Ok(token_pair) => {
            log::info!("Token refresh successful");
            Ok(HttpResponse::Ok().json(token_pair))
        }
        Err(e) => {
            log::error!("Token refresh failed: {}", e);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Token refresh failed",
                "message": "Invalid or expired refresh token"
            })))
        }
    }
}

/// Token revocation endpoint
pub async fn revoke_token(
    token_manager: web::Data<Arc<TokenManager>>,
    req: web::Json<TokenRevocationRequest>,
    _claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    log::info!("Token revocation request received");

    let user_id = _claims.sub;
    let reason = req.reason.clone().unwrap_or_else(|| "user_request".to_string());

    match req.token_type.as_str() {
        "access" | "refresh" => {
            // Validate and extract JTI from the provided token
            if let Ok(token_claims) = token_manager.validate_enhanced_token(&req.token) {
                token_manager.revoke_token(
                    &token_claims.jti,
                    user_id,
                    req.token_type.clone(),
                    reason,
                );
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Token revoked successfully"
                })))
            } else {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid token",
                    "message": "The provided token is invalid or malformed"
                })))
            }
        }
        "all" => {
            token_manager.revoke_all_user_tokens(user_id, reason);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "All tokens revoked successfully"
            })))
        }
        _ => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid token type",
                "message": "Token type must be 'access', 'refresh', or 'all'"
            })))
        }
    }
}

/// Session management endpoint
pub async fn manage_sessions(
    token_manager: web::Data<Arc<TokenManager>>,
    req: web::Json<SessionManagementRequest>,
    _claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    log::info!("Session management request received: {}", req.action);

    let user_id = _claims.sub;

    match req.action.as_str() {
        "list" => {
            let sessions = token_manager.get_user_sessions(user_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "sessions": sessions
            })))
        }
        "revoke" => {
            if let Some(session_id) = &req.session_id {
                match token_manager.revoke_session(session_id, "user_request".to_string()) {
                    Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "message": "Session revoked successfully"
                    }))),
                    Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Session revocation failed",
                        "message": e
                    })))
                }
            } else {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Missing session_id",
                    "message": "Session ID is required for revocation"
                })))
            }
        }
        "revoke_all" => {
            token_manager.revoke_all_user_tokens(user_id, "user_request".to_string());
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "All sessions revoked successfully"
            })))
        }
        _ => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid action",
                "message": "Action must be 'list', 'revoke', or 'revoke_all'"
            })))
        }
    }
}

/// Token analytics endpoint
pub async fn get_token_analytics(
    token_manager: web::Data<Arc<TokenManager>>,
    _claims: web::ReqData<EnhancedClaims>,
    query: web::Query<HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    log::info!("Token analytics request received");

    let user_id = _claims.sub;
    let days = query
        .get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(30);

    let analytics = token_manager.get_user_analytics(user_id, days);
    let statistics = token_manager.get_token_statistics();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "analytics": analytics,
        "statistics": statistics,
        "period_days": days
    })))
}

/// Cleanup endpoint (admin only)
pub async fn cleanup_tokens(
    token_manager: web::Data<Arc<TokenManager>>,
    _claims: web::ReqData<EnhancedClaims>,
) -> ActixResult<HttpResponse> {
    log::info!("Token cleanup request received");

    // TODO: Add admin role check
    // For now, allow any authenticated user to trigger cleanup
    
    token_manager.cleanup_expired_tokens();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Token cleanup completed successfully"
    })))
}

/// Configure token management routes
#[allow(dead_code)]
pub fn configure_token_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tokens")
            .route("/refresh", web::post().to(refresh_token))
            .route("/revoke", web::post().to(revoke_token))
            .route("/sessions", web::post().to(manage_sessions))
            .route("/analytics", web::get().to(get_token_analytics))
            .route("/cleanup", web::post().to(cleanup_tokens))
    );
}