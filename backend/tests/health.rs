use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

mod common;

async fn oneshot(uri: &str) -> (StatusCode, serde_json::Value) {
    let app = common::test_app();
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();

    (status, serde_json::from_slice(&body).unwrap_or(json!({})))
}

#[tokio::test]
async fn health_returns_ok() {
    let (status, body) = oneshot("/api/health").await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /api/health should return 200 OK"
    );
    assert_eq!(body["status"], "ok", "health status should be ok");
}

#[tokio::test]
async fn unknown_returns_404() {
    let (status, _body) = oneshot("/api/unknown").await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "unknown route should return 404"
    );
}
