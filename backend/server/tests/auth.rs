use axum::http::{Method, StatusCode};
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::http::{req::req, req_with_app::req_with_app, send::send};

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
    let status = req(Method::GET, "/api/health", None).await;
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/health should not be 401"
    );
    assert_eq!(status, StatusCode::OK);

    let status = req(Method::GET, "/api/auth/config", None).await;
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/auth/config should not be 401"
    );
    assert_eq!(status, StatusCode::OK);

    let status = req(
        Method::POST,
        "/api/auth/callback",
        Some(json!({
            "code": "fake",
            "redirect_uri": portail_core::config::Config::global().oidc_redirect_uri.as_str()
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

#[cfg(feature = "test-api")]
#[tokio::test]
async fn oidc_test_endpoint_is_protected() {
    let status = req(
        Method::POST,
        "/api/auth/oidc-test",
        Some(json!({ "client_id": "portail-dev" })),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "POST /api/auth/oidc-test should be 401 without token"
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

#[tokio::test]
async fn user_can_access_me_without_admin() {
    let user_app = common::http::test_app_as_user::test_app_as_user();

    let status = req_with_app(user_app, Method::GET, "/api/auth/me", None).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "GET /api/auth/me user should be 200, got {status}"
    );
}

#[tokio::test]
async fn me_reports_is_admin_and_groups_for_admin() {
    let (status, body) = send(Method::GET, "/api/auth/me", None).await;

    assert_eq!(status, StatusCode::OK, "admin /me should be 200");
    assert_eq!(
        body["is_admin"].as_bool(),
        Some(true),
        "admin /me must report is_admin true, got {body}"
    );
    assert_eq!(
        body["preferred_username"].as_str(),
        Some("admin"),
        "/me must expose the logged-in username, got {body}"
    );
}
