use axum::http::{Method, StatusCode};

#[allow(dead_code)]
mod common;
use common::http::{
    req::req, req_with_app::req_with_app, send::send, test_app_as_user::test_app_as_user,
};
use common::seed::test_user::TestUser;

const CLAIMS_URI: &str = "/api/auth/debug/claims?uid=ghostuser&client_id=ghost";

#[tokio::test]
async fn debug_returns_oidc_info_for_admin() {
    let (status, body) = send(Method::GET, "/api/auth/debug", None).await;

    assert_eq!(status, StatusCode::OK, "admin should get 200 on debug");
    assert_eq!(
        body["issuer"].as_str(),
        Some(
            portail_core::config::Config::global()
                .oidc_issuer_url
                .as_str()
        ),
        "issuer should match the config"
    );
    assert!(
        body["clients"].is_array(),
        "clients should be a list, got {body}"
    );

    let raw = body.to_string();
    assert!(
        !raw.contains("secret_file") && !raw.contains("secrets"),
        "debug payload must not leak secret paths: {raw}"
    );
}

#[tokio::test]
async fn debug_requires_token() {
    let status = req(Method::GET, "/api/auth/debug", None).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/auth/debug should be 401 without token"
    );
}

#[tokio::test]
async fn debug_forbidden_for_non_admin() {
    let user_app = test_app_as_user();

    let status = req_with_app(user_app, Method::GET, "/api/auth/debug", None).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "GET /api/auth/debug user should be 403, got {status}"
    );
}

#[tokio::test]
async fn debug_claims_requires_token() {
    let status = req(Method::GET, CLAIMS_URI, None).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/auth/debug/claims should be 401 without token"
    );
}

#[tokio::test]
async fn debug_claims_forbidden_for_non_admin() {
    let user_app = test_app_as_user();

    let status = req_with_app(user_app, Method::GET, CLAIMS_URI, None).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "GET /api/auth/debug/claims user should be 403, got {status}"
    );
}

#[tokio::test]
async fn debug_claims_unknown_client_is_bad_request() {
    let (status, body) = send(Method::GET, CLAIMS_URI, None).await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unknown client should be 400, got {status}"
    );
    assert!(
        body["error"].as_str().unwrap_or_default().contains("ghost"),
        "error should name the unknown client, got {body}"
    );
}

#[tokio::test]
async fn debug_claims_returns_pairwise_sub_for_seeded_user() {
    let user = TestUser::new("claimspyw");
    let (created, _) = user.create().await;
    if created == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    let uri = format!(
        "/api/auth/debug/claims?uid={}&client_id=portail-dev",
        user.uid
    );

    let (status, body) = send(Method::GET, &uri, None).await;
    let sub = body["sub"].as_str().unwrap_or_default().to_string();

    assert_eq!(status, StatusCode::OK, "claims preview should be 200");
    assert_eq!(
        body["preferred_username"].as_str(),
        Some(user.uid.as_str()),
        "preferred_username keeps the uid"
    );
    assert_eq!(
        body["aud"].as_str(),
        Some("portail-dev"),
        "aud must be the selected client"
    );
    assert_eq!(sub.len(), 36, "sub must be a pairwise uuid, got {sub}");
    assert_ne!(sub, user.uid, "sub must not leak the uid");
    assert_eq!(
        body["env"].as_str(),
        Some("test"),
        "unconditional custom claim must appear in the preview, got {body}"
    );
    assert!(
        body.get("power").is_none(),
        "group-scoped claim must be absent for a user outside the admin group: {body}"
    );
    assert_eq!(
        body["outsider"].as_str(),
        Some("yes"),
        "not_group claim must be present for a user outside the admin group, got {body}"
    );

    user.delete().await;
}
