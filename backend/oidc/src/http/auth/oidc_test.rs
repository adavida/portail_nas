use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{controllers::oidc, domain::oidc::ClientId};
use portail_core::env::Env;
use portail_core::error::AppError;

#[derive(Deserialize)]
pub struct OidcTestRequest {
    #[serde(default)]
    pub client_id: Option<String>,
}

pub async fn oidc_test(
    State(env): State<Env>,
    Json(payload): Json<OidcTestRequest>,
) -> Result<Json<oidc::OidcTestReport>, AppError> {
    let raw = payload
        .client_id
        .unwrap_or_else(|| env.oidc_client_id.clone());
    let client_id = ClientId::try_new(raw).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(oidc::test_client(client_id)?))
}
