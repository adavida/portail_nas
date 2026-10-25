use axum::http::{Method, StatusCode};
use portail_server::app;
use serde_json::Value;

use super::req_with_app::req_with_app;

pub async fn req(method: Method, uri: &str, body: Option<Value>) -> StatusCode {
    crate::common::config::set_test_config::set_test_config();
    let config = portail_core::config::Config::global();
    portail_oidc::auth::secrets::ensure(config).unwrap();
    req_with_app(app(), method, uri, body).await
}
