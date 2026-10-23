use super::inject_admin::inject_admin;

pub fn test_app() -> axum::Router {
    use axum::middleware;
    portail_core::env::Env::ensure_init();
    let env = portail_core::env::Env::global();
    super::bootstrap_clients::bootstrap_clients_file(env);
    portail_oidc::auth::secrets::ensure(env).unwrap();
    let prod = portail_server::app();
    prod.layer(middleware::from_fn(inject_admin))
}
