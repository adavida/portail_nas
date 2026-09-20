pub mod auth;
pub mod error;
pub mod groups;
pub mod health;
pub mod users;

use axum::{Router, middleware, routing::get};
use tower_http::trace::TraceLayer;

use crate::{auth::require_auth, env::Env};

pub fn router() -> Router {
    crate::env::Env::ensure_init();
    let env = crate::env::Env::global().clone();
    let admin: Router<Env> = Router::<Env>::new()
        .nest("/api/users", users::users_router())
        .nest("/api/groups", groups::groups_router())
        .nest("/api/auth", auth::protected_auth_router())
        .layer(middleware::from_fn(crate::auth::middleware::require_admin))
        .layer(middleware::from_fn_with_state(env.clone(), require_auth));

    Router::<Env>::new()
        .route("/api/health", get(health::health))
        .nest("/api/auth", auth::auth_router(env.clone()))
        .merge(admin)
        .layer(TraceLayer::new_for_http())
        .with_state(env)
}
