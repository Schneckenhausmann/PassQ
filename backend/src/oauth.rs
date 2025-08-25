//! OAuth module for SSO authentication

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use crate::schema::oauth_accounts;

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(crate::models::User))]
#[diesel(table_name = oauth_accounts)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub access_token_hash: Option<String>,
    pub refresh_token_hash: Option<String>,
    pub token_expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = oauth_accounts)]
pub struct NewOAuthAccount {
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: String,
    pub access_token_hash: Option<String>,
    pub refresh_token_hash: Option<String>,
    pub token_expires_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthLoginRequest {
    pub provider: String,
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthLoginResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user: crate::models::User,
    pub is_new_user: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum OAuthProvider {
    Microsoft,
    Google,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Microsoft => "microsoft",
            OAuthProvider::Google => "google",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "microsoft" => Some(OAuthProvider::Microsoft),
            "google" => Some(OAuthProvider::Google),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> OAuthConfig {
        match self {
            OAuthProvider::Microsoft => OAuthConfig {
                client_id: std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
                client_secret: std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                user_info_url: "https://graph.microsoft.com/v1.0/me".to_string(),
                scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            },
            OAuthProvider::Google => OAuthConfig {
                client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
                client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                user_info_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
                scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            },
        }
    }
}

// Database operations
impl OAuthAccount {
    pub fn find_by_provider_and_email(
        conn: &mut PgConnection,
        provider: &str,
        email: &str,
    ) -> QueryResult<Option<OAuthAccount>> {
        oauth_accounts::table
            .filter(oauth_accounts::provider.eq(provider))
            .filter(oauth_accounts::email.eq(email))
            .first::<OAuthAccount>(conn)
            .optional()
    }

    pub fn find_by_user_id(
        conn: &mut PgConnection,
        user_id: Uuid,
    ) -> QueryResult<Vec<OAuthAccount>> {
        oauth_accounts::table
            .filter(oauth_accounts::user_id.eq(user_id))
            .load::<OAuthAccount>(conn)
    }

    pub fn create(
        conn: &mut PgConnection,
        new_account: &NewOAuthAccount,
    ) -> QueryResult<OAuthAccount> {
        diesel::insert_into(oauth_accounts::table)
            .values(new_account)
            .get_result(conn)
    }

    pub fn update_tokens(
        conn: &mut PgConnection,
        account_id: Uuid,
        access_token_hash: Option<String>,
        refresh_token_hash: Option<String>,
        expires_at: Option<chrono::NaiveDateTime>,
    ) -> QueryResult<OAuthAccount> {
        diesel::update(oauth_accounts::table.find(account_id))
            .set((
                oauth_accounts::access_token_hash.eq(access_token_hash),
                oauth_accounts::refresh_token_hash.eq(refresh_token_hash),
                oauth_accounts::token_expires_at.eq(expires_at),
                oauth_accounts::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result(conn)
    }
}