//! SSO Authentication module for OAuth providers

use actix_web::{web, HttpResponse, Error, HttpRequest};
use serde::{Deserialize, Serialize};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    AuthUrl, TokenUrl, basic::BasicClient, reqwest::async_http_client,
    TokenResponse, AccessToken,
};
use reqwest::Client;
use uuid::Uuid;
use diesel::prelude::*;
use crate::{
    db::DbPool,
    oauth::{OAuthProvider, OAuthUserInfo, OAuthLoginResponse, OAuthAccount, NewOAuthAccount},
    models::{User, NewUser},
    auth,
    schema::{users, oauth_accounts},
};
use std::env;
use ring::digest;

// Helper function to hash tokens
fn hash_token(token: &str) -> String {
    let digest = digest::digest(&digest::SHA256, token.as_bytes());
    hex::encode(digest.as_ref())
}

/// Configure OAuth client for the given provider
fn get_oauth_client(provider: &OAuthProvider) -> Result<BasicClient, Box<dyn std::error::Error + Send + Sync>> {
    let (client_id, client_secret, auth_url, token_url, redirect_uri) = match provider {
        OAuthProvider::Microsoft => {
            let client_id = env::var("MICROSOFT_CLIENT_ID")
                .map_err(|_| "MICROSOFT_CLIENT_ID not set")?;
            let client_secret = env::var("MICROSOFT_CLIENT_SECRET")
                .map_err(|_| "MICROSOFT_CLIENT_SECRET not set")?;
            let redirect_uri = env::var("MICROSOFT_REDIRECT_URI")
                .map_err(|_| "MICROSOFT_REDIRECT_URI not set")?;
            
            (client_id, client_secret, 
             "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
             "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
             redirect_uri)
        },
        OAuthProvider::Google => {
            let client_id = env::var("GOOGLE_CLIENT_ID")
                .map_err(|_| "GOOGLE_CLIENT_ID not set")?;
            let client_secret = env::var("GOOGLE_CLIENT_SECRET")
                .map_err(|_| "GOOGLE_CLIENT_SECRET not set")?;
            let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
                .map_err(|_| "GOOGLE_REDIRECT_URI not set")?;
            
            (client_id, client_secret,
             "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
             "https://www.googleapis.com/oauth2/v4/token".to_string(),
             redirect_uri)
        },
    };

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url)?,
        Some(TokenUrl::new(token_url)?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

    Ok(client)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthAuthUrlResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
}

/// Generate OAuth authorization URL
pub async fn get_oauth_auth_url(
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let provider_str = path.into_inner();
    
    let provider = match provider_str.as_str() {
        "microsoft" => OAuthProvider::Microsoft,
        "google" => OAuthProvider::Google,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid OAuth provider"
            })));
        }
    };

    let client = match get_oauth_client(&provider) {
        Ok(client) => client,
        Err(e) => {
            log::error!("OAuth client configuration error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth provider not configured"
            })));
        }
    };

    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    
    // Add provider-specific scopes
    let scopes = match provider {
        OAuthProvider::Microsoft => vec!["openid", "profile", "email"],
        OAuthProvider::Google => vec!["openid", "profile", "email"],
    };
    
    for scope in scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }

    let (auth_url, csrf_token) = auth_request.url();

    Ok(HttpResponse::Ok().json(OAuthAuthUrlResponse {
        auth_url: auth_url.to_string(),
        state: csrf_token.secret().clone(),
    }))
}

/// Handle OAuth callback
pub async fn handle_oauth_callback(
    path: web::Path<String>,
    callback_data: web::Json<OAuthCallbackRequest>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let provider_str = path.into_inner();
    
    let provider = match provider_str.as_str() {
        "microsoft" => OAuthProvider::Microsoft,
        "google" => OAuthProvider::Google,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid OAuth provider"
            })));
        }
    };

    let client = match get_oauth_client(&provider) {
        Ok(client) => client,
        Err(e) => {
            log::error!("OAuth client configuration error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth provider not configured"
            })));
        }
    };

    let token_result = client
        .exchange_code(AuthorizationCode::new(callback_data.code.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            log::error!("Token exchange failed: {}", e);
            actix_web::error::ErrorUnauthorized("OAuth token exchange failed")
        })?;

    let access_token = token_result.access_token();
    
    // Get user information from provider
    let user_info_url = match provider {
        OAuthProvider::Microsoft => "https://graph.microsoft.com/v1.0/me",
        OAuthProvider::Google => "https://www.googleapis.com/oauth2/v2/userinfo",
    };

    let user_info = get_user_info(&provider, access_token, user_info_url).await
        .map_err(|e| {
            log::error!("Failed to get user info: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to retrieve user information")
        })?;

    // Process user authentication/registration
    let mut conn = db_pool.get().map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let (user, is_new_user) = process_oauth_user(&mut conn, &provider, &user_info, access_token, token_result.refresh_token())
        .map_err(|e| {
            log::error!("OAuth user processing failed: {}", e);
            actix_web::error::ErrorInternalServerError("User authentication failed")
        })?;

    // Generate JWT tokens
    let token_pair = auth::generate_token_pair(user.id)
        .map_err(|e| {
            log::error!("Token generation failed: {}", e);
            actix_web::error::ErrorInternalServerError("Token generation failed")
        })?;

    Ok(HttpResponse::Ok().json(OAuthLoginResponse {
        access_token: token_pair.access_token,
        refresh_token: Some(token_pair.refresh_token),
        user,
        is_new_user,
    }))
}

/// Get user information from OAuth provider
async fn get_user_info(
    provider: &OAuthProvider,
    access_token: &AccessToken,
    user_info_url: &str,
) -> Result<OAuthUserInfo, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    
    let response = client
        .get(user_info_url)
        .bearer_auth(access_token.secret())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let user_info = match provider {
        OAuthProvider::Microsoft => {
            OAuthUserInfo {
                id: response["id"].as_str().unwrap_or_default().to_string(),
                email: response["mail"].as_str()
                    .or_else(|| response["userPrincipalName"].as_str())
                    .unwrap_or_default().to_string(),
                name: response["displayName"].as_str().map(|s| s.to_string()),
                picture: None, // Microsoft Graph requires separate call for photo
                verified_email: Some(true), // Microsoft emails are verified
            }
        },
        OAuthProvider::Google => {
            OAuthUserInfo {
                id: response["id"].as_str().unwrap_or_default().to_string(),
                email: response["email"].as_str().unwrap_or_default().to_string(),
                name: response["name"].as_str().map(|s| s.to_string()),
                picture: response["picture"].as_str().map(|s| s.to_string()),
                verified_email: response["verified_email"].as_bool(),
            }
        },
    };

    Ok(user_info)
}

/// Process OAuth user authentication/registration
fn process_oauth_user(
    conn: &mut PgConnection,
    provider: &OAuthProvider,
    user_info: &OAuthUserInfo,
    access_token: &AccessToken,
    refresh_token: Option<&oauth2::RefreshToken>,
) -> Result<(User, bool), Box<dyn std::error::Error + Send + Sync>> {
    // Check if OAuth account already exists
    if let Some(oauth_account) = OAuthAccount::find_by_provider_and_email(
        conn,
        provider.as_str(),
        &user_info.email,
    )? {
        // Existing OAuth account - update tokens and return user
        let access_token_hash = Some(hash_token(access_token.secret()));
        let refresh_token_hash = refresh_token.map(|rt| hash_token(rt.secret()));
        let expires_at = None; // TODO: Calculate expiration from token response
        
        OAuthAccount::update_tokens(
            conn,
            oauth_account.id,
            access_token_hash,
            refresh_token_hash,
            expires_at,
        )?;

        let user = users::table
            .find(oauth_account.user_id)
            .first::<User>(conn)?;

        return Ok((user, false));
    }

    // Check if user exists with this email (for account linking)
    let existing_user = users::table
        .filter(users::email.eq(&user_info.email))
        .first::<User>(conn)
        .optional()?;

    let is_new_user = existing_user.is_none();

    let user = if let Some(mut user) = existing_user {
        // Link OAuth account to existing user
        user.auth_method = Some("hybrid".to_string());
        user.is_sso_user = Some(true);
        user.sso_display_name = user_info.name.clone();
        user.sso_avatar_url = user_info.picture.clone();
        
        diesel::update(users::table.find(user.id))
            .set((
                users::auth_method.eq(&user.auth_method),
                users::is_sso_user.eq(&user.is_sso_user),
                users::sso_display_name.eq(&user.sso_display_name),
                users::sso_avatar_url.eq(&user.sso_avatar_url),
            ))
            .get_result::<User>(conn)?
    } else {
        // Create new user
        let new_user = NewUser {
            id: Uuid::new_v4(),
            username: user_info.email.split('@').next().unwrap_or(&user_info.email).to_string(),
            password_hash: "sso_placeholder".to_string(), // Placeholder hash for SSO users
            salt: "sso_salt".to_string(),
            mfa_secret: None,
            reset_token: None,
            reset_token_expires_at: None,
            email: user_info.email.clone(),
            auth_method: Some(provider.as_str().to_string()),
            is_sso_user: Some(true),
            sso_display_name: user_info.name.clone(),
            sso_avatar_url: user_info.picture.clone(),
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)?
    };

    // Create OAuth account record
    let access_token_hash = Some(hash_token(access_token.secret()));
    let refresh_token_hash = refresh_token.map(|rt| hash_token(rt.secret()));
    
    let new_oauth_account = NewOAuthAccount {
        user_id: user.id,
        provider: provider.as_str().to_string(),
        provider_user_id: user_info.id.clone(),
        email: user_info.email.clone(),
        access_token_hash,
        refresh_token_hash,
        token_expires_at: None, // TODO: Calculate expiration
    };

    OAuthAccount::create(conn, &new_oauth_account)?;

    Ok((user, is_new_user))
}

/// Get user's linked OAuth accounts
pub async fn get_user_oauth_accounts(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let user_id = match auth::extract_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication required"
            })));
        }
    };

    let mut conn = db_pool.get().map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    let oauth_accounts = OAuthAccount::find_by_user_id(&mut conn, user_id)
        .map_err(|e| {
            log::error!("Failed to fetch OAuth accounts: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch OAuth accounts")
        })?;

    Ok(HttpResponse::Ok().json(oauth_accounts))
}

/// Unlink OAuth account
pub async fn unlink_oauth_account(
    req: HttpRequest,
    path: web::Path<Uuid>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let user_id = match auth::extract_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication required"
            })));
        }
    };

    let account_id = path.into_inner();
    
    let mut conn = db_pool.get().map_err(|e| {
        log::error!("Database connection error: {}", e);
        actix_web::error::ErrorInternalServerError("Database connection failed")
    })?;

    // Verify the account belongs to the user
    let _oauth_account = oauth_accounts::table
        .filter(oauth_accounts::id.eq(account_id))
        .filter(oauth_accounts::user_id.eq(user_id))
        .first::<OAuthAccount>(&mut conn)
        .map_err(|_| {
            actix_web::error::ErrorNotFound("OAuth account not found")
        })?;

    // Delete the OAuth account
    diesel::delete(oauth_accounts::table.find(account_id))
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("Failed to delete OAuth account: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to unlink account")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OAuth account unlinked successfully"
    })))
}