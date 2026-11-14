use axum::{Json, extract::State};
use serde::Serialize;

use portail_core::config::Config;

#[derive(Serialize)]
pub struct AuthConfig {
    issuer: String,
    client_id: String,
    redirect_uri: String,
    password_min_length: usize,
}

pub async fn config(State(config): State<Config>) -> Json<AuthConfig> {
    Json(AuthConfig {
        issuer: config.oidc.issuer_url.clone(),
        client_id: config.oidc.client_id.clone(),
        redirect_uri: config.oidc.redirect_uri.clone(),
        password_min_length: config.security.password_min_length,
    })
}
