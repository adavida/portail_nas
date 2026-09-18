use axum::{Extension, Json, Router, http::StatusCode, routing::get, routing::post};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::{auth::middleware::AuthUser, env, error::AppError};

pub fn auth_router() -> Router {
    Router::new()
        .route("/config", get(config))
        .route("/callback", post(callback))
}

pub fn protected_auth_router() -> Router {
    Router::new().route("/me", get(me))
}

#[derive(Serialize)]
pub struct AuthConfig {
    issuer: String,
    client_id: String,
    redirect_uri: String,
}

pub async fn config() -> Json<AuthConfig> {
    Json(AuthConfig {
        issuer: env::OIDC_ISSUER_URL.to_string(),
        client_id: env::OIDC_CLIENT_ID.to_string(),
        redirect_uri: env::OIDC_REDIRECT_URI.to_string(),
    })
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub redirect_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<u64>,
}

pub async fn callback(
    Json(payload): Json<CallbackRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    tracing::info!(
        "callback code len={} redirect_uri={:?}",
        payload.code.len(),
        payload.redirect_uri
    );
    let issuer = env::OIDC_ISSUER_URL.to_string();
    let client_id = env::OIDC_CLIENT_ID.to_string();
    let client_secret = env::OIDC_CLIENT_SECRET.to_string();
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| env::OIDC_REDIRECT_URI.to_string());

    let token_url = format!("{issuer}/api/oidc/token");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let credentials = STANDARD.encode(format!("{client_id}:{client_secret}"));
    let params = [
        ("grant_type", "authorization_code"),
        ("code", payload.code.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let resp = client
        .post(&token_url)
        .header("Authorization", format!("Basic {credentials}"))
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        tracing::warn!("token exchange failed {}: {}", status, txt);
        return Err(AppError::Internal(format!("token exchange failed: {txt}")));
    }
    tracing::info!("token exchange ok for code len={}", payload.code.len());

    let token = resp
        .json::<TokenResponse>()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(token))
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "sub": claims.sub,
        "groups": claims.groups,
        "email": claims.email,
        "preferred_username": claims.preferred_username,
    }))
}

pub async fn _health_auth() -> StatusCode {
    StatusCode::OK
}
