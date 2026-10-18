use super::inject_admin::inject_admin;

pub fn test_app() -> axum::Router {
    use axum::middleware;
    portail_backend::env::Env::ensure_init();
    portail_backend::auth::secrets::ensure(portail_backend::env::Env::global()).unwrap();
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_admin))
}
