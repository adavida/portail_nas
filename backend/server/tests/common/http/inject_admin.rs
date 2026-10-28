use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use portail_oidc::auth::{Claims, middleware::AuthUser};

pub async fn inject_admin(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = Claims {
        sub: "admin".into(),
        aud: portail_core::config::Config::global()
            .oidc_client_id
            .clone(),
        iss: portail_core::config::Config::global()
            .oidc_issuer_url
            .as_str()
            .to_string(),
        iat: 0,
        exp: 9999999999,
        groups: vec!["admin".into()],
        email: Some("admin@example.com".into()),
        preferred_username: Some("admin".into()),
        nonce: None,
        azp: None,
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
