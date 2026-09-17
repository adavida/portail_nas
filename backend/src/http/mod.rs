pub mod error;
pub mod groups;
pub mod health;
pub mod users;

use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .nest("/api/users", users::users_router())
        .nest("/api/groups", groups::groups_router())
}
