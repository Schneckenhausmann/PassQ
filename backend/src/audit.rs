//! Audit logging module with tamper protection

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use diesel::prelude::*;
use crate::db::DbPool;
use crate::schema::audit_logs;
use ring::hmac;
use ring::rand::{SystemRandom, SecureRandom};
use std::env;
use log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    UserLogin,
    UserLogout,
    UserRegistration,
    PasswordCreated,
    PasswordUpdated,
    PasswordDeleted,
    PasswordViewed,
    FolderCreated,
    FolderUpdated,
    FolderDeleted,
    ShareCreated,
    ShareRemoved,
    PasswordReset,
    TokenRefresh,
    LoginFailed,
    UnauthorizedAccess,
    DataExport,
    DataImport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = audit_logs)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub timestamp: chrono::NaiveDateTime,
    pub integrity_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub timestamp: chrono::NaiveDateTime,
    pub integrity_hash: String,
}

/// Generate HMAC for audit log integrity
fn generate_integrity_hash(log: &AuditEvent) -> Result<String, String> {
    let secret = env::var("AUDIT_SECRET")
        .map_err(|_| "AUDIT_SECRET environment variable not set".to_string())?;
    
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    
    // Create a deterministic string representation of the log
    let log_data = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        format!("{:?}", log.event_type),
        log.user_id.map(|id| id.to_string()).unwrap_or_default(),
        log.resource_id.map(|id| id.to_string()).unwrap_or_default(),
        log.ip_address.as_deref().unwrap_or_default(),
        log.user_agent.as_deref().unwrap_or_default(),
        log.details.as_deref().unwrap_or_default(),
        log.timestamp.to_rfc3339()
    );
    
    let signature = hmac::sign(&key, log_data.as_bytes());
    Ok(hex::encode(signature.as_ref()))
}

/// Verify audit log integrity
pub fn verify_log_integrity(log: &AuditLog) -> Result<bool, String> {
    let event = AuditEvent {
        event_type: parse_event_type(&log.event_type)?,
        user_id: log.user_id,
        resource_id: log.resource_id,
        ip_address: log.ip_address.clone(),
        user_agent: log.user_agent.clone(),
        details: log.details.clone(),
        timestamp: DateTime::from_naive_utc_and_offset(log.timestamp, Utc),
    };
    
    let expected_hash = generate_integrity_hash(&event)?;
    Ok(expected_hash == log.integrity_hash)
}

/// Parse event type from string
fn parse_event_type(event_type: &str) -> Result<AuditEventType, String> {
    match event_type {
        "UserLogin" => Ok(AuditEventType::UserLogin),
        "UserLogout" => Ok(AuditEventType::UserLogout),
        "UserRegistration" => Ok(AuditEventType::UserRegistration),
        "PasswordCreated" => Ok(AuditEventType::PasswordCreated),
        "PasswordUpdated" => Ok(AuditEventType::PasswordUpdated),
        "PasswordDeleted" => Ok(AuditEventType::PasswordDeleted),
        "PasswordViewed" => Ok(AuditEventType::PasswordViewed),
        "FolderCreated" => Ok(AuditEventType::FolderCreated),
        "FolderUpdated" => Ok(AuditEventType::FolderUpdated),
        "FolderDeleted" => Ok(AuditEventType::FolderDeleted),
        "ShareCreated" => Ok(AuditEventType::ShareCreated),
        "ShareRemoved" => Ok(AuditEventType::ShareRemoved),
        "PasswordReset" => Ok(AuditEventType::PasswordReset),
        "TokenRefresh" => Ok(AuditEventType::TokenRefresh),
        "LoginFailed" => Ok(AuditEventType::LoginFailed),
        "UnauthorizedAccess" => Ok(AuditEventType::UnauthorizedAccess),
        "DataExport" => Ok(AuditEventType::DataExport),
        "DataImport" => Ok(AuditEventType::DataImport),
        _ => Err(format!("Unknown event type: {}", event_type)),
    }
}

/// Log an audit event
pub async fn log_event(
    db_pool: &DbPool,
    event: AuditEvent,
) -> Result<(), String> {
    let mut conn = db_pool.get()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let integrity_hash = generate_integrity_hash(&event)?;
    
    let new_log = NewAuditLog {
        id: Uuid::new_v4(),
        event_type: format!("{:?}", event.event_type),
        user_id: event.user_id,
        resource_id: event.resource_id,
        ip_address: event.ip_address,
        user_agent: event.user_agent,
        details: event.details,
        timestamp: event.timestamp.naive_utc(),
        integrity_hash,
    };
    
    diesel::insert_into(audit_logs::table)
        .values(&new_log)
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("Failed to insert audit log: {}", e);
            format!("Failed to insert audit log: {}", e)
        })?;
    
    log::info!("Audit event logged: {:?} for user {:?}", event.event_type, event.user_id);
    Ok(())
}

/// Extract IP address from request
pub fn extract_ip_address(req: &actix_web::HttpRequest) -> Option<String> {
    // Check X-Forwarded-For header first (for proxies)
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return Some(real_ip_str.to_string());
        }
    }
    
    // Fall back to connection info
    req.connection_info().peer_addr().map(|addr| addr.to_string())
}

/// Extract User-Agent from request
pub fn extract_user_agent(req: &actix_web::HttpRequest) -> Option<String> {
    req.headers().get("User-Agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|ua| ua.to_string())
}

/// Get audit logs for a user (admin only)
pub async fn get_user_audit_logs(
    db_pool: &DbPool,
    user_id: Uuid,
    limit: Option<i64>,
) -> Result<Vec<AuditLog>, String> {
    let mut conn = db_pool.get()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let mut query = audit_logs::table
        .filter(audit_logs::user_id.eq(user_id))
        .order(audit_logs::timestamp.desc())
        .into_boxed();
    
    if let Some(limit_val) = limit {
        query = query.limit(limit_val);
    }
    
    query.load::<AuditLog>(&mut conn)
        .map_err(|e| format!("Failed to load audit logs: {}", e))
}

/// Verify integrity of all audit logs (admin function)
pub async fn verify_all_logs_integrity(
    db_pool: &DbPool,
) -> Result<(usize, usize), String> {
    let mut conn = db_pool.get()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    let logs: Vec<AuditLog> = audit_logs::table
        .load(&mut conn)
        .map_err(|e| format!("Failed to load audit logs: {}", e))?;
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for log in logs {
        match verify_log_integrity(&log) {
            Ok(true) => valid_count += 1,
            Ok(false) => {
                invalid_count += 1;
                log::warn!("Invalid audit log detected: {}", log.id);
            },
            Err(e) => {
                invalid_count += 1;
                log::error!("Error verifying audit log {}: {}", log.id, e);
            }
        }
    }
    
    Ok((valid_count, invalid_count))
}

/// Helper macro for logging audit events
#[macro_export]
macro_rules! audit_log {
    ($db_pool:expr, $event_type:expr, $user_id:expr, $req:expr) => {
        {
            let event = $crate::audit::AuditEvent {
                event_type: $event_type,
                user_id: $user_id,
                resource_id: None,
                ip_address: $crate::audit::extract_ip_address($req),
                user_agent: $crate::audit::extract_user_agent($req),
                details: None,
                timestamp: chrono::Utc::now(),
            };
            if let Err(e) = $crate::audit::log_event($db_pool, event).await {
                log::error!("Failed to log audit event: {}", e);
            }
        }
    };
    ($db_pool:expr, $event_type:expr, $user_id:expr, $req:expr, $resource_id:expr) => {
        {
            let event = $crate::audit::AuditEvent {
                event_type: $event_type,
                user_id: $user_id,
                resource_id: Some($resource_id),
                ip_address: $crate::audit::extract_ip_address($req),
                user_agent: $crate::audit::extract_user_agent($req),
                details: None,
                timestamp: chrono::Utc::now(),
            };
            if let Err(e) = $crate::audit::log_event($db_pool, event).await {
                log::error!("Failed to log audit event: {}", e);
            }
        }
    };
    ($db_pool:expr, $event_type:expr, $user_id:expr, $req:expr, $resource_id:expr, $details:expr) => {
        {
            let event = $crate::audit::AuditEvent {
                event_type: $event_type,
                user_id: $user_id,
                resource_id: Some($resource_id),
                ip_address: $crate::audit::extract_ip_address($req),
                user_agent: $crate::audit::extract_user_agent($req),
                details: Some($details),
                timestamp: chrono::Utc::now(),
            };
            if let Err(e) = $crate::audit::log_event($db_pool, event).await {
                log::error!("Failed to log audit event: {}", e);
            }
        }
    };
}