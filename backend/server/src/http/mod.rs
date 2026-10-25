pub mod health;

use axum::{Router, middleware, routing::get};
use tower_http::trace::TraceLayer;

use portail_core::config::Config;
use portail_directory::http::{groups::groups_router, users::users_router};
use portail_oidc::{
    auth::{require_admin, require_auth},
    http::{auth, oidc},
};

pub fn router() -> Router {
    let config = Config::global().clone();
    let admin: Router<Config> = Router::<Config>::new()
        .nest("/api/users", users_router())
        .nest("/api/groups", groups_router())
        .nest("/api/auth", auth::protected_auth_router())
        .layer(middleware::from_fn(require_admin))
        .layer(middleware::from_fn_with_state(config.clone(), require_auth));

    Router::<Config>::new()
        .route("/api/health", get(health::health))
        .nest("/api/auth", auth::auth_router(config.clone()))
        .merge(oidc::oidc_router(config.clone()))
        .merge(admin)
        .layer(TraceLayer::new_for_http())
        .with_state(config)
}
