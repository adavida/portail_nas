use axum::middleware;

use super::bootstrap_clients::bootstrap_clients_file;
use super::inject_admin::inject_admin;

pub(crate) fn test_app() -> axum::Router {
    crate::env::Env::ensure_init();
    let env = crate::env::Env::global();
    bootstrap_clients_file(env);
    crate::auth::secrets::ensure(env).unwrap();
    crate::app().layer(middleware::from_fn(inject_admin))
}
