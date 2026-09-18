pub mod auth;
pub mod error;
pub mod groups;
pub mod health;
pub mod users;

use axum::{Router, middleware, routing::get};
use tower_http::trace::TraceLayer;

use crate::auth::require_auth;

pub fn router() -> Router {
    let admin = Router::new()
        .nest("/api/users", users::users_router())
        .nest("/api/groups", groups::groups_router())
        .nest("/api/auth", auth::protected_auth_router())
        .layer(middleware::from_fn(crate::auth::middleware::require_admin))
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .route("/api/health", get(health::health))
        .nest("/api/auth", auth::auth_router())
        .merge(admin)
        .layer(TraceLayer::new_for_http())
}
