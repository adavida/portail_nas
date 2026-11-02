use super::inject_admin::inject_admin;

pub fn test_app() -> axum::Router {
    use axum::middleware;
    crate::common::config::set_test_config::set_test_config();
    let config = portail_core::config::Config::global();
    portail_oidc::auth::secrets::ensure(config).unwrap();
    let prod = portail_server::app();
    prod.layer(middleware::from_fn(inject_admin))
}
