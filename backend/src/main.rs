use portail_backend::{app, env::Env};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .init();
    // Validate env once at startup — aggregated error if any var missing/empty.
    let env = Env::create().unwrap_or_else(|e| panic!("{e}"));
    env.log_summary();
    let bind = env.bind_addr.clone();
    Env::set(env);
    let app = app();
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
