use axum::{Json, extract::State};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::{env::Env, error::AppError};

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
    State(env): State<Env>,
    Json(payload): Json<CallbackRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    tracing::info!(
        "callback code len={} redirect_uri={:?}",
        payload.code.len(),
        payload.redirect_uri
    );
    let issuer = env.oidc_issuer_url.clone();
    let client_id = env.oidc_client_id.clone();
    let client_secret = env.oidc_client_secret.clone();
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| env.oidc_redirect_uri.clone());

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
