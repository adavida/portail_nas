use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::test_app::test_app;

pub async fn send_raw(
    method: Method,
    uri: &str,
    headers: &[(&'static str, String)],
    body: Option<String>,
) -> (StatusCode, HeaderMap, serde_json::Value, axum::body::Bytes) {
    let app = test_app();
    let mut builder = Request::builder().method(method).uri(uri);
    for (k, v) in headers {
        builder = builder.header(*k, v);
    }
    let body = body.map(Body::from).unwrap_or(Body::empty());
    let resp: Response<Body> = app.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let raw = resp.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&raw).unwrap_or(serde_json::json!({}));
    (status, headers, json, raw)
}
