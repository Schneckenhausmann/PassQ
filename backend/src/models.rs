use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub mfa_secret: Option<String>,
    pub reset_token: Option<String>,
    pub reset_token_expires_at: Option<chrono::NaiveDateTime>,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    #[diesel(column_name = id)]
    pub id: Uuid,
    #[diesel(column_name = username)]
    pub username: String,
    #[diesel(column_name = password_hash)]
    pub password_hash: String,
    #[diesel(column_name = salt)]
    pub salt: String,
    #[diesel(column_name = mfa_secret)]
    pub mfa_secret: Option<String>,
    #[diesel(column_name = reset_token)]
    pub reset_token: Option<String>,
    #[diesel(column_name = reset_token_expires_at)]
    pub reset_token_expires_at: Option<chrono::NaiveDateTime>,
    #[diesel(column_name = email)]
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
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
    pub encrypted_website: Option<Vec<u8>>,
    pub encrypted_username: Option<Vec<u8>>,
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
    pub encrypted_website: Option<Vec<u8>>,
    pub encrypted_username: Option<Vec<u8>>,
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
    pub name: Option<String>,
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
        Self {
            success: false,
            message,
            data: None,
        }
    }
}

// TODO: Fix LoginHistory model type mappings
// #[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
// #[diesel(table_name = crate::schema::login_history)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct LoginHistory {
//     pub id: i32,
//     pub user_id: Uuid,
//     pub ip_address: String,
//     pub country: Option<String>,
//     pub region: Option<String>,
//     pub city: Option<String>,
//     pub latitude: Option<String>,
//     pub longitude: Option<String>,
//     pub timezone: Option<String>,
//     pub isp: Option<String>,
//     pub user_agent: Option<String>,
//     pub login_time: chrono::DateTime<chrono::Utc>,
//     pub is_suspicious: bool,
// }

// #[derive(Insertable, Serialize, Deserialize, Debug)]
// #[diesel(table_name = crate::schema::login_history)]
// pub struct NewLoginHistory {
//     pub user_id: Uuid,
//     pub ip_address: String,
//     pub country: Option<String>,
//     pub region: Option<String>,
//     pub city: Option<String>,
//     pub latitude: Option<String>,
//     pub longitude: Option<String>,
//     pub timezone: Option<String>,
//     pub isp: Option<String>,
//     pub user_agent: Option<String>,
//     pub login_time: chrono::DateTime<chrono::Utc>,
//     pub is_suspicious: bool,
// }