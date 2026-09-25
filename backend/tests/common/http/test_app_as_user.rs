use super::inject_user::inject_user;

pub fn test_app_as_user() -> axum::Router {
    use axum::middleware;
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_user))
}
