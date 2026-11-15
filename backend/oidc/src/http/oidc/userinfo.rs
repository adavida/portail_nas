use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use std::collections::BTreeMap;

use super::{attrs_claims, attrs_snapshot};
use crate::auth::tokens;
use portail_core::config::Config;
use portail_core::error::AppError;

#[derive(Serialize)]
pub struct Userinfo {
    pub(crate) sub: String,
    pub(crate) preferred_username: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) groups: Vec<String>,
    #[serde(flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) attrs: BTreeMap<String, String>,
}

pub async fn userinfo(
    State(config): State<Config>,
    headers: HeaderMap,
) -> Result<Json<Userinfo>, AppError> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(AppError::Unauthorized("missing Bearer token".into()));
    }
    let claims = tokens::verify_any_aud(&config, token)?;
    let expose = config
        .oidc
        .clients
        .iter()
        .find(|client| client.id == claims.aud)
        .map(|client| client.expose_attrs.clone())
        .unwrap_or_default();
    let attrs = match claims.preferred_username.as_deref() {
        Some(uid) if !expose.is_empty() => attrs_claims(&expose, &attrs_snapshot(uid).await)
            .into_iter()
            .collect(),
        _ => BTreeMap::new(),
    };
    Ok(Json(Userinfo {
        sub: claims.sub,
        preferred_username: claims.preferred_username,
        name: claims.name,
        email: claims.email,
        groups: claims.groups,
        attrs,
    }))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn userinfo_without_token_is_401() {
        let config = portail_core::config::Config::global().clone();
        let app = super::super::oidc_router(config.clone()).with_state(config);
        let req = Request::builder()
            .uri("/api/oidc/userinfo")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "userinfo without Bearer must be 401"
        );
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert!(
            String::from_utf8_lossy(&body).contains("missing"),
            "401 body should explain the missing token"
        );
    }
}
