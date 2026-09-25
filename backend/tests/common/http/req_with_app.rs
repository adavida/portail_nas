use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

pub async fn req_with_app(
    app: axum::Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> StatusCode {
    let mut builder = Request::builder().method(method).uri(uri);
    let resp = if let Some(b) = body {
        builder = builder.header("content-type", "application/json");
        let bytes = serde_json::to_vec(&b).unwrap();
        app.oneshot(builder.body(Body::from(bytes)).unwrap())
            .await
            .unwrap()
    } else {
        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    };
    let status = resp.status();
    let _ = resp.into_body().collect().await.unwrap().to_bytes();
    status
}
