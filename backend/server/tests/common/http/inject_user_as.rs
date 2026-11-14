use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use portail_oidc::auth::{Claims, middleware::AuthUser};

pub async fn inject_user_as(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let uid = req
        .headers()
        .get("x-test-uid")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("user")
        .to_string();
    let claims = Claims {
        sub: uid.clone(),
        aud: portail_core::config::Config::global()
            .oidc
            .client_id
            .clone(),
        iss: portail_core::config::Config::global()
            .oidc
            .issuer_url
            .as_str()
            .to_string(),
        iat: 0,
        exp: 9999999999,
        groups: vec![],
        email: Some(format!("{uid}@example.com")),
        preferred_username: Some(uid),
        nonce: None,
        azp: None,
        custom_claims: Default::default(),
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
