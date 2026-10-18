use axum::{
    Json,
    extract::{Form, State},
    http::HeaderMap,
};
use serde::Deserialize;

use crate::{
    auth::tokens::{self, TokenResponse, basic_credentials},
    env::Env,
    error::AppError,
};

#[derive(Deserialize)]
pub struct TokenForm {
    #[serde(default)]
    pub grant_type: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub code_verifier: Option<String>,
}

/// Standard token endpoint (`client_secret_basic` + `authorization_code` +
/// PKCE). The portal itself goes through `/api/auth/callback` (same logic,
/// credentials from the env).
pub async fn token(
    State(env): State<Env>,
    headers: HeaderMap,
    Form(f): Form<TokenForm>,
) -> Result<Json<TokenResponse>, AppError> {
    if f.grant_type.as_deref() != Some("authorization_code") {
        return Err(AppError::BadRequest("unsupported_grant_type".into()));
    }
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let (client_id, client_secret) =
        basic_credentials(auth).ok_or_else(|| AppError::BadRequest("invalid_client".into()))?;
    let tokens = tokens::exchange(
        &env,
        &client_id,
        &client_secret,
        &f.code.unwrap_or_default(),
        &f.redirect_uri.unwrap_or_default(),
        f.code_verifier.as_deref(),
    )?;
    Ok(Json(tokens))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{HeaderValue, Method, Request, StatusCode},
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> axum::Router {
        crate::env::Env::ensure_init();
        let env = crate::env::Env::global().clone();
        super::super::oidc_router(env.clone()).with_state(env)
    }

    fn basic(client_id: &str, secret: &str) -> String {
        format!("Basic {}", STANDARD.encode(format!("{client_id}:{secret}")))
    }

    #[tokio::test]
    async fn token_rejects_unknown_grant_type() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/oidc/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("authorization", HeaderValue::from_static("bogus"))
            .body(Body::from("grant_type=password"))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "unknown grant_type must be 400"
        );
    }

    #[tokio::test]
    async fn token_rejects_unknown_client() {
        let app = app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/oidc/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .header(
                "authorization",
                HeaderValue::from_str(&basic("ghost", "x")).unwrap(),
            )
            .body(Body::from(
                "grant_type=authorization_code&code=c&redirect_uri=http%3A%2F%2Flocalhost%3A5173%2Fcallback",
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));

        assert_eq!(
            v["error"], "invalid_client",
            "unknown client must return invalid_client"
        );
    }
}
