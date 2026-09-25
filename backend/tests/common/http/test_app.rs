use super::inject_admin::inject_admin;

pub fn test_app() -> axum::Router {
    use axum::middleware;
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_admin))
}
