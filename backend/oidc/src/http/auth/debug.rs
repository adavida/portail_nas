use axum::{Json, extract::State};
use serde::Serialize;

use portail_core::config::{ClientEntry, Config};

#[derive(Serialize)]
pub struct DebugClient {
    id: String,
    enabled: bool,
    has_secret: bool,
    redirect_uris: Vec<String>,
    scopes: Vec<String>,
    required_group: String,
    forbidden_group: String,
}

impl DebugClient {
    fn from_entry(entry: &ClientEntry) -> Self {
        Self {
            id: entry.id.clone(),
            enabled: entry.enabled,
            has_secret: entry.secret_file.is_some(),
            redirect_uris: entry.redirect_uris.clone(),
            scopes: entry.scopes.clone(),
            required_group: entry.required_group.clone(),
            forbidden_group: entry.forbidden_group.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct DebugInfo {
    issuer: String,
    client_id: String,
    redirect_uri: String,
    key_bits: usize,
    pending_ttl_secs: u64,
    code_ttl_secs: u64,
    session_ttl_secs: u64,
    access_ttl_secs: u64,
    clients: Vec<DebugClient>,
}

pub async fn debug(State(config): State<Config>) -> Json<DebugInfo> {
    Json(DebugInfo {
        issuer: config.oidc_issuer_url.clone(),
        client_id: config.oidc_client_id.clone(),
        redirect_uri: config.oidc_redirect_uri.clone(),
        key_bits: config.oidc_key_bits,
        pending_ttl_secs: config.oidc_pending_ttl.as_secs(),
        code_ttl_secs: config.oidc_code_ttl.as_secs(),
        session_ttl_secs: config.oidc_session_ttl.as_secs(),
        access_ttl_secs: config.oidc_access_ttl.as_secs(),
        clients: config
            .oidc_clients
            .iter()
            .map(DebugClient::from_entry)
            .collect(),
    })
}
