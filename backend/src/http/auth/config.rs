use axum::{Json, extract::State};
use serde::Serialize;

use crate::env::Env;

#[derive(Serialize)]
pub struct AuthConfig {
    issuer: String,
    client_id: String,
    redirect_uri: String,
}

pub async fn config(State(env): State<Env>) -> Json<AuthConfig> {
    Json(AuthConfig {
        issuer: env.oidc_issuer_url.clone(),
        client_id: env.oidc_client_id.clone(),
        redirect_uri: env.oidc_redirect_uri.clone(),
    })
}
