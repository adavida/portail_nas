use super::inject_admin::inject_admin;

pub fn test_app() -> axum::Router {
    use axum::middleware;
    portail_backend::env::Env::ensure_init();
    let env = portail_backend::env::Env::global();
    super::bootstrap_clients::bootstrap_clients_file(env);
    portail_backend::auth::secrets::ensure(env).unwrap();
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_admin))
}
