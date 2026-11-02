use axum::{
    Extension, Json, Router,
    routing::{get, post},
};

use crate::auth::Claims;
use crate::auth::middleware::AuthUser;
use portail_core::config::Config;

pub mod callback;
pub mod config;
pub mod debug;
pub mod debug_claims;
pub mod oidc_test;

#[derive(serde::Serialize)]
pub struct Me {
    sub: String,
    preferred_username: Option<String>,
    email: Option<String>,
    groups: Vec<String>,
    is_admin: bool,
}

impl Me {
    fn from_claims(claims: &Claims) -> Self {
        Self {
            sub: claims.sub.clone(),
            preferred_username: claims.preferred_username.clone(),
            email: claims.email.clone(),
            groups: claims.groups.clone(),
            is_admin: claims.is_admin(),
        }
    }
}

pub fn auth_router(config: Config) -> Router<Config> {
    Router::new()
        .route("/config", get(config::config))
        .route("/callback", post(callback::callback))
        .with_state(config)
}

pub fn user_auth_router() -> Router<Config> {
    Router::new().route("/me", get(me))
}

pub fn protected_auth_router() -> Router<Config> {
    let r = Router::new()
        .route("/debug", get(debug::debug))
        .route("/debug/claims", get(debug_claims::debug_claims));

    #[cfg(feature = "test-api")]
    let r = r.route("/oidc-test", post(oidc_test::oidc_test));

    r
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<Me> {
    Json(Me::from_claims(&claims))
}
