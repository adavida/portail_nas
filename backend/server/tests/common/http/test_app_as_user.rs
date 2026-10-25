use super::inject_user::inject_user;

pub fn test_app_as_user() -> axum::Router {
    use axum::middleware;
    crate::common::config::set_test_config::set_test_config();
    let config = portail_core::config::Config::global();
    portail_oidc::auth::secrets::ensure(config).unwrap();
    let prod = portail_server::app();
    prod.layer(middleware::from_fn(inject_user))
}
