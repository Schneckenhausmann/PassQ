//! Main entry point for the web server

#[macro_use]
mod audit;
mod auth;
mod crypto;
mod db;
mod email;
mod enhanced_auth_handlers;
mod enterprise_session_manager;
mod ip_controls;
mod key_management;
mod mfa;
mod models;
mod oauth;
mod schema;
mod sso_auth;
mod token_management;
mod zero_knowledge;

use actix_web::{web, App, HttpServer, middleware::Logger, http::header, dev::{ServiceRequest, ServiceResponse}, Error, Result};
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use dotenv::dotenv;
use std::env;
use log;
use actix_web::dev::{forward_ready, Service, Transform};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

// CSP Middleware
pub struct CspMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CspMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CspMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CspMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct CspMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CspMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        
        Box::pin(async move {
            let mut res = service.call(req).await?;
            
            // Add CSP headers
            let csp_policy = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";
            
            res.headers_mut().insert(
                header::HeaderName::from_static("content-security-policy"),
                header::HeaderValue::from_static(csp_policy),
            );
            
            // Add additional security headers
            res.headers_mut().insert(
                header::HeaderName::from_static("x-frame-options"),
                header::HeaderValue::from_static("DENY"),
            );
            
            res.headers_mut().insert(
                header::HeaderName::from_static("x-content-type-options"),
                header::HeaderValue::from_static("nosniff"),
            );
            
            res.headers_mut().insert(
                header::HeaderName::from_static("referrer-policy"),
                header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            
            Ok(res)
        })
    }
}

mod handlers {
    use actix_web::{web, Error, HttpResponse};
    use uuid::Uuid;
    use crate::{auth, db, crypto, ip_controls, mfa, models::{UserRegistration, UserLogin, ApiResponse, NewUser, User, Password, NewPassword, PasswordRequest, PasswordMoveRequest, PasswordResponse, Folder, NewFolder, FolderRequest, Share, ShareRequest, PasswordResetRequest, PasswordResetConfirm, RefreshTokenRequest, ChangePasswordRequest}};
    use diesel::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize)]
    pub struct CsvImportRequest {
        pub csv_data: String,
    }

    #[derive(Deserialize)]
    pub struct CsvExportRequest {
        pub password: String,
    }

    #[derive(Serialize)]
    pub struct CsvExportEntry {
        pub name: String,
        pub url: String,
        pub username: String,
        pub password: String,
        pub notes: String,
        pub folder: String,
    }
    
    // User registration handler
    pub async fn register(
        user_data: web::Json<UserRegistration>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users;
        
        // Validate email format
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(&user_data.email) {
            return Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error("Invalid email format".to_string())
            ));
        }
        
        // Validate input using auth module
        match auth::sanitize_username(&user_data.username) {
            Ok(sanitized_username) => {
                // Validate password strength
                match auth::validate_password(&user_data.password) {
                    Ok(_) => {
                        // Check if user already exists
                        let mut conn = db_pool.get().map_err(|e| {
                            log::error!("Failed to get database connection: {}", e);
                            actix_web::error::ErrorInternalServerError("Database connection error")
                        })?;
                        
                        use diesel::prelude::*;
                        
                        // Check if username already exists
                        let username_exists = users::table
                            .filter(users::username.eq(&sanitized_username))
                            .select(User::as_select())
                            .first(&mut conn)
                            .optional()
                            .map_err(|e| {
                                log::error!("Database error: {}", e);
                                actix_web::error::ErrorInternalServerError("Database error")
                            })?;
                        
                        if username_exists.is_some() {
                            return Ok(HttpResponse::BadRequest().json(
                                ApiResponse::<()>::error("Username already exists".to_string())
                            ));
                        }
                        
                        // Check if email already exists
                        let email_exists = users::table
                            .filter(users::email.eq(&user_data.email))
                            .select(User::as_select())
                            .first(&mut conn)
                            .optional()
                            .map_err(|e| {
                                log::error!("Database error: {}", e);
                                actix_web::error::ErrorInternalServerError("Database error")
                            })?;
                        
                        if email_exists.is_some() {
                            return Ok(HttpResponse::BadRequest().json(
                                ApiResponse::<()>::error("Email already exists".to_string())
                            ));
                        }
                        
                        // Hash the password
                        let password_hash = auth::hash_password(&user_data.password);
                        
                        // Create new user
                        let new_user = NewUser {
                            id: Uuid::new_v4(),
                            username: sanitized_username,
                            password_hash: password_hash,
                            salt: "".to_string(), // In a real app, you'd generate and store a proper salt
                            mfa_secret: None, // Default to no MFA initially
                            reset_token: None,
                            reset_token_expires_at: None,
                            email: user_data.email.clone(),
                            auth_method: Some("password".to_string()),
                            is_sso_user: Some(false),
                            sso_display_name: None,
                            sso_avatar_url: None,
                        };
                        
                        // Insert user into database
                        diesel::insert_into(users::table)
                            .values(&new_user)
                            .execute(&mut conn)
                            .map_err(|e| {
                                log::error!("Failed to create user in database: {}", e);
                                actix_web::error::ErrorInternalServerError("Database error")
                            })?;
                        
                        log::info!("User registered successfully: {}", new_user.username);
                        
                        Ok(HttpResponse::Ok().json(
                            ApiResponse::<()>::success("User registered successfully".to_string(), None)
                        ))
                    }
                    Err(error_msg) => {
                        Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error(format!("Password validation failed: {}", error_msg))
                        ))
                    }
                }
            }
            Err(error_msg) => {
                Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(format!("Username validation failed: {}", error_msg))
                ))
            }
        }
    }

    // User auth verification handler
    pub async fn verify_auth(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users;
        use diesel::prelude::*;
        
        match auth::extract_user_id_from_request(&req) {
            Ok(user_id) => {
                // Get database connection
                let mut conn = db_pool.get().map_err(|e| {
                    log::error!("Failed to get database connection: {}", e);
                    actix_web::error::ErrorInternalServerError("Database connection error")
                })?;
                
                // Find user in database
                let user_result = users::table
                    .filter(users::id.eq(&user_id))
                    .first::<User>(&mut conn)
                    .optional()
                    .map_err(|e| {
                        log::error!("Database error: {}", e);
                        actix_web::error::ErrorInternalServerError("Database error")
                    })?;
                
                match user_result {
                    Some(user) => {
                        #[derive(serde::Serialize)]
                        struct UserInfo {
                            username: String,
                        }
                        
                        Ok(HttpResponse::Ok().json(
                            ApiResponse::success("Authenticated".to_string(), Some(UserInfo {
                                username: user.username,
                            }))
                        ))
                    },
                    None => {
                        Ok(HttpResponse::Unauthorized().json(
                            ApiResponse::<()>::error("User not found".to_string())
                        ))
                    }
                }
            },
            Err(_) => {
                Ok(HttpResponse::Unauthorized().json(
                    ApiResponse::<()>::error("Invalid authentication".to_string())
                ))
            }
        }
    }
    
    // User logout handler
    pub async fn logout() -> Result<HttpResponse, Error> {
        // Create an expired cookie to clear the auth token
        let cookie_value = "auth_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0";
        
        Ok(HttpResponse::Ok()
            .insert_header(("Set-Cookie", cookie_value))
            .json(ApiResponse::<String>::success("Logged out successfully".to_string(), None))
        )
    }

    // CSRF token generation handler
    pub async fn get_csrf_token() -> Result<HttpResponse, Error> {
        match auth::generate_csrf_token() {
            Ok(token) => {
                #[derive(serde::Serialize)]
                struct CsrfResponse {
                    csrf_token: String,
                }
                
                Ok(HttpResponse::Ok().json(
                    ApiResponse::success("CSRF token generated".to_string(), Some(CsrfResponse {
                        csrf_token: token,
                    }))
                ))
            },
            Err(error_msg) => {
                log::error!("Failed to generate CSRF token: {}", error_msg);
                Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error("Failed to generate CSRF token".to_string())
                ))
            }
        }
    }

    // User login handler with IP-based controls
    pub async fn login(
        req: actix_web::HttpRequest,
        user_data: web::Json<UserLogin>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users;
        use diesel::prelude::*;
        
        log::info!("User login attempted: {}", user_data.username);
        
        // Extract client IP address for logging
        let client_ip = ip_controls::extract_client_ip(&req);
        
        // Check IP whitelist if enabled
        if let Some(ip) = client_ip {
            let whitelist = ip_controls::IpWhitelist::from_env();
            if std::env::var("ENABLE_IP_WHITELIST").unwrap_or_else(|_| "false".to_string()) == "true" {
                if !whitelist.is_allowed(&ip) {
                    log::warn!("Login attempt from non-whitelisted IP: {}", ip);
                    return Ok(HttpResponse::Forbidden().json(
                        ApiResponse::<()>::error("Access denied from this IP address".to_string())
                    ));
                }
            }
        }
        
        // Validate input
        match auth::sanitize_username(&user_data.username) {
            Ok(sanitized_username) => {
                // Get database connection
                let mut conn = db_pool.get().map_err(|e| {
                    log::error!("Failed to get database connection: {}", e);
                    actix_web::error::ErrorInternalServerError("Database connection error")
                })?;
                
                // Find user in database by username or email
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
                        if auth::verify_password(&user_data.password, &user.password_hash) {
                            // Log the IP address for security monitoring
                            if let Some(ip) = client_ip {
                                log::info!("Successful login for user {} from IP: {}", sanitized_username, ip);
                            }
                            
                            // Generate JWT token pair
                            match auth::generate_token_pair(user.id) {
                                Ok(token_pair) => {
                                    log::info!("User {} logged in successfully", sanitized_username);
                                    
                                    // Log successful login
                                    audit_log!(&db_pool, crate::audit::AuditEventType::UserLogin, Some(user.id), &req);
                                    
                                    // Create HttpOnly cookie for access token (15 minutes)
                                    let cookie_value = format!("auth_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=900", token_pair.access_token);
                                    
                                    // Prepare response data
                                    let response_data = serde_json::json!({
                                        "access_token": token_pair.access_token,
                                        "refresh_token": token_pair.refresh_token,
                                        "expires_in": 900 // 15 minutes in seconds
                                    });
                                    
                                    // Future: Add geolocation-based alerts here
                                    
                                    Ok(HttpResponse::Ok()
                                        .insert_header(("Set-Cookie", cookie_value))
                                        .json(ApiResponse::success("Login successful".to_string(), Some(response_data)))
                                    )
                                },
                                Err(e) => {
                                    log::error!("Failed to generate token pair: {}", e);
                                    Ok(HttpResponse::InternalServerError().json(
                                        ApiResponse::<()>::error("Failed to generate authentication tokens".to_string())
                                    ))
                                }
                            }
                        } else {
                            log::warn!("Invalid password for user: {}", sanitized_username);
                            
                            // Log failed login attempt
                            audit_log!(&db_pool, crate::audit::AuditEventType::LoginFailed, Some(user.id), &req, user.id, format!("Invalid password for user: {}", sanitized_username));
                            
                            Ok(HttpResponse::Unauthorized().json(
                                ApiResponse::<()>::error("Invalid username or password".to_string())
                            ))
                        }
                    },
                    None => {
                        log::warn!("User not found: {}", sanitized_username);
                        
                        // Log failed login attempt for non-existent user
                        audit_log!(&db_pool, crate::audit::AuditEventType::LoginFailed, None, &req, Uuid::nil(), format!("User not found: {}", sanitized_username));
                        
                        Ok(HttpResponse::Unauthorized().json(
                            ApiResponse::<()>::error("Invalid username or password".to_string())
                        ))
                    }
                }
            },
            Err(e) => {
                log::warn!("Invalid username format: {}", e);
                Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(e)
                ))
            }
        }
    }

    // Refresh access token
    pub async fn refresh_token(
        refresh_data: web::Json<RefreshTokenRequest>,
        _db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        match auth::refresh_access_token(&refresh_data.refresh_token) {
            Ok(token_pair) => {
                // Create HttpOnly cookie for new access token (15 minutes)
                let cookie_value = format!("auth_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=900", token_pair.access_token);
                
                // Return new token pair and expiration info
                let response_data = serde_json::json!({
                    "access_token": token_pair.access_token,
                    "refresh_token": token_pair.refresh_token,
                    "expires_in": token_pair.expires_in
                });
                
                Ok(HttpResponse::Ok()
                    .insert_header(("Set-Cookie", cookie_value))
                    .json(ApiResponse::success("Token refreshed successfully".to_string(), Some(response_data)))
                )
            },
            Err(e) => {
                log::warn!("Failed to refresh token: {}", e);
                Ok(HttpResponse::Unauthorized().json(
                    ApiResponse::<()>::error("Invalid or expired refresh token".to_string())
                ))
            }
        }
    }

    // Request password reset
    pub async fn request_password_reset(
        reset_data: web::Json<PasswordResetRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users::dsl::*;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Database connection error: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection failed")
        })?;
        
        // Validate email format
        let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        if !email_regex.is_match(&reset_data.email) {
            return Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error("Invalid email format".to_string())
            ));
        }
        
        // Find user by email
        let user_result: Result<User, diesel::result::Error> = users
            .filter(email.eq(&reset_data.email))
            .select(User::as_select())
            .first(&mut conn);
            
        match user_result {
            Ok(user) => {
                // Generate reset token and expiry
                let reset_token_value = match auth::generate_password_reset_token() {
                    Ok(token) => token,
                    Err(e) => {
                        log::error!("Failed to generate reset token: {}", e);
                        return Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::<()>::error("Failed to process reset request".to_string())
                        ));
                    }
                };
                let token_expiry = auth::generate_reset_token_expiry();
                
                // Update user with reset token
                let update_result = diesel::update(users.filter(id.eq(user.id)))
                    .set((
                        crate::schema::users::reset_token.eq(Some(&reset_token_value)),
                        crate::schema::users::reset_token_expires_at.eq(Some(token_expiry))
                    ))
                    .execute(&mut conn);
                    
                match update_result {
                    Ok(_) => {
                        log::info!("Password reset token generated for user: {}", reset_data.email);
                        
                        // Send password reset email
                        match crate::email::EmailService::new() {
                            Ok(email_service) => {
                                match email_service.send_password_reset_email(&reset_data.email, &user.username, &reset_token_value).await {
                                    Ok(_) => {
                                        log::info!("Password reset email sent to: {}", reset_data.email);
                                    },
                                    Err(e) => {
                                        log::error!("Failed to send password reset email: {}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("Failed to initialize email service: {}", e);
                            }
                        }
                        
                        // Always return success to not reveal if user exists
                        Ok(HttpResponse::Ok().json(
                            ApiResponse::<()>::success("Password reset email sent".to_string(), None)
                        ))
                    },
                    Err(e) => {
                        log::error!("Failed to update reset token: {}", e);
                        Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::<()>::error("Failed to process reset request".to_string())
                        ))
                    }
                }
            },
            Err(_) => {
                // Don't reveal if user exists or not for security
                log::warn!("Password reset requested for non-existent user: {}", reset_data.email);
                Ok(HttpResponse::Ok().json(
                    ApiResponse::<()>::success("Password reset email sent".to_string(), None)
                ))
            }
        }
    }
    
    // Confirm password reset
    pub async fn confirm_password_reset(
        reset_data: web::Json<PasswordResetConfirm>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users::dsl::*;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Database connection error: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection failed")
        })?;
        
        // Validate new password strength
        if let Err(e) = auth::validate_password_strength(&reset_data.new_password) {
            return Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error(e)
            ));
        }
        
        // Find user by reset token
        let user_result: Result<User, diesel::result::Error> = users
            .filter(reset_token.eq(Some(&reset_data.token)))
            .first(&mut conn);
            
        match user_result {
            Ok(user) => {
                // Check if token is still valid
                if let Some(expires_at) = user.reset_token_expires_at {
                    if !auth::is_reset_token_valid(Some(expires_at)) {
                        log::warn!("Expired reset token used for user: {}", user.username);
                        return Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error("Reset token has expired".to_string())
                        ));
                    }
                } else {
                    log::warn!("Reset token without expiry for user: {}", user.username);
                    return Ok(HttpResponse::BadRequest().json(
                        ApiResponse::<()>::error("Invalid reset token".to_string())
                    ));
                }
                
                // Hash new password
                let hashed_password = auth::hash_password(&reset_data.new_password);
                
                // Update password and clear reset token
                let update_result = diesel::update(users.filter(id.eq(user.id)))
                    .set((
                        crate::schema::users::password_hash.eq(hashed_password),
                        crate::schema::users::reset_token.eq(None::<String>),
                        crate::schema::users::reset_token_expires_at.eq(None::<chrono::NaiveDateTime>)
                    ))
                    .execute(&mut conn);
                    
                match update_result {
                    Ok(_) => {
                        log::info!("Password reset completed for user: {}", user.username);
                        Ok(HttpResponse::Ok().json(
                            ApiResponse::<()>::success("Password reset successful".to_string(), None)
                        ))
                    },
                    Err(e) => {
                        log::error!("Failed to update password: {}", e);
                        Ok(HttpResponse::InternalServerError().json(
                            ApiResponse::<()>::error("Failed to reset password".to_string())
                        ))
                    }
                }
            },
            Err(_) => {
                log::warn!("Invalid reset token used: {}", reset_data.token);
                Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error("Invalid or expired reset token".to_string())
                ))
            }
        }
    }

    // Change user password handler
    pub async fn change_password(
        req: actix_web::HttpRequest,
        change_data: web::Json<ChangePasswordRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users;
        
        // Extract user ID from request
        let user_id = match auth::extract_user_id_from_request(&req) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(
                    ApiResponse::<()>::error("Authentication required".to_string())
                ));
            }
        };

        // Validate new password strength
        if let Err(e) = auth::validate_password_strength(&change_data.new_password) {
            log::warn!("Password validation failed for user {}: {}", user_id, e);
            return Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error(e)
            ));
        }

        let mut conn = match db_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                log::error!("Database connection failed: {}", e);
                return Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error("Database connection failed".to_string())
                ));
            }
        };

        // Get current user data
        let user: User = match users::table
            .filter(users::id.eq(user_id))
            .first::<User>(&mut conn)
        {
            Ok(user) => user,
            Err(diesel::NotFound) => {
                log::warn!("User not found: {}", user_id);
                return Ok(HttpResponse::NotFound().json(
                    ApiResponse::<()>::error("User not found".to_string())
                ));
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error("Database error".to_string())
                ));
            }
        };

        // Verify current password
        if !auth::verify_password(&change_data.current_password, &user.password_hash) {
            log::warn!("Invalid current password for user: {}", user_id);
            return Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error("Current password is incorrect".to_string())
            ));
        }

        // Hash new password
        let new_password_hash = auth::hash_password(&change_data.new_password);

        // Update password in database
        match diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::password_hash.eq(&new_password_hash))
            .execute(&mut conn)
        {
            Ok(_) => {
                log::info!("Password changed successfully for user: {}", user_id);
                Ok(HttpResponse::Ok().json(
                    ApiResponse::<()>::success("Password changed successfully".to_string(), None)
                ))
            }
            Err(e) => {
                log::error!("Failed to update password for user {}: {}", user_id, e);
                Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error("Failed to update password".to_string())
                ))
            }
        }
    }

    // Get all passwords for a user
    pub async fn get_passwords(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::passwords;
        
        // Extract user ID from request
        let user_id = match auth::extract_user_id_from_request(&req) {
            Ok(id) => id,
            Err(e) => {
                log::error!("Authentication failed: {}", e);
                return Err(actix_web::error::ErrorUnauthorized("Authentication failed"));
            }
        };
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Get passwords that belong to the authenticated user
        let passwords_list = passwords::table
            .filter(passwords::user_id.eq(user_id))
            .select(Password::as_select())
            .load(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Decrypt passwords and convert to response format
        let mut decrypted_passwords = Vec::new();
        for password in passwords_list {
            match crypto::decrypt_password(&password.encrypted_password) {
                Ok(decrypted_password) => {
                    // Decrypt metadata if available, otherwise use unencrypted fields
                    let website = match &password.encrypted_website {
                        Some(encrypted_data) => {
                            crypto::decrypt_metadata(encrypted_data)
                                .unwrap_or_else(|_| password.website.clone())
                        }
                        None => password.website.clone()
                    };
                    
                    let username = match &password.encrypted_username {
                        Some(encrypted_data) => {
                            crypto::decrypt_metadata(encrypted_data)
                                .unwrap_or_else(|_| password.username.clone())
                        }
                        None => password.username.clone()
                    };
                    
                    decrypted_passwords.push(PasswordResponse {
                        id: password.id,
                        folder_id: password.folder_id,
                        website,
                        username,
                        password: decrypted_password,
                        user_id: password.user_id,
                        notes: password.notes,
                        otp_secret: password.otp_secret,
                        attachments: password.attachments,
                    });
                }
                Err(e) => {
                    log::error!("Failed to decrypt password for ID {}: {}", password.id, e);
                    // Skip this password or return an error - for now, we'll skip
                    continue;
                }
            }
        }

        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Passwords retrieved successfully".to_string(),
            Some(decrypted_passwords)
        )))
    }

    // Create a new password
    pub async fn create_password(
        req: actix_web::HttpRequest,
        password_data: web::Json<PasswordRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            actix_web::error::ErrorUnauthorized(e)
        })?;
        use crate::schema::passwords;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Sanitize input fields
        let sanitized_website = match auth::sanitize_website_url(&password_data.website) {
            Ok(url) => url,
            Err(error_msg) => {
                return Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(format!("Website URL validation failed: {}", error_msg))
                ));
            }
        };
        
        let sanitized_username = match auth::sanitize_username(&password_data.username) {
            Ok(username) => username,
            Err(error_msg) => {
                return Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(format!("Username validation failed: {}", error_msg))
                ));
            }
        };
        
        let sanitized_notes = match &password_data.notes {
            Some(notes) => {
                match auth::sanitize_notes(notes) {
                    Ok(sanitized) => Some(sanitized),
                    Err(error_msg) => {
                        return Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error(format!("Notes validation failed: {}", error_msg))
                        ));
                    }
                }
            },
            None => None
        };
        
        let sanitized_otp_secret = match &password_data.otp_secret {
            Some(secret) => {
                match auth::sanitize_otp_secret(secret) {
                    Ok(otp) => if otp.is_empty() { None } else { Some(otp) },
                    Err(error_msg) => {
                        return Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error(format!("OTP secret validation failed: {}", error_msg))
                        ));
                    }
                }
            },
            None => None
        };
        
        // Encrypt the password
        let encrypted_password = crypto::encrypt_password(&password_data.password)
            .map_err(|e| {
                log::error!("Encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption error")
            })?;
        
        // Encrypt metadata (website and username)
        let encrypted_website = crypto::encrypt_metadata(&sanitized_website)
            .map_err(|e| {
                log::error!("Website encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Metadata encryption error")
            })?;
        
        let encrypted_username = crypto::encrypt_metadata(&sanitized_username)
            .map_err(|e| {
                log::error!("Username encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Metadata encryption error")
            })?;
        
        let new_password = NewPassword {
            id: Uuid::new_v4(),
            folder_id: password_data.folder_id,
            website: sanitized_website,
            username: sanitized_username,
            encrypted_password,
            user_id,
            notes: sanitized_notes,
            otp_secret: sanitized_otp_secret,
            attachments: password_data.attachments.clone(),
            encrypted_website: Some(encrypted_website),
            encrypted_username: Some(encrypted_username),
        };
        
        let created_password = diesel::insert_into(passwords::table)
            .values(&new_password)
            .returning(Password::as_select())
            .get_result(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Log password creation event
        audit_log!(&db_pool, crate::audit::AuditEventType::PasswordCreated, Some(user_id), &req, created_password.id, format!("Password created for {}", created_password.website));
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Password created successfully".to_string(),
            Some(created_password)
        )))
    }

    // Update a password
    pub async fn update_password(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        password_data: web::Json<PasswordRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            actix_web::error::ErrorUnauthorized(e)
        })?;
        use crate::schema::passwords;
        
        let password_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Sanitize input fields
        let sanitized_website = match auth::sanitize_website_url(&password_data.website) {
            Ok(url) => url,
            Err(error_msg) => {
                return Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(format!("Website URL validation failed: {}", error_msg))
                ));
            }
        };
        
        let sanitized_username = match auth::sanitize_username(&password_data.username) {
            Ok(username) => username,
            Err(error_msg) => {
                return Ok(HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error(format!("Username validation failed: {}", error_msg))
                ));
            }
        };
        
        let sanitized_notes = match &password_data.notes {
            Some(notes) => {
                match auth::sanitize_notes(notes) {
                    Ok(sanitized) => Some(sanitized),
                    Err(error_msg) => {
                        return Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error(format!("Notes validation failed: {}", error_msg))
                        ));
                    }
                }
            },
            None => None
        };
        
        let sanitized_otp_secret = match &password_data.otp_secret {
            Some(secret) => {
                match auth::sanitize_otp_secret(secret) {
                    Ok(otp) => if otp.is_empty() { None } else { Some(otp) },
                    Err(error_msg) => {
                        return Ok(HttpResponse::BadRequest().json(
                            ApiResponse::<()>::error(format!("OTP secret validation failed: {}", error_msg))
                        ));
                    }
                }
            },
            None => None
        };
        
        // Encrypt the password
        let encrypted_password = crypto::encrypt_password(&password_data.password)
            .map_err(|e| {
                log::error!("Encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption error")
            })?;
        
        // Encrypt metadata
        let encrypted_website = crypto::encrypt_metadata(&sanitized_website)
            .map_err(|e| {
                log::error!("Website encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption error")
            })?;
        
        let encrypted_username = crypto::encrypt_metadata(&sanitized_username)
            .map_err(|e| {
                log::error!("Username encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption error")
            })?;
        
        // Update password only if it belongs to the authenticated user
        let rows_affected = diesel::update(
            passwords::table
                .filter(passwords::id.eq(password_id))
                .filter(passwords::user_id.eq(user_id))
        )
            .set((
                passwords::folder_id.eq(password_data.folder_id),
                passwords::website.eq(&sanitized_website),
                passwords::username.eq(&sanitized_username),
                passwords::encrypted_password.eq(encrypted_password),
                passwords::encrypted_website.eq(Some(encrypted_website)),
                passwords::encrypted_username.eq(Some(encrypted_username)),
                passwords::notes.eq(sanitized_notes),
                passwords::otp_secret.eq(sanitized_otp_secret),
                passwords::attachments.eq(password_data.attachments.clone()),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if rows_affected == 0 {
            log::warn!("Password not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Password not found"));
        }
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Password updated successfully".to_string(),
            None::<String>
        )))
    }

    // Move a password to a different folder
    pub async fn move_password(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        move_data: web::Json<PasswordMoveRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (handles both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication required")
        })?;
        use crate::schema::passwords;
        
        let password_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Update only the folder_id without re-encrypting the password
        let rows_affected = diesel::update(
            passwords::table
                .filter(passwords::id.eq(password_id))
                .filter(passwords::user_id.eq(user_id))
        )
            .set(passwords::folder_id.eq(move_data.folder_id))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if rows_affected == 0 {
            log::warn!("Password not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Password not found"));
        }
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Password moved successfully".to_string(),
            None::<String>
        )))
    }

    // Delete a password
    pub async fn delete_password(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (handles both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication required")
        })?;
        use crate::schema::passwords;
        
        let password_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Delete password only if it belongs to the authenticated user
        let deleted_rows = diesel::delete(
            passwords::table
                .filter(passwords::id.eq(password_id))
                .filter(passwords::user_id.eq(user_id))
        )
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if deleted_rows == 0 {
            log::warn!("Password not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Password not found"));
        }
        
        // Log password deletion event
        audit_log!(&db_pool, crate::audit::AuditEventType::PasswordDeleted, Some(user_id), &req, password_id, format!("Password deleted: {}", password_id));
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Password deleted successfully".to_string(),
            None::<String>
        )))
    }

    // Get all folders for a user
    pub async fn get_folders(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (supports both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication failed")
        })?;
        use crate::schema::folders;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        let folders_list = folders::table
            .filter(folders::user_id.eq(user_id))
            .load::<Folder>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Folders retrieved successfully".to_string(),
            Some(folders_list)
        )))
    }

    // Create a new folder
    pub async fn create_folder(
        req: actix_web::HttpRequest,
        folder_data: web::Json<FolderRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (supports both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication failed")
        })?;
        use crate::schema::folders;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        let folder_name = folder_data.name.as_ref()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Folder name is required"))?;
        
        let new_folder = NewFolder {
            id: Uuid::new_v4(),
            user_id,
            parent_folder_id: folder_data.parent_folder_id,
            name: folder_name.clone(),
        };
        
        let created_folder = diesel::insert_into(folders::table)
            .values(&new_folder)
            .get_result::<Folder>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Folder created successfully".to_string(),
            Some(created_folder)
        )))
    }

    // Delete a folder
    pub async fn delete_folder(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (supports both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication failed")
        })?;
        use crate::schema::{folders, passwords, shares};
        
        let folder_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // First, verify the folder exists and belongs to the user
        let folder = folders::table
            .filter(folders::id.eq(folder_id))
            .filter(folders::user_id.eq(user_id))
            .first::<Folder>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if folder.is_none() {
            log::warn!("Folder not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Folder not found"));
        }
        
        let folder = folder.unwrap();
        
        // Move all passwords from this folder to its parent folder
        diesel::update(
            passwords::table
                .filter(passwords::folder_id.eq(folder_id))
                .filter(passwords::user_id.eq(user_id))
        )
            .set(passwords::folder_id.eq(folder.parent_folder_id))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to move passwords: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Move all subfolders to the parent folder
        diesel::update(
            folders::table
                .filter(folders::parent_folder_id.eq(folder_id))
                .filter(folders::user_id.eq(user_id))
        )
            .set(folders::parent_folder_id.eq(folder.parent_folder_id))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to move subfolders: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Delete any shares for this folder
        diesel::delete(
            shares::table
                .filter(shares::folder_id.eq(folder_id))
        )
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to delete folder shares: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Finally, delete the folder
        let deleted_rows = diesel::delete(
            folders::table
                .filter(folders::id.eq(folder_id))
                .filter(folders::user_id.eq(user_id))
        )
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if deleted_rows == 0 {
            log::warn!("Failed to delete folder for user: {}", user_id);
            return Err(actix_web::error::ErrorInternalServerError("Failed to delete folder"));
        }
        
        log::info!("Folder {} deleted successfully by user {}", folder_id, user_id);
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Folder deleted successfully".to_string(),
            None::<String>
        )))
    }

    // Update a folder
    pub async fn update_folder(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        folder_data: web::Json<FolderRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (supports both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication failed")
        })?;
        use crate::schema::folders;
        
        let folder_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Update folder only if it belongs to the authenticated user
        let updated_rows = if let Some(ref name) = folder_data.name {
            // Update name only
            diesel::update(
                folders::table
                    .filter(folders::id.eq(folder_id))
                    .filter(folders::user_id.eq(user_id))
            )
                .set(folders::name.eq(name))
                .execute(&mut conn)
                .map_err(|e| {
                    log::error!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?
        } else if folder_data.parent_folder_id.is_some() {
            // Update parent_folder_id
            diesel::update(
                folders::table
                    .filter(folders::id.eq(folder_id))
                    .filter(folders::user_id.eq(user_id))
            )
                .set(folders::parent_folder_id.eq(folder_data.parent_folder_id))
                .execute(&mut conn)
                .map_err(|e| {
                    log::error!("Database error: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?
        } else {
            return Err(actix_web::error::ErrorBadRequest("No fields to update"));
        };
        
        if updated_rows == 0 {
            log::warn!("Folder not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Folder not found"));
        }
        
        // Fetch the updated folder to return it
        let updated_folder = folders::table
            .filter(folders::id.eq(folder_id))
            .filter(folders::user_id.eq(user_id))
            .first::<Folder>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Folder updated successfully".to_string(),
            Some(updated_folder)
        )))
    }

    // Generate OTP code for a password entry
    pub async fn generate_otp(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract user ID from request (supports both cookies and Authorization header)
        let user_id = auth::extract_user_id_from_request(&req).map_err(|e| {
            log::error!("Authentication failed: {}", e);
            actix_web::error::ErrorUnauthorized("Authentication failed")
        })?;
        let password_id = path.into_inner();
        
        use crate::schema::passwords;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Get the password entry and verify ownership
        let password = passwords::table
            .filter(passwords::id.eq(password_id))
            .filter(passwords::user_id.eq(user_id))
            .first::<Password>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
            .ok_or_else(|| {
                log::warn!("Password not found or access denied for user: {}", user_id);
                actix_web::error::ErrorNotFound("Password not found")
            })?;
        
        // Check if password has OTP secret
        if let Some(otp_secret) = &password.otp_secret {
            match mfa::generate_totp_code(otp_secret) {
                Ok(code) => {
                    Ok(HttpResponse::Ok().json(ApiResponse::success(
                        "OTP code generated successfully".to_string(),
                        Some(serde_json::json!({
                            "otp_code": code,
                            "expires_in": 30
                        }))
                    )))
                }
                Err(e) => {
                    log::error!("Failed to generate OTP code: {}", e);
                    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                        "Failed to generate OTP code".to_string()
                    )))
                }
            }
        } else {
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "No OTP secret configured for this password".to_string()
            )))
        }
    }

    // Share password handler - temporarily disabled
    pub async fn share_password(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        share_data: web::Json<ShareRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::{passwords, users, shares};
        use crate::models::NewShare;
        use diesel::prelude::*;
        use chrono::{Utc, Duration};
        
        // Extract user ID from JWT token
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_uuid) => user_uuid,
            Err(e) => {
                log::error!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let password_id = path.into_inner();
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Verify the password exists and belongs to the current user
        let password_exists = passwords::table
            .filter(passwords::id.eq(password_id))
            .filter(passwords::user_id.eq(current_user_id))
            .first::<Password>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking password ownership: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if password_exists.is_none() {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Password not found or access denied".to_string())));
        }
        
        // Find the recipient user by username
        let recipient_user = users::table
            .filter(users::username.eq(&share_data.recipient_username))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error finding recipient user: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        let recipient_user = match recipient_user {
            Some(user) => user,
            None => {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Recipient user not found".to_string())));
            }
        };
        
        // Prevent sharing with yourself
        if recipient_user.id == current_user_id {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Cannot share with yourself".to_string())));
        }
        
        // Check if already shared with this user
        let existing_share = shares::table
            .filter(shares::password_id.eq(password_id))
            .filter(shares::shared_with_user_id.eq(recipient_user.id))
            .first::<Share>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking existing share: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if existing_share.is_some() {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Password already shared with this user".to_string())));
        }
        
        // Calculate expiration date
        let expires_at = share_data.expiration_days.map(|days| {
            (Utc::now() + Duration::days(days as i64)).naive_utc()
        });
        
        // Create new share
        let new_share = NewShare {
            id: Uuid::new_v4(),
            password_id: Some(password_id),
            folder_id: None,
            user_id: current_user_id,
            shared_with_user_id: recipient_user.id,
            permission_level: share_data.permission_level.clone(),
            expires_at,
            created_at: Utc::now().naive_utc(),
        };
        
        // Insert share into database
        diesel::insert_into(shares::table)
            .values(&new_share)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to create password share: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        log::info!("Password {} shared successfully with user {} by user {}", password_id, recipient_user.username, current_user_id);
        
        Ok(HttpResponse::Ok().json(ApiResponse::<()>::success("Password shared successfully".to_string(), None)))
    }

    // Share folder handler
    pub async fn share_folder(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        share_data: web::Json<ShareRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::{folders, users, shares};
        use crate::models::NewShare;
        use diesel::prelude::*;
        use chrono::{Utc, Duration};
        
        // Extract user ID from JWT token
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_uuid) => user_uuid,
            Err(e) => {
                log::error!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let folder_id = path.into_inner();
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Verify the folder exists and belongs to the current user
        let folder_exists = folders::table
            .filter(folders::id.eq(folder_id))
            .filter(folders::user_id.eq(current_user_id))
            .first::<Folder>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking folder ownership: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if folder_exists.is_none() {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Folder not found or access denied".to_string())));
        }
        
        // Find the recipient user by username
        let recipient_user = users::table
            .filter(users::username.eq(&share_data.recipient_username))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error finding recipient user: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        let recipient_user = match recipient_user {
            Some(user) => user,
            None => {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Recipient user not found".to_string())));
            }
        };
        
        // Prevent sharing with yourself
        if recipient_user.id == current_user_id {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Cannot share with yourself".to_string())));
        }
        
        // Check if already shared with this user
        let existing_share = shares::table
            .filter(shares::folder_id.eq(folder_id))
            .filter(shares::shared_with_user_id.eq(recipient_user.id))
            .first::<Share>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking existing share: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if existing_share.is_some() {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Folder already shared with this user".to_string())));
        }
        
        // Calculate expiration date
        let expires_at = share_data.expiration_days.map(|days| {
            (Utc::now() + Duration::days(days as i64)).naive_utc()
        });
        
        // Create new share
        let new_share = NewShare {
            id: Uuid::new_v4(),
            password_id: None,
            folder_id: Some(folder_id),
            user_id: current_user_id,
            shared_with_user_id: recipient_user.id,
            permission_level: share_data.permission_level.clone(),
            expires_at,
            created_at: Utc::now().naive_utc(),
        };
        
        // Insert share into database
        diesel::insert_into(shares::table)
            .values(&new_share)
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to create folder share: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        log::info!("Folder {} shared successfully with user {} by user {}", folder_id, recipient_user.username, current_user_id);
        
        Ok(HttpResponse::Ok().json(ApiResponse::<()>::success("Folder shared successfully".to_string(), None)))
    }

    // Get shared items handler
    pub async fn get_shared_items(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::shares;
        use diesel::prelude::*;
        use chrono::Utc;
        
        // Extract user ID from JWT token
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_uuid) => user_uuid,
            Err(e) => {
                log::error!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Get all shares where the current user is the recipient
        let mut all_shares = shares::table
            .filter(shares::shared_with_user_id.eq(current_user_id))
            .load::<Share>(&mut conn)
            .map_err(|e| {
                log::error!("Database error retrieving shares: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Filter out expired shares
        let now = Utc::now().naive_utc();
        all_shares.retain(|share| {
            match share.expires_at {
                Some(expiry) => expiry > now,
                None => true, // Never expires
            }
        });
        
        // Remove expired shares from database (cleanup)
        let expired_share_ids: Vec<Uuid> = shares::table
            .filter(shares::shared_with_user_id.eq(current_user_id))
            .filter(shares::expires_at.is_not_null())
            .filter(shares::expires_at.lt(now))
            .select(shares::id)
            .load::<Uuid>(&mut conn)
            .unwrap_or_default();
        
        if !expired_share_ids.is_empty() {
            diesel::delete(shares::table.filter(shares::id.eq_any(&expired_share_ids)))
                .execute(&mut conn)
                .map_err(|e| {
                    log::error!("Failed to cleanup expired shares: {}", e);
                    // Don't fail the request for cleanup errors
                })
                .ok();
            
            log::info!("Cleaned up {} expired shares for user {}", expired_share_ids.len(), current_user_id);
        }
        
        log::info!("Retrieved {} shared items for user {}", all_shares.len(), current_user_id);
        Ok(HttpResponse::Ok().json(ApiResponse::success("Shared items retrieved successfully".to_string(), Some(all_shares))))
    }
    
    // Get shared passwords with decrypted content
    pub async fn get_shared_passwords(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::shares;
        use diesel::prelude::*;
        use chrono::Utc;
        
        // Extract user ID from JWT token
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_uuid) => user_uuid,
            Err(e) => {
                log::error!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Get all password shares where the current user is the recipient and not expired
        let now = Utc::now().naive_utc();
        let password_shares = shares::table
            .inner_join(crate::schema::passwords::table)
            .filter(shares::shared_with_user_id.eq(current_user_id))
            .filter(shares::password_id.is_not_null())
            .filter(shares::expires_at.is_null().or(shares::expires_at.gt(now)))
            .select((shares::all_columns, crate::schema::passwords::all_columns))
            .load::<(Share, Password)>(&mut conn)
            .map_err(|e| {
                log::error!("Database error retrieving shared passwords: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Decrypt passwords and prepare response
        let mut decrypted_passwords = Vec::new();
        for (share, password) in password_shares {
            match crypto::decrypt_password(&password.encrypted_password) {
                Ok(decrypted_password) => {
                    let password_response = serde_json::json!({
                        "id": password.id,
                        "website": password.website,
                        "username": password.username,
                        "password": decrypted_password,
                        "notes": password.notes,
                        "otp_secret": password.otp_secret,
                        "folder_id": password.folder_id,
                        "shared_by": share.user_id,
                        "permission_level": share.permission_level,
                        "expires_at": share.expires_at
                    });
                    decrypted_passwords.push(password_response);
                },
                Err(e) => {
                    log::error!("Failed to decrypt shared password {}: {}", password.id, e);
                    // Skip this password but continue with others
                }
            }
        }
        
        log::info!("Retrieved {} shared passwords for user {}", decrypted_passwords.len(), current_user_id);
        Ok(HttpResponse::Ok().json(ApiResponse::success("Shared passwords retrieved successfully".to_string(), Some(decrypted_passwords))))
    }
    
    // Remove a share (unshare)
    pub async fn remove_share(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::shares;
        use diesel::prelude::*;
        
        // Extract user ID from JWT token
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_uuid) => user_uuid,
            Err(e) => {
                log::error!("Failed to extract user ID: {}", e);
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let share_id = path.into_inner();
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Verify the share exists and the user has permission to remove it
        // (either the owner who shared it or the recipient)
        let share = shares::table
            .filter(shares::id.eq(share_id))
            .filter(shares::user_id.eq(current_user_id).or(shares::shared_with_user_id.eq(current_user_id)))
            .first::<Share>(&mut conn)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking share ownership: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if share.is_none() {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("Share not found or access denied".to_string())));
        }
        
        // Delete the share
        diesel::delete(shares::table.filter(shares::id.eq(share_id)))
            .execute(&mut conn)
            .map_err(|e| {
                log::error!("Failed to remove share: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        log::info!("Share {} removed successfully by user {}", share_id, current_user_id);
        Ok(HttpResponse::Ok().json(ApiResponse::<()>::success("Share removed successfully".to_string(), None)))
    }

    // CSV Export handler
    pub async fn export_csv(
        req: actix_web::HttpRequest,
        export_data: web::Json<CsvExportRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::{passwords, folders, users};
        
        // Authenticate user
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_id) => user_id,
            Err(_) => {
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Verify user password before allowing export
        let user = users::table
            .filter(users::id.eq(current_user_id))
            .first::<User>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        if !auth::verify_password(&export_data.password, &user.password_hash) {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid password".to_string())));
        }
        
        // Get all passwords for the user
        let user_passwords = passwords::table
            .filter(passwords::user_id.eq(current_user_id))
            .load::<Password>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Get all folders for the user to map folder names
        let user_folders = folders::table
            .filter(folders::user_id.eq(current_user_id))
            .load::<Folder>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        // Create folder lookup map
        let folder_map: HashMap<Uuid, String> = user_folders
            .into_iter()
            .map(|f| (f.id, f.name))
            .collect();
        
        // Convert passwords to CSV format
        let mut csv_entries = Vec::new();
        for password in user_passwords {
            // Decrypt password
            let decrypted_password = match crypto::decrypt_password(&password.encrypted_password) {
                Ok(pwd) => pwd,
                Err(e) => {
                    log::error!("Failed to decrypt password: {}", e);
                    continue; // Skip this entry if decryption fails
                }
            };
            
            let folder_name = password.folder_id
                .and_then(|id| folder_map.get(&id))
                .map(|name| name.clone())
                .unwrap_or_else(|| "No Folder".to_string());
            
            csv_entries.push(CsvExportEntry {
                name: password.website.clone(),
                url: password.website,
                username: password.username,
                password: decrypted_password,
                notes: password.notes.unwrap_or_default(),
                folder: folder_name,
            });
        }
        
        // Generate CSV content
        let mut csv_content = String::from("name,url,username,password,notes,folder\n");
        for entry in csv_entries {
            csv_content.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                entry.name.replace('"', "\"\""),
                entry.url.replace('"', "\"\""),
                entry.username.replace('"', "\"\""),
                entry.password.replace('"', "\"\""),
                entry.notes.replace('"', "\"\""),
                entry.folder.replace('"', "\"\"")
            ));
        }
        
        log::info!("CSV export completed for user {}", current_user_id);
        
        Ok(HttpResponse::Ok()
            .content_type("text/csv")
            .append_header(("Content-Disposition", "attachment; filename=\"passq_export.csv\""))
            .body(csv_content))
    }
    
    // CSV Import handler
    #[derive(Debug, Clone)]
    enum CsvFormat {
        PassQ,      // name,url,username,password,notes,folder
        Bitwarden,  // folder,favorite,type,name,notes,fields,reprompt,login_uri,login_username,login_password,login_totp
        LastPass,   // url,username,password,extra,name,grouping,fav (or with totp)
        OnePassword, // Title,Website,Username,Password,One-time password,Favorite status,Archived status,Tags,Notes
        Chrome,     // name,url,username,password
        Firefox,    // url,username,password,httpRealm,formActionOrigin,guid,timeCreated,timeLastUsed,timePasswordChanged
        Dashlane,   // Username,Password,Website,Title,Note
        KeePass,    // Account,Login Name,Password,Web Site,Comments
        Kaspersky,  // Name,Website,Login,Password,Comment
    }

    fn detect_csv_format(header_line: &str) -> CsvFormat {
        let headers = parse_csv_line(header_line);
        let headers_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
        
        // Check for Bitwarden format
        if headers_lower.contains(&"folder".to_string()) && 
           headers_lower.contains(&"favorite".to_string()) && 
           headers_lower.contains(&"login_uri".to_string()) && 
           headers_lower.contains(&"login_username".to_string()) && 
           headers_lower.contains(&"login_password".to_string()) {
            return CsvFormat::Bitwarden;
        }
        
        // Check for LastPass format
        if headers_lower.contains(&"url".to_string()) && 
           headers_lower.contains(&"username".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"grouping".to_string()) && 
           headers_lower.contains(&"extra".to_string()) {
            return CsvFormat::LastPass;
        }
        
        // Check for 1Password format
        if headers_lower.contains(&"title".to_string()) && 
           headers_lower.contains(&"website".to_string()) && 
           headers_lower.contains(&"username".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"one-time password".to_string()) {
            return CsvFormat::OnePassword;
        }
        
        // Check for Firefox format
        if headers_lower.contains(&"url".to_string()) && 
           headers_lower.contains(&"username".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"httprealm".to_string()) && 
           headers_lower.contains(&"formactionorigin".to_string()) {
            return CsvFormat::Firefox;
        }
        
        // Check for KeePass format
        if headers_lower.contains(&"account".to_string()) && 
           headers_lower.contains(&"login name".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"web site".to_string()) {
            return CsvFormat::KeePass;
        }
        
        // Check for Dashlane format
        if headers_lower.contains(&"username".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"website".to_string()) && 
           headers_lower.contains(&"title".to_string()) && 
           headers_lower.contains(&"note".to_string()) {
            return CsvFormat::Dashlane;
        }
        
        // Check for Kaspersky format
        if headers_lower.contains(&"name".to_string()) && 
           headers_lower.contains(&"website".to_string()) && 
           headers_lower.contains(&"login".to_string()) && 
           headers_lower.contains(&"password".to_string()) && 
           headers_lower.contains(&"comment".to_string()) {
            return CsvFormat::Kaspersky;
        }
        
        // Check for Chrome format (simple: name,url,username,password)
        if headers_lower.len() == 4 && 
           headers_lower.contains(&"name".to_string()) && 
           headers_lower.contains(&"url".to_string()) && 
           headers_lower.contains(&"username".to_string()) && 
           headers_lower.contains(&"password".to_string()) {
            return CsvFormat::Chrome;
        }
        
        // Default to PassQ format
        CsvFormat::PassQ
    }

    pub async fn import_csv(
        req: actix_web::HttpRequest,
        import_data: web::Json<CsvImportRequest>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::{passwords, folders};
        
        // Authenticate user
        let current_user_id = match auth::extract_user_id_from_request(&req) {
            Ok(user_id) => user_id,
            Err(_) => {
                return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or missing token".to_string())));
            }
        };
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Parse CSV data
        let lines: Vec<&str> = import_data.csv_data.lines().collect();
        if lines.is_empty() {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Empty CSV data".to_string())));
        }
        
        // Detect CSV format from header
        let format = detect_csv_format(lines[0]);
        log::info!("Detected CSV format: {:?}", format);
        
        let mut imported_count = 0;
        let mut errors = Vec::new();
        
        // Get existing folders for the user
        let user_folders = folders::table
            .filter(folders::user_id.eq(current_user_id))
            .load::<Folder>(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
        let mut folder_map: HashMap<String, Uuid> = user_folders
            .into_iter()
            .map(|f| (f.name.clone(), f.id))
            .collect();
        
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            
            let fields = parse_csv_line(line);
            
            // Extract data based on detected format
            let (name, url, username, password, notes, folder_name) = match format {
                CsvFormat::Bitwarden => {
                    // Bitwarden format: folder,favorite,type,name,notes,fields,reprompt,login_uri,login_username,login_password,login_totp
                    if fields.len() < 10 {
                        errors.push(format!("Line {}: Invalid Bitwarden format - expected at least 10 fields, got {}", line_num + 2, fields.len()));
                        continue;
                    }
                    let folder = fields.get(0).unwrap_or(&String::new()).clone();
                    let name = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(4).unwrap_or(&String::new()).clone();
                    let url = fields.get(7).unwrap_or(&String::new()).clone();
                    let username = fields.get(8).unwrap_or(&String::new()).clone();
                    let password = fields.get(9).unwrap_or(&String::new()).clone();
                    (name, url, username, password, notes, if folder.is_empty() { "No Folder".to_string() } else { folder })
                },
                CsvFormat::LastPass => {
                    if fields.len() < 7 {
                        errors.push(format!("Line {}: Invalid LastPass format", line_num + 2));
                        continue;
                    }
                    let url = fields.get(0).unwrap_or(&String::new()).clone();
                    let username = fields.get(1).unwrap_or(&String::new()).clone();
                    let password = fields.get(2).unwrap_or(&String::new()).clone();
                    let notes = fields.get(3).unwrap_or(&String::new()).clone();
                    let name = fields.get(4).unwrap_or(&String::new()).clone();
                    let folder = fields.get(5).unwrap_or(&String::new()).clone();
                    (name, url, username, password, notes, if folder.is_empty() { "No Folder".to_string() } else { folder })
                },
                CsvFormat::OnePassword => {
                    if fields.len() < 9 {
                        errors.push(format!("Line {}: Invalid 1Password format", line_num + 2));
                        continue;
                    }
                    let name = fields.get(0).unwrap_or(&String::new()).clone();
                    let url = fields.get(1).unwrap_or(&String::new()).clone();
                    let username = fields.get(2).unwrap_or(&String::new()).clone();
                    let password = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(8).unwrap_or(&String::new()).clone();
                    let tags = fields.get(7).unwrap_or(&String::new()).clone();
                    let folder = if tags.is_empty() { "No Folder".to_string() } else { tags.split(',').next().unwrap_or("No Folder").to_string() };
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::Chrome => {
                    if fields.len() < 4 {
                        errors.push(format!("Line {}: Invalid Chrome format", line_num + 2));
                        continue;
                    }
                    let name = fields.get(0).unwrap_or(&String::new()).clone();
                    let url = fields.get(1).unwrap_or(&String::new()).clone();
                    let username = fields.get(2).unwrap_or(&String::new()).clone();
                    let password = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = String::new();
                    let folder = "No Folder".to_string();
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::Firefox => {
                    if fields.len() < 3 {
                        errors.push(format!("Line {}: Invalid Firefox format", line_num + 2));
                        continue;
                    }
                    let url = fields.get(0).unwrap_or(&String::new()).clone();
                    let username = fields.get(1).unwrap_or(&String::new()).clone();
                    let password = fields.get(2).unwrap_or(&String::new()).clone();
                    let name = if url.is_empty() { "Firefox Entry".to_string() } else { url.clone() };
                    let notes = String::new();
                    let folder = "No Folder".to_string();
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::Dashlane => {
                    if fields.len() < 4 {
                        errors.push(format!("Line {}: Invalid Dashlane format", line_num + 2));
                        continue;
                    }
                    let username = fields.get(0).unwrap_or(&String::new()).clone();
                    let password = fields.get(1).unwrap_or(&String::new()).clone();
                    let url = fields.get(2).unwrap_or(&String::new()).clone();
                    let name = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(4).unwrap_or(&String::new()).clone();
                    let folder = "No Folder".to_string();
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::KeePass => {
                    if fields.len() < 3 {
                        errors.push(format!("Line {}: Invalid KeePass format", line_num + 2));
                        continue;
                    }
                    let name = fields.get(0).unwrap_or(&String::new()).clone();
                    let username = fields.get(1).unwrap_or(&String::new()).clone();
                    let password = fields.get(2).unwrap_or(&String::new()).clone();
                    let url = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(4).unwrap_or(&String::new()).clone();
                    let folder = "No Folder".to_string();
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::Kaspersky => {
                    if fields.len() < 4 {
                        errors.push(format!("Line {}: Invalid Kaspersky format", line_num + 2));
                        continue;
                    }
                    let name = fields.get(0).unwrap_or(&String::new()).clone();
                    let url = fields.get(1).unwrap_or(&String::new()).clone();
                    let username = fields.get(2).unwrap_or(&String::new()).clone();
                    let password = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(4).unwrap_or(&String::new()).clone();
                    let folder = "No Folder".to_string();
                    (name, url, username, password, notes, folder)
                },
                CsvFormat::PassQ => {
                    if fields.len() < 4 {
                        errors.push(format!("Line {}: Invalid PassQ format (need at least name,url,username,password)", line_num + 2));
                        continue;
                    }
                    let name = fields.get(0).unwrap_or(&String::new()).clone();
                    let url = fields.get(1).unwrap_or(&String::new()).clone();
                    let username = fields.get(2).unwrap_or(&String::new()).clone();
                    let password = fields.get(3).unwrap_or(&String::new()).clone();
                    let notes = fields.get(4).unwrap_or(&String::new()).clone();
                    let folder_name = fields.get(5).unwrap_or(&String::from("No Folder")).clone();
                    (name, url, username, password, notes, folder_name)
                }
            };
            
            // Skip entries without essential data
            if name.is_empty() && url.is_empty() {
                errors.push(format!("Line {}: Missing both name and URL", line_num + 2));
                continue;
            }
            
            if username.is_empty() && password.is_empty() {
                errors.push(format!("Line {}: Missing both username and password", line_num + 2));
                continue;
            }
            
            // Use name as website if URL is empty, or URL as name if name is empty
             let final_name = if name.is_empty() { url.clone() } else { name.clone() };
             let final_url = if url.is_empty() { final_name.clone() } else { url.clone() };
            
            // Get or create folder
            let folder_id = if folder_name == "No Folder" || folder_name.is_empty() {
                None
            } else {
                match folder_map.get(&folder_name) {
                    Some(id) => Some(*id),
                    None => {
                        // Create new folder
                        let new_folder = NewFolder {
                            id: Uuid::new_v4(),
                            user_id: current_user_id,
                            parent_folder_id: None,
                            name: folder_name.clone(),
                        };
                        
                        match diesel::insert_into(folders::table)
                            .values(&new_folder)
                            .execute(&mut conn) {
                            Ok(_) => {
                                folder_map.insert(folder_name, new_folder.id);
                                Some(new_folder.id)
                            }
                            Err(e) => {
                                log::error!("Failed to create folder: {}", e);
                                errors.push(format!("Line {}: Failed to create folder", line_num + 2));
                                continue;
                            }
                        }
                    }
                }
            };
            
            // Encrypt password
            let encrypted_password = match crypto::encrypt_password(&password) {
                Ok(encrypted) => encrypted,
                Err(e) => {
                    log::error!("Failed to encrypt password: {}", e);
                    errors.push(format!("Line {}: Failed to encrypt password", line_num + 2));
                    continue;
                }
            };
            
            // Encrypt metadata
            let encrypted_website = match crypto::encrypt_metadata(&final_url) {
                Ok(encrypted) => encrypted,
                Err(e) => {
                    log::error!("Failed to encrypt website: {}", e);
                    errors.push(format!("Line {}: Failed to encrypt website", line_num + 2));
                    continue;
                }
            };
            
            let encrypted_username = match crypto::encrypt_metadata(&username) {
                Ok(encrypted) => encrypted,
                Err(e) => {
                    log::error!("Failed to encrypt username: {}", e);
                    errors.push(format!("Line {}: Failed to encrypt username", line_num + 2));
                    continue;
                }
            };
            
            // Create new password entry
            let new_password = NewPassword {
                id: Uuid::new_v4(),
                user_id: current_user_id,
                folder_id,
                website: final_url,
                username,
                encrypted_password,
                notes: if notes.is_empty() { None } else { Some(notes) },
                otp_secret: None,
                attachments: None,
                encrypted_website: Some(encrypted_website),
                encrypted_username: Some(encrypted_username),
            };
            
            match diesel::insert_into(passwords::table)
                .values(&new_password)
                .execute(&mut conn) {
                Ok(_) => imported_count += 1,
                Err(e) => {
                    log::error!("Failed to insert password: {}", e);
                    errors.push(format!("Line {}: Failed to save password", line_num + 2));
                }
            }
        }
        
        log::info!("CSV import completed for user {}: {} imported, {} errors", current_user_id, imported_count, errors.len());
        
        let message = if errors.is_empty() {
            format!("Successfully imported {} passwords", imported_count)
        } else {
            format!("Imported {} passwords with {} errors: {}", imported_count, errors.len(), errors.join("; "))
        };
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(message, Some(imported_count))))
    }
    
    // Helper function to parse CSV line with quoted fields
    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        // Escaped quote
                        current_field.push('"');
                        chars.next(); // Skip the second quote
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }
        
        fields.push(current_field.trim().to_string());
        fields
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging
    env_logger::init();
    log::info!("Starting Passq backend server");
    
    // Initialize database connection pool
    let db_pool = db::establish_connection();
    log::info!("Database connection pool established");
    
    // Initialize token manager
    let token_manager = std::sync::Arc::new(token_management::TokenManager::new(db_pool.clone()));
    log::info!("Token manager initialized");
    
    // Get port from environment or default to 8080
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Invalid port number");

    println!("Starting server on http://0.0.0.0:{}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")  // React development server
            .allowed_origin("http://127.0.0.1:3000")  // Alternative localhost
            .allowed_origin("http://localhost:8080")  // Alternative frontend port
            .allowed_origin("https://passq.app")      // Production domain (update as needed)
            .allowed_origin_fn(|origin, _req_head| {
                // Allow Chrome extension origins
                origin.as_bytes().starts_with(b"chrome-extension://")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "Accept"])
            .supports_credentials();

        // Rate limiting configuration
        let auth_governor_conf = GovernorConfigBuilder::default()
            .per_second(10)  // 10 requests per second for auth endpoints
            .burst_size(20)  // Allow burst of 20 requests
            .finish()
            .unwrap();

        let general_governor_conf = GovernorConfigBuilder::default()
            .per_second(30) // 30 requests per second for general endpoints
            .burst_size(50) // Allow burst of 50 requests
            .finish()
            .unwrap();
            
        App::new()
            .wrap(CspMiddleware)
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Governor::new(&general_governor_conf))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(token_manager.clone()))
            .service(
                web::resource("/register")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::register))
            )
            .service(
                web::resource("/login")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::login))
            )
            .service(
                web::resource("/auth/verify")
                    .route(web::get().to(handlers::verify_auth))
            )
            .service(
                web::resource("/auth/logout")
                    .route(web::post().to(handlers::logout))
            )
            .service(
                web::resource("/auth/csrf-token")
                    .route(web::get().to(handlers::get_csrf_token))
            )
            // Password reset endpoints
            .service(
                web::resource("/auth/password-reset/request")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::request_password_reset))
            )
            .service(
                web::resource("/auth/password-reset/confirm")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::confirm_password_reset))
            )
            // Token refresh endpoint
            .service(
                web::resource("/auth/refresh")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::refresh_token))
            )
            // Change password endpoint
            .service(
                web::resource("/auth/change-password")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(handlers::change_password))
            )
            // Token management endpoints
            .service(
                web::resource("/auth/token/refresh")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(token_management::refresh_token))
            )
            .service(
                web::resource("/auth/token/revoke")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(token_management::revoke_token))
            )
            .service(
                web::resource("/auth/sessions")
                    .route(web::post().to(token_management::manage_sessions))
            )
            .service(
                web::resource("/auth/analytics")
                    .route(web::get().to(token_management::get_token_analytics))
            )
            .service(
                web::resource("/auth/cleanup")
                    .route(web::post().to(token_management::cleanup_tokens))
            )
            // Enterprise session management endpoints
            .service(
                web::resource("/auth/enterprise/sessions")
                    .route(web::post().to(enterprise_session_manager::create_enterprise_session))
            )
            .service(
                web::resource("/auth/enterprise/sessions/validate")
                    .route(web::post().to(enterprise_session_manager::validate_enterprise_session))
            )
            .service(
                web::resource("/auth/enterprise/analytics")
                    .route(web::get().to(enterprise_session_manager::get_enterprise_analytics))
            )
            .service(
                web::resource("/auth/enterprise/cleanup")
                    .route(web::post().to(enterprise_session_manager::cleanup_enterprise_data))
            )
            // Enhanced authentication endpoints
            .service(
                web::resource("/auth/enhanced/login")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(enhanced_auth_handlers::enhanced_login))
            )
            .service(
                web::resource("/auth/enhanced/refresh")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(enhanced_auth_handlers::enhanced_refresh_token))
            )
            .service(
                web::resource("/auth/enhanced/logout")
                    .route(web::post().to(enhanced_auth_handlers::enhanced_logout))
            )
            .service(
                web::resource("/auth/enhanced/verify")
                    .route(web::get().to(enhanced_auth_handlers::enhanced_verify_auth))
            )
            .service(
                web::resource("/auth/enhanced/sessions")
                    .route(web::get().to(enhanced_auth_handlers::get_user_sessions))
            )
            .service(
                web::resource("/auth/enhanced/statistics")
                    .route(web::get().to(enhanced_auth_handlers::get_token_statistics))
            )
            // OAuth endpoints
            .service(
                web::resource("/auth/oauth/{provider}/url")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::get().to(sso_auth::get_oauth_auth_url))
            )
            .service(
                web::resource("/auth/oauth/{provider}/callback")
                    .wrap(Governor::new(&auth_governor_conf))
                    .route(web::post().to(sso_auth::handle_oauth_callback))
            )
            .service(
                web::resource("/auth/oauth/accounts")
                    .route(web::get().to(sso_auth::get_user_oauth_accounts))
            )
            .service(
                web::resource("/auth/oauth/accounts/{id}")
                    .route(web::delete().to(sso_auth::unlink_oauth_account))
            )
            // Password endpoints
            .service(
                web::resource("/passwords")
                    .route(web::get().to(handlers::get_passwords))
                    .route(web::post().to(handlers::create_password))
            )
            .service(
                web::resource("/passwords/{id}")
                    .route(web::put().to(handlers::update_password))
                    .route(web::delete().to(handlers::delete_password))
            )
            .service(
                web::resource("/passwords/{id}/move")
                    .route(web::put().to(handlers::move_password))
            )
            .service(
                web::resource("/passwords/{id}/otp")
                    .route(web::get().to(handlers::generate_otp))
            )
            // Folder endpoints
            .service(
                web::resource("/folders")
                    .route(web::get().to(handlers::get_folders))
                    .route(web::post().to(handlers::create_folder))
            )
            .service(
                web::resource("/folders/{id}")
                    .route(web::put().to(handlers::update_folder))
                    .route(web::delete().to(handlers::delete_folder))
            )
            // Sharing endpoints
            .service(
                web::resource("/passwords/{id}/share")
                    .route(web::post().to(handlers::share_password))
            )
            .service(
                web::resource("/folders/{id}/share")
                    .route(web::post().to(handlers::share_folder))
            )
            .service(
                web::resource("/shared")
                    .route(web::get().to(handlers::get_shared_items))
            )
            .service(
                web::resource("/shared/passwords")
                    .route(web::get().to(handlers::get_shared_passwords))
            )
            .service(
                web::resource("/shares/{id}")
                    .route(web::delete().to(handlers::remove_share))
            )
            // CSV endpoints
            .service(
                web::resource("/export/csv")
                    .route(web::post().to(handlers::export_csv))
            )
            .service(
                web::resource("/import/csv")
                    .route(web::post().to(handlers::import_csv))
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
