use axum::middleware;

use super::inject_admin::inject_admin;

pub(crate) fn test_app() -> axum::Router {
    crate::env::Env::ensure_init();
    crate::auth::secrets::ensure(crate::env::Env::global()).unwrap();
    crate::app().layer(middleware::from_fn(inject_admin))
}
