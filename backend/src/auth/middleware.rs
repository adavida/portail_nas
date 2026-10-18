use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::{
    auth::{Claims, tokens},
    env::Env,
};

pub fn verify_token(env: &Env, token: &str) -> Result<Claims, String> {
    tokens::verify(env, token).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

pub async fn require_auth(
    State(env): State<Env>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if req.extensions().get::<AuthUser>().is_some() {
        tracing::info!("auth bypass via injected AuthUser for {}", req.uri());
        return Ok(next.run(req).await);
    }
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        tracing::warn!("401 missing Bearer for {} {}", req.method(), req.uri());
        return Err(StatusCode::UNAUTHORIZED);
    }
    match verify_token(&env, token) {
        Ok(claims) => {
            tracing::info!(
                "auth ok sub={} groups={:?} for {}",
                claims.sub,
                claims.groups,
                req.uri()
            );
            req.extensions_mut().insert(AuthUser(claims));
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!(
                "401 token invalid for {} {}: {}",
                req.method(),
                req.uri(),
                e
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn require_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    let Some(AuthUser(claims)) = req.extensions().get::<AuthUser>().cloned() else {
        tracing::warn!("403 no AuthUser for {}", req.uri());
        return Err(StatusCode::FORBIDDEN);
    };
    if claims.is_admin() {
        Ok(next.run(req).await)
    } else {
        tracing::warn!(
            "403 forbidden for sub={} groups={:?} on {}",
            claims.sub,
            claims.groups,
            req.uri()
        );
        Err(StatusCode::FORBIDDEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_rejects_garbage_token() {
        let env = crate::env::Env::global();

        let err = verify_token(env, "not.a.jwt").unwrap_err();

        assert!(
            err.contains("token invalid") || err.contains("signing key"),
            "garbage token must be rejected: {err}"
        );
    }
}
