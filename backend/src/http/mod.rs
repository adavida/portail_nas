pub mod error;
pub mod health;
pub mod users;

use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/users", get(users::list_users))
}
