use axum::{
    Extension, Json, Router,
    routing::{get, post},
};

use crate::auth::middleware::AuthUser;
use portail_core::env::Env;

pub mod callback;
pub mod config;
pub mod oidc_test;

pub fn auth_router(env: Env) -> Router<Env> {
    Router::new()
        .route("/config", get(config::config))
        .route("/callback", post(callback::callback))
        .with_state(env)
}

pub fn protected_auth_router() -> Router<Env> {
    let r = Router::new().route("/me", get(me));

    #[cfg(feature = "test-api")]
    let r = r.route("/oidc-test", post(oidc_test::oidc_test));

    r
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "sub": claims.sub,
        "groups": claims.groups,
        "email": claims.email,
        "preferred_username": claims.preferred_username,
    }))
}
