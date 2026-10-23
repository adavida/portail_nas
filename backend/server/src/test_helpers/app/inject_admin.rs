use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

use portail_core::env::Env;
use portail_oidc::auth::{Claims, middleware::AuthUser};

pub(crate) async fn inject_admin(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    Env::ensure_init();
    let claims = Claims {
        sub: "admin".into(),
        aud: serde_json::json!(Env::global().oidc_client_id.clone()),
        iss: Env::global().oidc_issuer_url.clone(),
        exp: 9999999999,
        groups: vec!["admin".into()],
        email: Some("admin@example.com".into()),
        preferred_username: Some("admin".into()),
        nonce: None,
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
