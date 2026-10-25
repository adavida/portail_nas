use axum::middleware;

use super::inject_admin::inject_admin;

pub(crate) fn test_app() -> axum::Router {
    let config = portail_core::config::Config::global();
    portail_oidc::auth::secrets::ensure(config).unwrap();
    crate::app().layer(middleware::from_fn(inject_admin))
}
