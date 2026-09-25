use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use portail_backend::auth::{Claims, middleware::AuthUser};

pub async fn inject_user(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = Claims {
        sub: "user".into(),
        aud: serde_json::json!(env!("OIDC_CLIENT_ID")),
        iss: env!("OIDC_ISSUER_URL").to_string(),
        exp: 9999999999,
        groups: vec![],
        email: Some("user@example.com".into()),
        preferred_username: Some("user".into()),
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}
