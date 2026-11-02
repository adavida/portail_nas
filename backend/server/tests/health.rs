use axum::http::StatusCode;

#[allow(dead_code)]
mod common;
use common::http::oneshot::oneshot;

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
