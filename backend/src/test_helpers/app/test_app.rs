use axum::middleware;

use super::inject_admin::inject_admin;

pub(crate) fn test_app() -> axum::Router {
    crate::app().layer(middleware::from_fn(inject_admin))
}
