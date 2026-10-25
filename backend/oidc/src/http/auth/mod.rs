use axum::{
    Extension, Json, Router,
    routing::{get, post},
};

use crate::auth::middleware::AuthUser;
use crate::http::oidc::userinfo::Userinfo;
use portail_core::config::Config;

pub mod callback;
pub mod config;
pub mod oidc_test;

pub fn auth_router(config: Config) -> Router<Config> {
    Router::new()
        .route("/config", get(config::config))
        .route("/callback", post(callback::callback))
        .with_state(config)
}

pub fn protected_auth_router() -> Router<Config> {
    let r = Router::new().route("/me", get(me));

    #[cfg(feature = "test-api")]
    let r = r.route("/oidc-test", post(oidc_test::oidc_test));

    r
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<Userinfo> {
    Json(Userinfo {
        sub: claims.sub,
        preferred_username: claims.preferred_username,
        email: claims.email,
        groups: claims.groups,
    })
}
