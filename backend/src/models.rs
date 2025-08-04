use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

#[derive(Queryable, Serialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub mfa_secret: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub mfa_secret: Option<String>,
}

#[derive(Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

// Password models
#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = crate::schema::passwords)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Password {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub user_id: Uuid,
    pub notes: Option<String>,
    pub otp_secret: Option<String>,
    pub attachments: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::passwords)]
pub struct NewPassword {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub user_id: Uuid,
    pub notes: Option<String>,
    pub otp_secret: Option<String>,
    pub attachments: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PasswordRequest {
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub password: String,
    pub notes: Option<String>,
    pub otp_secret: Option<String>,
    pub attachments: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PasswordMoveRequest {
    pub folder_id: Option<Uuid>,
}

// Response struct for decrypted passwords
#[derive(Serialize, Debug)]
pub struct PasswordResponse {
    pub id: Uuid,
    pub folder_id: Option<Uuid>,
    pub website: String,
    pub username: String,
    pub password: String, // Decrypted password
    pub user_id: Uuid,
    pub notes: Option<String>,
    pub otp_secret: Option<String>,
    pub attachments: Option<serde_json::Value>,
}

// Folder models
#[derive(Queryable, Serialize, Debug)]
pub struct Folder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::folders)]
pub struct NewFolder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
}

#[derive(Deserialize)]
pub struct FolderRequest {
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
}

// Share models
#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = crate::schema::shares)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Share {
    pub id: Uuid,
    pub password_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub shared_with_user_id: Uuid,
    pub permission_level: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::shares)]
pub struct NewShare {
    pub id: Uuid,
    pub password_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub shared_with_user_id: Uuid,
    pub permission_level: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct ShareRequest {
    pub recipient_username: String,
    pub permission_level: String, // "view" or "edit"
    pub expiration_days: Option<i32>, // None for never expires
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: String, data: Option<T>) -> Self {
        ApiResponse {
            success: true,
            message,
            data,
        }
    }

    pub fn error(message: String) -> Self {
        ApiResponse {
            success: false,
            message,
            data: None,
        }
    }
}