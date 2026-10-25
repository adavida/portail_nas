use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use portail_oidc::auth::{Claims, middleware::AuthUser};

pub async fn inject_user(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = Claims {
        sub: "user".into(),
        aud: serde_json::json!(
            portail_core::config::Config::global()
                .oidc_client_id
                .as_str()
        ),
        iss: portail_core::config::Config::global()
            .oidc_issuer_url
            .as_str()
            .to_string(),
        exp: 9999999999,
        groups: vec![],
        email: Some("user@example.com".into()),
        preferred_username: Some("user".into()),
        nonce: None,
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
