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
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
