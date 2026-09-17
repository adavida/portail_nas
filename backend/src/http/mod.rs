pub mod error;
pub mod health;
pub mod users;

use axum::{Router, routing::get, routing::put};

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .nest("/api/users", users_router())
}

fn users_router() -> Router {
    let r = Router::new()
        .route("/", get(users::list_users).post(users::create_user))
        .route("/{uid}", put(users::update_user).delete(users::delete_user))
        .route("/{uid}/password", put(users::update_user_password));

    #[cfg(feature = "test-api")]
    let r = r.route(
        "/{uid}/authenticate",
        axum::routing::post(users::authenticate_user),
    );

    r
}
