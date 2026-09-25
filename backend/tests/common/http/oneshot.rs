use axum::http::{Method, StatusCode};
use serde_json::Value;

use super::send::send;

pub async fn oneshot(uri: &str) -> (StatusCode, Value) {
    send(Method::GET, uri, None).await
}
