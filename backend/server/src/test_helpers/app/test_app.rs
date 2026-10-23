use axum::middleware;

use super::bootstrap_clients::bootstrap_clients_file;
use super::inject_admin::inject_admin;

pub(crate) fn test_app() -> axum::Router {
    portail_core::env::Env::ensure_init();
    let env = portail_core::env::Env::global();
    bootstrap_clients_file(env);
    portail_oidc::auth::secrets::ensure(env).unwrap();
    crate::app().layer(middleware::from_fn(inject_admin))
}
