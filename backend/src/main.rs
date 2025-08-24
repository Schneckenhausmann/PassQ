//! Main entry point for the web server

mod auth;
mod crypto;
mod db;
mod mfa;
mod models;
mod schema;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;
use log;

mod handlers {
    use actix_web::{web, Error, HttpResponse};
    use uuid::Uuid;
    use crate::{auth, db, crypto, mfa, models::{UserRegistration, UserLogin, ApiResponse, NewUser, User, Password, NewPassword, PasswordRequest, PasswordMoveRequest, PasswordResponse, Folder, NewFolder, FolderRequest, Share, ShareRequest}};
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
                        let user_exists = users::table
                            .filter(users::username.eq(&sanitized_username))
                            .first::<User>(&mut conn)
                            .optional()
                            .map_err(|e| {
                                log::error!("Database error: {}", e);
                                actix_web::error::ErrorInternalServerError("Database error")
                            })?;
                        let user_exists = user_exists.is_some();
                        
                        if user_exists {
                            return Ok(HttpResponse::BadRequest().json(
                                ApiResponse::<()>::error("Username already exists".to_string())
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

    // User login handler
    pub async fn login(
        user_data: web::Json<UserLogin>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::users;
        use diesel::prelude::*;
        
        log::info!("User login attempted: {}", user_data.username);
        
        // Validate input
        match auth::sanitize_username(&user_data.username) {
            Ok(sanitized_username) => {
                // Get database connection
                let mut conn = db_pool.get().map_err(|e| {
                    log::error!("Failed to get database connection: {}", e);
                    actix_web::error::ErrorInternalServerError("Database connection error")
                })?;
                
                // Find user in database
                let user_result = users::table
                    .filter(users::username.eq(&sanitized_username))
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
                            // Generate JWT token
                            match auth::generate_token(user.id) {
                                Ok(token) => {
                                    log::info!("User {} logged in successfully", sanitized_username);
                                    Ok(HttpResponse::Ok().json(
                                        ApiResponse::<String>::success("Login successful".to_string(), Some(token))
                                    ))
                                },
                                Err(e) => {
                                    log::error!("Failed to generate token: {}", e);
                                    Ok(HttpResponse::InternalServerError().json(
                                        ApiResponse::<()>::error("Failed to generate authentication token".to_string())
                                    ))
                                }
                            }
                        } else {
                            log::warn!("Invalid password for user: {}", sanitized_username);
                            Ok(HttpResponse::Unauthorized().json(
                                ApiResponse::<()>::error("Invalid username or password".to_string())
                            ))
                        }
                    },
                    None => {
                        log::warn!("User not found: {}", sanitized_username);
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

    // Get all passwords for a user
    pub async fn get_passwords(
        req: actix_web::HttpRequest,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        use crate::schema::passwords;
        
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
        
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
                    decrypted_passwords.push(PasswordResponse {
                        id: password.id,
                        folder_id: password.folder_id,
                        website: password.website,
                        username: password.username,
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
        use crate::schema::passwords;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Encrypt the password
        let encrypted_password = crypto::encrypt_password(&password_data.password)
            .map_err(|e| {
                log::error!("Encryption error: {}", e);
                actix_web::error::ErrorInternalServerError("Encryption error")
            })?;
        
        let new_password = NewPassword {
            id: Uuid::new_v4(),
            folder_id: password_data.folder_id,
            website: password_data.website.clone(),
            username: password_data.username.clone(),
            encrypted_password,
            user_id,
            notes: password_data.notes.clone(),
            otp_secret: password_data.otp_secret.clone(),
            attachments: password_data.attachments.clone(),
        };
        
        let created_password = diesel::insert_into(passwords::table)
            .values(&new_password)
            .returning(Password::as_select())
            .get_result(&mut conn)
            .map_err(|e| {
                log::error!("Database error: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
        
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
        use crate::schema::passwords;
        
        let password_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Encrypt the password
        let encrypted_password = crypto::encrypt_password(&password_data.password)
            .map_err(|e| {
                log::error!("Encryption error: {}", e);
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
                passwords::website.eq(&password_data.website),
                passwords::username.eq(&password_data.username),
                passwords::encrypted_password.eq(encrypted_password),
                passwords::notes.eq(password_data.notes.clone()),
                passwords::otp_secret.eq(password_data.otp_secret.clone()),
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
        use crate::schema::folders;
        
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        let new_folder = NewFolder {
            id: Uuid::new_v4(),
            user_id,
            parent_folder_id: folder_data.parent_folder_id,
            name: folder_data.name.clone(),
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
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
        use crate::schema::folders;
        
        let folder_id = path.into_inner();
        let mut conn = db_pool.get().map_err(|e| {
            log::error!("Failed to get database connection: {}", e);
            actix_web::error::ErrorInternalServerError("Database connection error")
        })?;
        
        // Delete folder only if it belongs to the authenticated user
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
            log::warn!("Folder not found or access denied for user: {}", user_id);
            return Err(actix_web::error::ErrorNotFound("Folder not found"));
        }
        
        Ok(HttpResponse::Ok().json(ApiResponse::success(
            "Folder deleted successfully".to_string(),
            None::<String>
        )))
    }

    // Generate OTP code for a password entry
    pub async fn generate_otp(
        req: actix_web::HttpRequest,
        path: web::Path<Uuid>,
        db_pool: web::Data<db::DbPool>,
    ) -> Result<HttpResponse, Error> {
        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                log::warn!("Missing or invalid Authorization header");
                actix_web::error::ErrorUnauthorized("Missing or invalid Authorization header")
            })?;
        
        let claims = auth::validate_token(auth_header).map_err(|e| {
            log::error!("JWT validation failed: {}", e);
            actix_web::error::ErrorUnauthorized("Invalid token")
        })?;
        
        let user_id = claims.sub;
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
                    if fields.len() < 11 {
                        errors.push(format!("Line {}: Invalid Bitwarden format", line_num + 2));
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
    
    // Get port from environment or default to 8080
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Invalid port number");

    println!("Starting server on http://0.0.0.0:{}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::resource("/register")
                    .route(web::post().to(handlers::register))
            )
            .service(
                web::resource("/login")
                    .route(web::post().to(handlers::login))
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
