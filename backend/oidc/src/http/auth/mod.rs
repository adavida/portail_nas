use axum::{
    Router,
    routing::{get, post, put},
};

use portail_core::config::Config;

pub mod callback;
pub mod config;
mod current_uid;
pub mod debug;
pub mod debug_claims;
mod me;
pub mod oidc_test;
mod update_me;
mod update_me_password;

use me::me;
use update_me::update_me;
use update_me_password::update_me_password;

pub fn auth_router(config: Config) -> Router<Config> {
    Router::new()
        .route("/config", get(config::config))
        .route("/callback", post(callback::callback))
        .with_state(config)
}

pub fn user_auth_router() -> Router<Config> {
    Router::new()
        .route("/me", get(me).put(update_me))
        .route("/me/password", put(update_me_password))
}

pub fn protected_auth_router() -> Router<Config> {
    let r = Router::new()
        .route("/debug", get(debug::debug))
        .route("/debug/claims", get(debug_claims::debug_claims));

    #[cfg(feature = "test-api")]
    let r = r.route("/oidc-test", post(oidc_test::oidc_test));

    r
}
