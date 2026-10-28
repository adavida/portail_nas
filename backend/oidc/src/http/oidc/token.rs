use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::Response,
};
use serde::Deserialize;

use crate::auth::tokens::{self, basic_credentials};
use portail_core::config::Config;
use portail_core::error::AppError;

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
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
}

pub async fn token(
    State(config): State<Config>,
    headers: HeaderMap,
    Form(f): Form<TokenForm>,
) -> Result<Response, AppError> {
    if f.grant_type.as_deref() != Some("authorization_code") {
        return Err(AppError::BadRequest("unsupported_grant_type".into()));
    }
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let (client_id, client_secret) = match basic_credentials(auth) {
        Some(credentials) => credentials,
        None => match f.client_id {
            Some(id) => (id, f.client_secret.unwrap_or_default()),
            None => return Err(AppError::InvalidClient("invalid_client".into())),
        },
    };
    let tokens = tokens::exchange(
        &config,
        &client_id,
        &client_secret,
        &f.code.unwrap_or_default(),
        &f.redirect_uri.unwrap_or_default(),
        f.code_verifier.as_deref(),
    )?;
    Ok(super::no_store_json(tokens))
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
        crate::auth::secrets::ensure(portail_core::config::Config::global()).unwrap();
        let config = portail_core::config::Config::global().clone();
        super::super::oidc_router(config.clone()).with_state(config)
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

        let status = resp.status();
        let www = resp
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));

        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "invalid_client must be 401"
        );
        assert_eq!(www, "Basic", "401 must advertise Basic auth");
        assert_eq!(
            v["error"], "invalid_client",
            "unknown client must return invalid_client"
        );
    }
}
