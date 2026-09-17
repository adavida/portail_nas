pub mod error;
pub mod health;
pub mod users;

use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route(
            "/api/users",
            get(users::list_users).post(users::create_user),
        )
        .route(
            "/api/users/:uid/password",
            axum::routing::put(users::update_user_password),
        )
}
