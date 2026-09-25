use axum::http::{Method, StatusCode};
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::http::{req::req, req_with_app::req_with_app};

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
        Some(json!({
            "code": "fake",
            "redirect_uri": env!("OIDC_REDIRECT_URI")
        })),
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

#[tokio::test]
async fn admin_can_access_protected_endpoints() {
    let admin_app = common::http::test_app::test_app();
    let protected = vec![
        (Method::GET, "/api/users"),
        (Method::GET, "/api/groups"),
        (Method::GET, "/api/auth/me"),
    ];
    for (method, uri) in protected {
        let status = req_with_app(admin_app.clone(), method.clone(), uri, None).await;
        assert_ne!(
            status,
            StatusCode::UNAUTHORIZED,
            "{} {} admin should not be 401",
            method,
            uri
        );
        assert_ne!(
            status,
            StatusCode::FORBIDDEN,
            "{} {} admin should not be 403",
            method,
            uri
        );
    }
}

#[tokio::test]
async fn user_gets_403_on_admin_endpoints() {
    let user_app = common::http::test_app_as_user::test_app_as_user();
    let admin_endpoints = vec![
        (Method::GET, "/api/users", None),
        (Method::GET, "/api/groups", None),
        (Method::GET, "/api/auth/me", None),
        (
            Method::POST,
            "/api/users",
            Some(json!({ "uid": "x", "name": "x", "email": "", "password": "x" })),
        ),
        (
            Method::POST,
            "/api/groups",
            Some(json!({ "gid": "x", "name": "x", "description": "", "members": ["x"] })),
        ),
    ];
    for (method, uri, body) in admin_endpoints {
        let status = req_with_app(user_app.clone(), method.clone(), uri, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{} {} user should be 403, got {status}",
            method,
            uri
        );
    }
}
