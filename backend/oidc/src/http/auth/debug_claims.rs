use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::auth::{Claims, store, tokens};
use crate::domain::oidc::Client;
use crate::http::oidc::enrich_flow;
use portail_core::config::Config;
use portail_core::error::AppError;

#[derive(Deserialize)]
pub struct DebugClaimsQuery {
    pub uid: String,
    pub client_id: String,
}

pub async fn debug_claims(
    State(config): State<Config>,
    Query(q): Query<DebugClaimsQuery>,
) -> Result<Json<Claims>, AppError> {
    let clients = tokens::clients(&config)?;
    let Some(client) = Client::find_in(&clients, &q.client_id) else {
        return Err(AppError::BadRequest(format!(
            "unknown client: {}",
            q.client_id
        )));
    };
    let mut flow = store::AuthFlow {
        client_id: client.id.as_str().to_string(),
        redirect_uri: String::new(),
        state: None,
        nonce: None,
        scopes: client.scopes.clone(),
        challenge: None,
        sub: q.uid.clone(),
        groups: Vec::new(),
        email: None,
    };
    enrich_flow(&mut flow).await?;
    if let Some(entry) = config.oidc_clients.iter().find(|e| e.id == q.client_id) {
        if let Some(reason) = entry.gate_denial(&flow.groups) {
            return Err(AppError::Forbidden(format!(
                "access to client '{}' denied: {reason}",
                q.client_id
            )));
        }
    }
    Ok(Json(tokens::claims(&config, &flow)?))
}
