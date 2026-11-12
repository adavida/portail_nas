use portail_core::config::Config;
use portail_directory::{controllers::users::ensure_admin, domain::ldap::Password};
use portail_server::app;

fn config_path_from_args() -> String {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--config" {
            if let Some(path) = args.next() {
                return path;
            }
        }
    }
    panic!("usage: portail-backend --config <path>");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .init();
    let config = Config::load(&config_path_from_args()).unwrap_or_else(|e| panic!("{e}"));
    let bind = config.bind_addr.clone();
    config.log_summary();
    Config::set(config);
    let admin_password =
        Password::for_authentication(Config::global().directory.admin_password.clone())
            .unwrap_or_else(|e| panic!("cannot build admin bootstrap password: {e}"));
    ensure_admin(admin_password)
        .await
        .unwrap_or_else(|e| panic!("cannot ensure admin user: {e}"));
    portail_oidc::auth::secrets::ensure(Config::global())
        .unwrap_or_else(|e| panic!("ensure panic : {e}"));
    let app: axum::Router = app();

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
