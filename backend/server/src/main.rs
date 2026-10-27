use portail_core::config::Config;
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
    tracing::info!("start ... ... portail");
    let config = Config::load(&config_path_from_args()).unwrap_or_else(|e| panic!("{e}"));
    config.log_summary();
    tracing::info!("ensure start");
    portail_oidc::auth::secrets::ensure(&config).unwrap_or_else(|e| panic!("ensure panic : {e}"));
    tracing::info!("ensure end");
    let bind = config.bind_addr.clone();
    Config::set(config);
    tracing::info!("Config::set");
    let app: axum::Router = app();
    tracing::info!("app ...");
      
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
