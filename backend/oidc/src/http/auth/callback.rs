use axum::{Json, extract::State, response::Response};
use serde::Deserialize;

use crate::{
    auth::tokens::{self, exchange},
    domain::oidc::Client,
    http::oidc,
};
use portail_core::config::Config;
use portail_core::error::AppError;

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
}

pub async fn callback(
    State(config): State<Config>,
    Json(payload): Json<CallbackRequest>,
) -> Result<Response, AppError> {
    tracing::info!(
        "callback code len={} redirect_uri={:?}",
        payload.code.len(),
        payload.redirect_uri
    );
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| config.oidc_redirect_uri.clone());
    let clients = tokens::clients(&config)?;
    let client = Client::find_in(&clients, &config.oidc_client_id).ok_or_else(|| {
        AppError::Internal(format!(
            "portal client '{}' not found in clients file",
            config.oidc_client_id
        ))
    })?;
    let secret = crate::auth::secrets::stored_secret(&config, client)
        .map_err(AppError::Internal)?
        .map(|s| s.as_str().to_string())
        .unwrap_or_default();
    let tokens = exchange(
        &config,
        client.id.as_str(),
        &secret,
        &payload.code,
        &redirect_uri,
        payload.code_verifier.as_deref(),
    )?;
    tracing::info!("token exchange ok for code len={}", payload.code.len());
    Ok(oidc::no_store_json(tokens))
}
