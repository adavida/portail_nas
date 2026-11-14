use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::inject_user_as::inject_user_as;

pub async fn send_as(
    uid: &str,
    method: Method,
    uri: &str,
    json_body: Option<Value>,
) -> (StatusCode, Value) {
    let config = portail_core::config::Config::global();
    portail_oidc::auth::secrets::ensure(config).unwrap();
    let app = portail_server::app().layer(middleware::from_fn(inject_user_as));
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-test-uid", uid);
    let resp = if let Some(b) = &json_body {
        builder = builder.header("content-type", "application/json");
        app.oneshot(
            builder
                .body(Body::from(serde_json::to_vec(b).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
    } else {
        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    };
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();

    (status, serde_json::from_slice(&bytes).unwrap_or(json!({})))
}
