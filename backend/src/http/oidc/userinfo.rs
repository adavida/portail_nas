use axum::{Json, extract::State, http::HeaderMap};
use serde_json::json;

use crate::{auth::tokens, env::Env, error::AppError};

/// Standard UserInfo — claims of the access token verified locally.
/// Any audience accepted (signature + issuer only): each client's own
/// token works here, audience checks stay on the resource servers.
pub async fn userinfo(
    State(env): State<Env>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(AppError::Unauthorized("missing Bearer token".into()));
    }
    let claims = tokens::verify_any_aud(&env, token)?;
    Ok(Json(json!({
        "sub": claims.sub,
        "preferred_username": claims.preferred_username,
        "email": claims.email,
        "groups": claims.groups,
    })))
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
        crate::env::Env::ensure_init();
        let env = crate::env::Env::global().clone();
        let app = super::super::oidc_router(env.clone()).with_state(env);
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
