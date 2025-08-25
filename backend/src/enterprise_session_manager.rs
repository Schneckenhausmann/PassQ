//! Enterprise Session Management Module
//!
//! This module provides enterprise-grade session management with:
//! - Persistent database storage for sessions and tokens
//! - Advanced session monitoring and analytics
//! - Device trust management
//! - Configurable session limits and policies
//! - Real-time security event detection
//! - Automated threat response

use actix_web::{web, HttpResponse, Result as ActixResult};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use jsonwebtoken::{encode, decode, Header, Algorithm, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use crate::auth::TokenPair;
use crate::schema::*;
use log::{info, error};

/// Database connection pool type
type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Enhanced session information with enterprise features
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = active_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EnterpriseSession {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: Uuid,
    pub access_token_jti: String,
    pub refresh_token_jti: String,
    pub created_at: chrono::DateTime<Utc>,
    pub last_activity: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub location_country: Option<String>,
    pub location_region: Option<String>,
    pub location_city: Option<String>,
    pub is_active: bool,
    pub created_by_ip: Option<String>,
    pub last_seen_ip: Option<String>,
    pub session_flags: Option<serde_json::Value>,
}

/// Enhanced revoked token with enterprise tracking
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = revoked_tokens)]
pub struct EnterpriseRevokedToken {
    pub id: Uuid,
    pub jti: String,
    pub user_id: Uuid,
    pub session_id: Option<String>,
    pub token_type: String,
    pub revoked_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
    pub revocation_reason: String,
    pub revoked_by_user_id: Option<Uuid>,
    pub revoked_by_admin: Option<bool>,
    pub original_expiry: Option<chrono::DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Enhanced token analytics with enterprise metrics
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = token_analytics)]
pub struct EnterpriseTokenAnalytics {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_id: Option<String>,
    pub event_type: String,
    pub token_type: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub device_fingerprint: Option<String>,
    pub geolocation: Option<serde_json::Value>,
    pub risk_score: Option<i32>,
    pub additional_data: Option<serde_json::Value>,
}

/// Session security event
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = session_security_events)]
pub struct SessionSecurityEvent {
    pub id: Uuid,
    pub session_id: String,
    pub user_id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub description: Option<String>,
    pub action_taken: Option<String>,
    pub resolved: Option<bool>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

/// Session limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = session_limits)]
pub struct SessionLimits {
    pub id: Uuid,
    pub user_id: Uuid,
    pub max_concurrent_sessions: i32,
    pub max_sessions_per_device: i32,
    pub session_timeout_minutes: i32,
    pub refresh_timeout_days: i32,
    pub enforce_single_session: Option<bool>,
    pub allow_concurrent_mobile: Option<bool>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Trusted device information
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = trusted_devices)]
pub struct TrustedDevice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_fingerprint: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub first_seen: chrono::DateTime<Utc>,
    pub last_seen: chrono::DateTime<Utc>,
    pub trust_level: String,
    pub trust_score: Option<i32>,
    pub ip_addresses: Option<serde_json::Value>,
    pub user_agent_patterns: Option<serde_json::Value>,
    pub location_history: Option<serde_json::Value>,
    pub session_count: Option<i32>,
    pub last_session_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Session monitoring rule
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct SessionMonitoringRule {
    pub id: Uuid,
    pub rule_name: String,
    pub rule_type: String,
    pub enabled: bool,
    pub severity: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub threshold_value: Option<i32>,
    pub time_window_minutes: Option<i32>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub last_triggered: Option<chrono::DateTime<Utc>>,
    pub trigger_count: Option<i32>,
}

/// Enhanced JWT claims for enterprise features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseJWTClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub token_type: String,
    pub session_id: String,
    pub aud: String,
    pub iss: String,
    pub device_id: Option<String>,
    pub device_fingerprint: Option<String>,
    pub trust_level: Option<String>,
    pub scope: Vec<String>,
    pub risk_score: Option<i32>,
    pub location: Option<serde_json::Value>,
}

/// Session creation request
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: Uuid,
    pub device_fingerprint: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub location: Option<serde_json::Value>,
}

/// Session validation response
#[derive(Debug, Serialize)]
pub struct SessionValidationResponse {
    pub valid: bool,
    pub session: Option<EnterpriseSession>,
    pub security_events: Vec<SessionSecurityEvent>,
    pub risk_score: i32,
    pub trust_level: String,
    pub actions_required: Vec<String>,
}

/// Enterprise Session Manager
pub struct EnterpriseSessionManager {
    db_pool: DbPool,
    jwt_secret: String,
    issuer: String,
    audience: String,
}

impl EnterpriseSessionManager {
    /// Create a new enterprise session manager
    #[allow(dead_code)]
    pub fn new(db_pool: DbPool) -> Self {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());
        let issuer = env::var("JWT_ISSUER")
            .unwrap_or_else(|_| "passq-api".to_string());
        let audience = env::var("JWT_AUDIENCE")
            .unwrap_or_else(|_| "passq-client".to_string());

        Self {
            db_pool,
            jwt_secret,
            issuer,
            audience,
        }
    }

    /// Create a new enterprise session with comprehensive tracking
    pub async fn create_session(
        &self,
        request: CreateSessionRequest,
    ) -> Result<(EnterpriseSession, TokenPair), Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        
        // Generate session ID
        let session_id = Uuid::new_v4().to_string();
        
        // Check session limits
        self.enforce_session_limits(&mut conn, request.user_id, &request.device_fingerprint).await?;
        
        // Update device trust
        self.update_device_trust(&mut conn, &request).await?;
        
        // Generate token JTIs
        let access_jti = Uuid::new_v4().to_string();
        let refresh_jti = Uuid::new_v4().to_string();
        
        // Create session record
        let now = Utc::now();
        let session = EnterpriseSession {
            id: Uuid::new_v4(),
            session_id: session_id.clone(),
            user_id: request.user_id,
            access_token_jti: access_jti.clone(),
            refresh_token_jti: refresh_jti.clone(),
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::days(7), // Refresh token expiry
            ip_address: request.ip_address.as_ref().and_then(|ip| ip.parse().ok()),
            user_agent: request.user_agent.clone(),
            device_fingerprint: request.device_fingerprint.clone(),
            device_name: request.device_name.clone(),
            device_type: request.device_type.clone(),
            location_country: request.location.as_ref()
                .and_then(|loc| loc.get("country"))
                .and_then(|c| c.as_str())
                .map(String::from),
            location_region: request.location.as_ref()
                .and_then(|loc| loc.get("region"))
                .and_then(|r| r.as_str())
                .map(String::from),
            location_city: request.location.as_ref()
                .and_then(|loc| loc.get("city"))
                .and_then(|c| c.as_str())
                .map(String::from),
            is_active: true,
            created_by_ip: request.ip_address.as_ref().and_then(|ip| ip.parse().ok()),
            last_seen_ip: request.ip_address.as_ref().and_then(|ip| ip.parse().ok()),
            session_flags: Some(serde_json::json!({})),
        };
        
        // Insert session into database
        let inserted_session: EnterpriseSession = diesel::insert_into(active_sessions::table)
            .values(&session)
            .returning(EnterpriseSession::as_select())
            .get_result(&mut conn)?;
        
        // Generate JWT tokens
        let token_pair = self.generate_token_pair(
            request.user_id,
            &session_id,
            &access_jti,
            &refresh_jti,
            &request,
        )?;
        
        // Record analytics
        self.record_analytics(&mut conn, EnterpriseTokenAnalytics {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            session_id: Some(session_id.clone()),
            event_type: "session_created".to_string(),
            token_type: "pair".to_string(),
            timestamp: now,
            ip_address: request.ip_address.as_ref().and_then(|ip| ip.parse().ok()),
            user_agent: request.user_agent.clone(),
            success: true,
            error_code: None,
            error_message: None,
            device_fingerprint: request.device_fingerprint.clone(),
            geolocation: request.location.clone(),
            risk_score: Some(0), // Calculate based on various factors
            additional_data: Some(serde_json::json!({
                "session_id": session_id,
                "device_type": request.device_type
            })),
        }).await?;
        
        // Check for security events
        self.check_security_events(&mut conn, &inserted_session, &request).await?;
        
        info!("Created enterprise session {} for user {}", session_id, request.user_id);
        
        Ok((inserted_session, token_pair))
    }
    
    /// Validate session and token with enterprise security checks
    pub async fn validate_session(
        &self,
        token: &str,
    ) -> Result<SessionValidationResponse, Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        
        // Decode and validate JWT
        let claims = self.validate_jwt_token(token)?;
        
        // Check if token is revoked
        if self.is_token_revoked(&mut conn, &claims.jti).await? {
            return Ok(SessionValidationResponse {
                valid: false,
                session: None,
                security_events: vec![],
                risk_score: 100,
                trust_level: "blocked".to_string(),
                actions_required: vec!["token_revoked".to_string()],
            });
        }
        
        // Get session from database
        let session: Option<EnterpriseSession> = active_sessions::table
            .filter(active_sessions::session_id.eq(&claims.session_id))
            .filter(active_sessions::is_active.eq(true))
            .select(EnterpriseSession::as_select())
            .first(&mut conn)
            .optional()?;
        
        match session {
            Some(mut session) => {
                // Update last activity
                session.last_activity = Utc::now();
                diesel::update(active_sessions::table)
                    .filter(active_sessions::session_id.eq(&claims.session_id))
                    .set(active_sessions::last_activity.eq(session.last_activity))
                    .execute(&mut conn)?;
                
                // Get recent security events
                let security_events = self.get_session_security_events(&mut conn, &claims.session_id).await?;
                
                // Calculate risk score
                let risk_score = self.calculate_risk_score(&session, &security_events).await;
                
                // Get trust level
                let trust_level = self.get_device_trust_level(&mut conn, &session.user_id, &session.device_fingerprint).await?;
                
                // Determine required actions
                let actions_required = self.determine_required_actions(&session, &security_events, risk_score).await;
                
                Ok(SessionValidationResponse {
                    valid: true,
                    session: Some(session),
                    security_events,
                    risk_score,
                    trust_level,
                    actions_required,
                })
            }
            None => {
                Ok(SessionValidationResponse {
                    valid: false,
                    session: None,
                    security_events: vec![],
                    risk_score: 100,
                    trust_level: "unknown".to_string(),
                    actions_required: vec!["session_not_found".to_string()],
                })
            }
        }
    }
    
    /// Revoke session with comprehensive cleanup
    pub async fn revoke_session(
        &self,
        session_id: &str,
        reason: &str,
        revoked_by: Option<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        
        // Get session details
        let session: EnterpriseSession = active_sessions::table
            .filter(active_sessions::session_id.eq(session_id))
            .first(&mut conn)?;
        
        // Revoke associated tokens
        self.revoke_session_tokens(&mut conn, &session, reason, revoked_by).await?;
        
        // Deactivate session
        diesel::update(active_sessions::table)
            .filter(active_sessions::session_id.eq(session_id))
            .set(active_sessions::is_active.eq(false))
            .execute(&mut conn)?;
        
        // Record security event
        self.record_security_event(&mut conn, SessionSecurityEvent {
            id: Uuid::new_v4(),
            session_id: session_id.to_string(),
            user_id: session.user_id,
            event_type: "session_revoked".to_string(),
            severity: "medium".to_string(),
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            description: Some(format!("Session revoked: {}", reason)),
            action_taken: Some("session_terminated".to_string()),
            resolved: Some(true),
            resolved_at: Some(Utc::now()),
            resolved_by: revoked_by,
            metadata: Some(serde_json::json!({
                "reason": reason,
                "revoked_by": revoked_by
            })),
        }).await?;
        
        info!("Revoked enterprise session {} for user {}: {}", session_id, session.user_id, reason);
        
        Ok(())
    }
    
    /// Get comprehensive session analytics
    pub async fn get_session_analytics(
        &self,
        user_id: Option<Uuid>,
        days: i64,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        let since = Utc::now() - Duration::days(days);
        
        let mut query = token_analytics::table
            .filter(token_analytics::timestamp.ge(since))
            .into_boxed();
        
        if let Some(uid) = user_id {
            query = query.filter(token_analytics::user_id.eq(uid));
        }
        
        let analytics: Vec<EnterpriseTokenAnalytics> = query.load(&mut conn)?;
        
        // Aggregate analytics data
        let mut result = HashMap::new();
        
        // Event type distribution
        let mut event_types = HashMap::new();
        for analytic in &analytics {
            *event_types.entry(analytic.event_type.clone()).or_insert(0) += 1;
        }
        result.insert("event_types".to_string(), serde_json::to_value(event_types)?);
        
        // Success rate
        let total = analytics.len() as f64;
        let successful = analytics.iter().filter(|a| a.success).count() as f64;
        let success_rate = if total > 0.0 { successful / total } else { 0.0 };
        result.insert("success_rate".to_string(), serde_json::to_value(success_rate)?);
        
        // Risk score distribution
        let risk_scores: Vec<i32> = analytics.iter()
            .filter_map(|a| a.risk_score)
            .collect();
        let avg_risk_score = if !risk_scores.is_empty() {
            risk_scores.iter().sum::<i32>() as f64 / risk_scores.len() as f64
        } else {
            0.0
        };
        result.insert("average_risk_score".to_string(), serde_json::to_value(avg_risk_score)?);
        
        // Device type distribution
        let mut device_types = HashMap::new();
        for analytic in &analytics {
            if let Some(ref device_fp) = analytic.device_fingerprint {
                *device_types.entry(device_fp.clone()).or_insert(0) += 1;
            }
        }
        result.insert("device_types".to_string(), serde_json::to_value(device_types)?);
        
        Ok(result)
    }
    
    /// Cleanup expired sessions and tokens
    pub async fn cleanup_expired_data(&self) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        let now = Utc::now();
        
        // Delete expired sessions
        let expired_sessions = diesel::delete(
            active_sessions::table
                .filter(active_sessions::expires_at.lt(now))
                .or_filter(active_sessions::last_activity.lt(now - Duration::days(30)))
        ).execute(&mut conn)?;
        
        // Delete expired revoked tokens (keep for 30 days after expiry)
        let expired_tokens = diesel::delete(
            revoked_tokens::table
                .filter(revoked_tokens::expires_at.lt(now - Duration::days(30)))
        ).execute(&mut conn)?;
        
        // Delete old analytics (keep for 90 days)
        let old_analytics = diesel::delete(
            token_analytics::table
                .filter(token_analytics::timestamp.lt(now - Duration::days(90)))
        ).execute(&mut conn)?;
        
        // Delete old security events (keep for 180 days)
        let old_events = diesel::delete(
            session_security_events::table
                .filter(session_security_events::timestamp.lt(now - Duration::days(180)))
        ).execute(&mut conn)?;
        
        let mut result = HashMap::new();
        result.insert("expired_sessions".to_string(), expired_sessions as u64);
        result.insert("expired_tokens".to_string(), expired_tokens as u64);
        result.insert("old_analytics".to_string(), old_analytics as u64);
        result.insert("old_events".to_string(), old_events as u64);
        
        info!("Cleanup completed: {} sessions, {} tokens, {} analytics, {} events", 
              expired_sessions, expired_tokens, old_analytics, old_events);
        
        Ok(result)
    }
    
    // Private helper methods
    
    async fn enforce_session_limits(
        &self,
        conn: &mut PgConnection,
        user_id: Uuid,
        device_fingerprint: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get user session limits
        let limits: Option<SessionLimits> = session_limits::table
            .filter(session_limits::user_id.eq(user_id))
            .first(conn)
            .optional()?;
        
        let limits = limits.unwrap_or_else(|| SessionLimits {
            id: Uuid::new_v4(),
            user_id,
            max_concurrent_sessions: 5,
            max_sessions_per_device: 3,
            session_timeout_minutes: 15,
            refresh_timeout_days: 7,
            enforce_single_session: Some(false),
            allow_concurrent_mobile: Some(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        
        // Count current active sessions
        let current_sessions = active_sessions::table
            .filter(active_sessions::user_id.eq(user_id))
            .filter(active_sessions::is_active.eq(true))
            .filter(active_sessions::expires_at.gt(Utc::now()))
            .count()
            .get_result::<i64>(conn)? as i32;
        
        // Enforce concurrent session limit
        if current_sessions >= limits.max_concurrent_sessions {
            // Terminate oldest sessions
            let oldest_sessions: Vec<String> = active_sessions::table
                .filter(active_sessions::user_id.eq(user_id))
                .filter(active_sessions::is_active.eq(true))
                .order(active_sessions::last_activity.asc())
                .limit((current_sessions - limits.max_concurrent_sessions + 1) as i64)
                .select(active_sessions::session_id)
                .load(conn)?;
            
            for session_id in oldest_sessions {
                self.revoke_session(&session_id, "concurrent_session_limit", None).await?;
            }
        }
        
        // Check device-specific limits
        if let Some(ref device_fp) = device_fingerprint {
            let device_sessions = active_sessions::table
                .filter(active_sessions::user_id.eq(user_id))
                .filter(active_sessions::device_fingerprint.eq(device_fp))
                .filter(active_sessions::is_active.eq(true))
                .filter(active_sessions::expires_at.gt(Utc::now()))
                .count()
                .get_result::<i64>(conn)? as i32;
            
            if device_sessions >= limits.max_sessions_per_device {
                // Terminate oldest device sessions
                let oldest_device_sessions: Vec<String> = active_sessions::table
                    .filter(active_sessions::user_id.eq(user_id))
                    .filter(active_sessions::device_fingerprint.eq(device_fp))
                    .filter(active_sessions::is_active.eq(true))
                    .order(active_sessions::last_activity.asc())
                    .limit((device_sessions - limits.max_sessions_per_device + 1) as i64)
                    .select(active_sessions::session_id)
                    .load(conn)?;
                
                for session_id in oldest_device_sessions {
                    self.revoke_session(&session_id, "device_session_limit", None).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn update_device_trust(
        &self,
        conn: &mut PgConnection,
        request: &CreateSessionRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref device_fp) = request.device_fingerprint {
            let now = Utc::now();
            
            // Check if device exists
            let existing_device: Option<TrustedDevice> = trusted_devices::table
                .filter(trusted_devices::user_id.eq(request.user_id))
                .filter(trusted_devices::device_fingerprint.eq(device_fp))
                .first(conn)
                .optional()?;
            
            match existing_device {
                Some(mut device) => {
                    // Update existing device
                    device.last_seen = now;
                    device.session_count = Some(device.session_count.unwrap_or(0) + 1);
                    
                    // Update IP addresses
                    if let Some(ref ip) = request.ip_address {
                        let mut ips: Vec<String> = device.ip_addresses
                            .as_ref()
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        
                        if !ips.contains(ip) {
                            ips.push(ip.clone());
                            // Keep only last 10 IPs
                            if ips.len() > 10 {
                                let skip_count = ips.len() - 10;
                                ips = ips.into_iter().skip(skip_count).collect();
                            }
                            device.ip_addresses = Some(serde_json::to_value(&ips)?);
                        }
                    }
                    
                    diesel::update(trusted_devices::table)
                        .filter(trusted_devices::user_id.eq(request.user_id))
                        .filter(trusted_devices::device_fingerprint.eq(device_fp))
                        .set((
                            trusted_devices::last_seen.eq(device.last_seen),
                            trusted_devices::session_count.eq(device.session_count),
                            trusted_devices::ip_addresses.eq(device.ip_addresses),
                            trusted_devices::updated_at.eq(now),
                        ))
                        .execute(conn)?;
                }
                None => {
                    // Create new device
                    let new_device = TrustedDevice {
                        id: Uuid::new_v4(),
                        user_id: request.user_id,
                        device_fingerprint: device_fp.clone(),
                        device_name: request.device_name.clone(),
                        device_type: request.device_type.clone(),
                        first_seen: now,
                        last_seen: now,
                        trust_level: "untrusted".to_string(),
                        trust_score: Some(0),
                        ip_addresses: request.ip_address.as_ref()
                            .map(|ip| serde_json::json!([ip])),
                        user_agent_patterns: request.user_agent.as_ref()
                            .map(|ua| serde_json::json!([ua])),
                        location_history: request.location.as_ref()
                            .map(|loc| serde_json::json!([loc])),
                        session_count: Some(1),
                        last_session_id: None,
                        notes: None,
                        created_at: now,
                        updated_at: now,
                    };
                    
                    diesel::insert_into(trusted_devices::table)
                        .values(&new_device)
                        .execute(conn)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_token_pair(
        &self,
        user_id: Uuid,
        session_id: &str,
        access_jti: &str,
        refresh_jti: &str,
        request: &CreateSessionRequest,
    ) -> Result<TokenPair, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let access_exp = now + Duration::minutes(15);
        let refresh_exp = now + Duration::days(7);
        
        // Create access token claims
        let access_claims = EnterpriseJWTClaims {
            sub: user_id,
            exp: access_exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: access_jti.to_string(),
            token_type: "access".to_string(),
            session_id: session_id.to_string(),
            aud: self.audience.clone(),
            iss: self.issuer.clone(),
            device_id: request.device_fingerprint.clone(),
            device_fingerprint: request.device_fingerprint.clone(),
            trust_level: Some("untrusted".to_string()),
            scope: vec!["read".to_string(), "write".to_string()],
            risk_score: Some(0),
            location: request.location.clone(),
        };
        
        // Create refresh token claims
        let refresh_claims = EnterpriseJWTClaims {
            sub: user_id,
            exp: refresh_exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: refresh_jti.to_string(),
            token_type: "refresh".to_string(),
            session_id: session_id.to_string(),
            aud: self.audience.clone(),
            iss: self.issuer.clone(),
            device_id: request.device_fingerprint.clone(),
            device_fingerprint: request.device_fingerprint.clone(),
            trust_level: Some("untrusted".to_string()),
            scope: vec!["refresh".to_string()],
            risk_score: Some(0),
            location: request.location.clone(),
        };
        
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());
        
        let access_token = encode(&header, &access_claims, &encoding_key)?;
        let refresh_token = encode(&header, &refresh_claims, &encoding_key)?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: 900, // 15 minutes
        })
    }
    
    fn validate_jwt_token(&self, token: &str) -> Result<EnterpriseJWTClaims, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);
        
        let token_data = decode::<EnterpriseJWTClaims>(token, &decoding_key, &validation)?;
        Ok(token_data.claims)
    }
    
    async fn is_token_revoked(
        &self,
        conn: &mut PgConnection,
        jti: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let count = revoked_tokens::table
            .filter(revoked_tokens::jti.eq(jti))
            .count()
            .get_result::<i64>(conn)?;
        
        Ok(count > 0)
    }
    
    async fn record_analytics(
        &self,
        conn: &mut PgConnection,
        analytics: EnterpriseTokenAnalytics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        diesel::insert_into(token_analytics::table)
            .values(&analytics)
            .execute(conn)?;
        
        Ok(())
    }
    
    async fn record_security_event(
        &self,
        conn: &mut PgConnection,
        event: SessionSecurityEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        diesel::insert_into(session_security_events::table)
            .values(&event)
            .execute(conn)?;
        
        Ok(())
    }
    
    async fn check_security_events(
        &self,
        conn: &mut PgConnection,
        session: &EnterpriseSession,
        request: &CreateSessionRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check for concurrent logins from different locations
        if let (Some(ref current_ip), Some(ref country)) = (&request.ip_address, &session.location_country) {
            let recent_sessions = active_sessions::table
                .filter(active_sessions::user_id.eq(session.user_id))
                .filter(active_sessions::created_at.gt(Utc::now() - Duration::hours(2)))
                .filter(active_sessions::location_country.ne(country))
                .filter(active_sessions::is_active.eq(true))
                .count()
                .get_result::<i64>(conn)?;
            
            if recent_sessions > 0 {
                self.record_security_event(conn, SessionSecurityEvent {
                    id: Uuid::new_v4(),
                    session_id: session.session_id.clone(),
                    user_id: session.user_id,
                    event_type: "suspicious_location".to_string(),
                    severity: "medium".to_string(),
                    timestamp: Utc::now(),
                    ip_address: current_ip.parse().ok(),
                    user_agent: request.user_agent.clone(),
                    description: Some("Login from different location within 2 hours".to_string()),
                    action_taken: Some("logged".to_string()),
                    resolved: Some(false),
                    resolved_at: None,
                    resolved_by: None,
                    metadata: Some(serde_json::json!({
                        "current_country": country,
                        "recent_sessions": recent_sessions
                    })),
                }).await?;
            }
        }
        
        // Check for new device
        if let Some(ref device_fp) = request.device_fingerprint {
            let device_exists = trusted_devices::table
                .filter(trusted_devices::user_id.eq(session.user_id))
                .filter(trusted_devices::device_fingerprint.eq(device_fp))
                .count()
                .get_result::<i64>(conn)?;
            
            if device_exists == 0 {
                self.record_security_event(conn, SessionSecurityEvent {
                    id: Uuid::new_v4(),
                    session_id: session.session_id.clone(),
                    user_id: session.user_id,
                    event_type: "new_device".to_string(),
                    severity: "low".to_string(),
                    timestamp: Utc::now(),
                    ip_address: request.ip_address.as_ref().and_then(|ip| ip.parse().ok()),
                    user_agent: request.user_agent.clone(),
                    description: Some("Login from new device".to_string()),
                    action_taken: Some("logged".to_string()),
                    resolved: Some(false),
                    resolved_at: None,
                    resolved_by: None,
                    metadata: Some(serde_json::json!({
                        "device_fingerprint": device_fp,
                        "device_type": request.device_type
                    })),
                }).await?;
            }
        }
        
        Ok(())
    }
    
    async fn get_session_security_events(
        &self,
        conn: &mut PgConnection,
        session_id: &str,
    ) -> Result<Vec<SessionSecurityEvent>, Box<dyn std::error::Error>> {
        let events = session_security_events::table
            .filter(session_security_events::session_id.eq(session_id))
            .filter(session_security_events::timestamp.gt(Utc::now() - Duration::days(7)))
            .order(session_security_events::timestamp.desc())
            .load(conn)?;
        
        Ok(events)
    }
    
    async fn calculate_risk_score(
        &self,
        session: &EnterpriseSession,
        events: &[SessionSecurityEvent],
    ) -> i32 {
        let mut risk_score = 0;
        
        // Base risk factors
        if session.device_fingerprint.is_none() {
            risk_score += 20; // Unknown device
        }
        
        if session.location_country.is_none() {
            risk_score += 10; // Unknown location
        }
        
        // Event-based risk
        for event in events {
            match event.severity.as_str() {
                "critical" => risk_score += 40,
                "high" => risk_score += 25,
                "medium" => risk_score += 15,
                "low" => risk_score += 5,
                _ => {},
            }
        }
        
        // Session age factor
        let session_age = Utc::now().signed_duration_since(session.created_at);
        if session_age > Duration::days(7) {
            risk_score += 10; // Old session
        }
        
        // Activity factor
        let last_activity_age = Utc::now().signed_duration_since(session.last_activity);
        if last_activity_age > Duration::hours(24) {
            risk_score += 15; // Inactive session
        }
        
        // Cap at 100
        std::cmp::min(risk_score, 100)
    }
    
    async fn get_device_trust_level(
        &self,
        conn: &mut PgConnection,
        user_id: &Uuid,
        device_fingerprint: &Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(ref device_fp) = device_fingerprint {
            let device: Option<TrustedDevice> = trusted_devices::table
                .filter(trusted_devices::user_id.eq(user_id))
                .filter(trusted_devices::device_fingerprint.eq(device_fp))
                .first(conn)
                .optional()?;
            
            Ok(device.map(|d| d.trust_level).unwrap_or_else(|| "unknown".to_string()))
        } else {
            Ok("unknown".to_string())
        }
    }
    
    async fn determine_required_actions(
        &self,
        session: &EnterpriseSession,
        events: &[SessionSecurityEvent],
        risk_score: i32,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        
        if risk_score > 80 {
            actions.push("require_mfa".to_string());
            actions.push("notify_user".to_string());
        } else if risk_score > 60 {
            actions.push("require_mfa".to_string());
        } else if risk_score > 40 {
            actions.push("monitor_closely".to_string());
        }
        
        // Check for critical events
        for event in events {
            if event.severity == "critical" && !event.resolved.unwrap_or(false) {
                actions.push("immediate_review".to_string());
                break;
            }
        }
        
        // Check session age
        let session_age = Utc::now().signed_duration_since(session.created_at);
        if session_age > Duration::days(30) {
            actions.push("force_refresh".to_string());
        }
        
        actions.sort();
        actions.dedup();
        actions
    }
    
    async fn revoke_session_tokens(
        &self,
        conn: &mut PgConnection,
        session: &EnterpriseSession,
        reason: &str,
        revoked_by: Option<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        
        // Revoke access token
        let access_revocation = EnterpriseRevokedToken {
            id: Uuid::new_v4(),
            jti: session.access_token_jti.clone(),
            user_id: session.user_id,
            session_id: Some(session.session_id.clone()),
            token_type: "access".to_string(),
            revoked_at: now,
            expires_at: now + Duration::minutes(15), // Original access token expiry
            revocation_reason: reason.to_string(),
            revoked_by_user_id: revoked_by,
            revoked_by_admin: Some(revoked_by.is_some()),
            original_expiry: Some(now + Duration::minutes(15)),
            ip_address: session.last_seen_ip.clone(),
            user_agent: session.user_agent.clone(),
        };
        
        // Revoke refresh token
        let refresh_revocation = EnterpriseRevokedToken {
            id: Uuid::new_v4(),
            jti: session.refresh_token_jti.clone(),
            user_id: session.user_id,
            session_id: Some(session.session_id.clone()),
            token_type: "refresh".to_string(),
            revoked_at: now,
            expires_at: session.expires_at,
            revocation_reason: reason.to_string(),
            revoked_by_user_id: revoked_by,
            revoked_by_admin: Some(revoked_by.is_some()),
            original_expiry: Some(session.expires_at),
            ip_address: session.last_seen_ip.clone(),
            user_agent: session.user_agent.clone(),
        };
        
        diesel::insert_into(revoked_tokens::table)
            .values(&vec![access_revocation, refresh_revocation])
            .execute(conn)?;
        
        Ok(())
    }
}

// HTTP endpoint handlers

/// Create a new enterprise session
pub async fn create_enterprise_session(
    session_manager: web::Data<Arc<EnterpriseSessionManager>>,
    req: web::Json<CreateSessionRequest>,
) -> ActixResult<HttpResponse> {
    match session_manager.create_session(req.into_inner()).await {
        Ok((session, tokens)) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "session": session,
                "tokens": tokens,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to create enterprise session: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create session",
                "status": "error"
            })))
        }
    }
}

/// Validate enterprise session
pub async fn validate_enterprise_session(
    session_manager: web::Data<Arc<EnterpriseSessionManager>>,
    req: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let token = req.get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Token required"))?;
    
    match session_manager.validate_session(token).await {
        Ok(validation) => {
            Ok(HttpResponse::Ok().json(validation))
        }
        Err(e) => {
            error!("Failed to validate enterprise session: {}", e);
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid session",
                "status": "error"
            })))
        }
    }
}

/// Get enterprise session analytics
pub async fn get_enterprise_analytics(
    session_manager: web::Data<Arc<EnterpriseSessionManager>>,
    query: web::Query<HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let user_id = query.get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok());
    let days = query.get("days")
        .and_then(|d| d.parse::<i64>().ok())
        .unwrap_or(30);
    
    match session_manager.get_session_analytics(user_id, days).await {
        Ok(analytics) => {
            Ok(HttpResponse::Ok().json(analytics))
        }
        Err(e) => {
            error!("Failed to get enterprise analytics: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get analytics",
                "status": "error"
            })))
        }
    }
}

/// Cleanup expired enterprise data
pub async fn cleanup_enterprise_data(
    session_manager: web::Data<Arc<EnterpriseSessionManager>>,
) -> ActixResult<HttpResponse> {
    match session_manager.cleanup_expired_data().await {
        Ok(stats) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "cleanup_stats": stats,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to cleanup enterprise data: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to cleanup data",
                "status": "error"
            })))
        }
    }
}

/// Configure enterprise session routes
#[allow(dead_code)]
pub fn configure_enterprise_session_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/enterprise/sessions")
            .route("/create", web::post().to(create_enterprise_session))
            .route("/validate", web::post().to(validate_enterprise_session))
            .route("/analytics", web::get().to(get_enterprise_analytics))
            .route("/cleanup", web::post().to(cleanup_enterprise_data))
    );
}