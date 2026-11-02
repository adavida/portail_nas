use axum::{Json, extract::State};
use serde::Serialize;

use portail_core::config::Config;

#[derive(Serialize)]
pub struct AuthConfig {
    issuer: String,
    client_id: String,
    redirect_uri: String,
}

pub async fn config(State(config): State<Config>) -> Json<AuthConfig> {
    Json(AuthConfig {
        issuer: config.oidc_issuer_url.clone(),
        client_id: config.oidc_client_id.clone(),
        redirect_uri: config.oidc_redirect_uri.clone(),
    })
}
