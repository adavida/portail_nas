use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use portail_backend::app;
use serde_json::json;
use tower::ServiceExt;

mod common;

async fn req(method: Method, uri: &str, body: Option<serde_json::Value>) -> StatusCode {
    let app = app();
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
    // drain body
    let _ = resp.into_body().collect().await.unwrap().to_bytes();
    status
}

#[tokio::test]
async fn protected_endpoints_return_401_without_token() {
    let protected = vec![
        (Method::GET, "/api/users", None),
        (
            Method::POST,
            "/api/users",
            Some(json!({ "uid": "x", "name": "x", "email": "", "password": "x" })),
        ),
        (
            Method::PUT,
            "/api/users/someuid",
            Some(json!({ "name": "x", "email": "" })),
        ),
        (Method::DELETE, "/api/users/someuid", None),
        (
            Method::PUT,
            "/api/users/someuid/password",
            Some(json!({ "password": "x" })),
        ),
        (Method::GET, "/api/groups", None),
        (
            Method::POST,
            "/api/groups",
            Some(json!({ "gid": "x", "name": "x", "description": "", "members": ["x"] })),
        ),
        (
            Method::PUT,
            "/api/groups/somegid",
            Some(json!({ "name": "x", "description": "" })),
        ),
        (Method::DELETE, "/api/groups/somegid", None),
        (
            Method::POST,
            "/api/groups/somegid/members",
            Some(json!({ "uid": "x" })),
        ),
        (Method::DELETE, "/api/groups/somegid/members/someuid", None),
        (Method::GET, "/api/auth/me", None),
    ];

    for (method, uri, body) in protected {
        let status = req(method.clone(), uri, body).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{} {} should be 401 without token, got {status}",
            method,
            uri
        );
    }
}

#[tokio::test]
async fn public_endpoints_not_401_without_token() {
    // health
    let status = req(Method::GET, "/api/health", None).await;
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/health should not be 401"
    );
    assert_eq!(status, StatusCode::OK);

    // auth config
    let status = req(Method::GET, "/api/auth/config", None).await;
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/auth/config should not be 401"
    );
    assert_eq!(status, StatusCode::OK);

    // auth callback with fake code should be 500 (token exchange fails) not 401
    let status = req(
        Method::POST,
        "/api/auth/callback",
        Some(json!({ "code": "fake", "redirect_uri": "http://localhost:5173/callback" })),
    )
    .await;
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /api/auth/callback should not be 401"
    );
}

#[cfg(feature = "test-api")]
#[tokio::test]
async fn authenticate_endpoint_is_protected() {
    let status = req(
        Method::POST,
        "/api/users/someuid/authenticate",
        Some(json!({ "password": "x" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /api/users/:uid/authenticate should be 401 without token"
    );
}
