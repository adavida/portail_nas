use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{
    auth::tokens::{self, TokenResponse, exchange},
    domain::oidc::Client,
    env::Env,
    error::AppError,
};

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub redirect_uri: Option<String>,
    // PKCE verifier generated at `login()` (frontend) — required for the
    // authorization_code flow.
    pub code_verifier: Option<String>,
}

/// Exchanges code -> tokens for the portal itself (client of its own
/// provider) — same logic as `/api/oidc/token`, credentials from the env.
pub async fn callback(
    State(env): State<Env>,
    Json(payload): Json<CallbackRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    tracing::info!(
        "callback code len={} redirect_uri={:?}",
        payload.code.len(),
        payload.redirect_uri
    );
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| env.oidc_redirect_uri.clone());
    let clients = tokens::clients(&env)?;
    let client = Client::find_in(&clients, &env.oidc_client_id).ok_or_else(|| {
        AppError::Internal(format!(
            "portal client '{}' not found in clients file",
            env.oidc_client_id
        ))
    })?;
    let secret = crate::auth::secrets::stored_secret(&env, client)
        .map_err(AppError::Internal)?
        .map(|s| s.as_str().to_string())
        .unwrap_or_default();
    let tokens = exchange(
        &env,
        client.id.as_str(),
        &secret,
        &payload.code,
        &redirect_uri,
        payload.code_verifier.as_deref(),
    )?;
    tracing::info!("token exchange ok for code len={}", payload.code.len());
    Ok(Json(tokens))
}
