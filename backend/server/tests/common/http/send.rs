use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::test_app::test_app;

pub async fn send(method: Method, uri: &str, json_body: Option<Value>) -> (StatusCode, Value) {
    let app = test_app();
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(b) = &json_body {
        req = req.header("content-type", "application/json");
        let body = serde_json::to_vec(b).unwrap();
        let resp = app
            .oneshot(req.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        return (status, serde_json::from_slice(&bytes).unwrap_or(json!({})));
    }

    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();

    (status, serde_json::from_slice(&bytes).unwrap_or(json!({})))
}
