use super::inject_user::inject_user;

pub fn test_app_as_user() -> axum::Router {
    use axum::middleware;
    portail_backend::env::Env::ensure_init();
    portail_backend::auth::secrets::ensure(portail_backend::env::Env::global()).unwrap();
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_user))
}
