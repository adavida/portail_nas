use axum::http::{Method, StatusCode};
use portail_backend::app;
use serde_json::Value;

use super::req_with_app::req_with_app;

pub async fn req(method: Method, uri: &str, body: Option<Value>) -> StatusCode {
    portail_backend::env::Env::ensure_init();
    portail_backend::auth::secrets::ensure(portail_backend::env::Env::global()).unwrap();
    req_with_app(app(), method, uri, body).await
}
