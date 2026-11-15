use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::common::{
    directory::assert_available::assert_available,
    http::{req::req, send::send, send_as::send_as},
    seed::test_user::TestUser,
};

#[tokio::test]
async fn update_me_changes_profile_and_me_reflects_it() {
    assert_available().await;
    let user = TestUser::new("apimeprof");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me",
        Some(json!({"name": "Renamed Me", "email": "renamed@example.com"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "PUT /api/auth/me should 200");
    assert_eq!(
        body["profile_saved"], true,
        "a valid profile half should report saved"
    );
    assert_eq!(
        body["attrs_saved"], true,
        "untouched attrs should report saved"
    );

    let (status, body) = send_as(&user.uid, Method::GET, "/api/auth/me", None).await;

    assert_eq!(status, StatusCode::OK, "GET /api/auth/me should be 200");
    assert_eq!(
        body["name"], "Renamed Me",
        "me should report the updated name"
    );
    assert_eq!(
        body["email"], "renamed@example.com",
        "me should report the updated email"
    );

    user.delete().await;
}

#[tokio::test]
async fn update_me_rejects_invalid_email() {
    assert_available().await;
    let user = TestUser::new("apimebad");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me",
        Some(json!({"name": "Still Me", "email": "not-an-email"})),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "the merge request itself should succeed"
    );
    assert_eq!(
        body["profile_saved"], false,
        "an invalid email should be rejected by the Email VO"
    );
    assert!(
        body["error"].is_string(),
        "the failure reason should be reported, got {body}"
    );

    user.delete().await;
}

#[tokio::test]
async fn update_me_saves_profile_and_attrs_in_one_request() {
    assert_available().await;
    let user = TestUser::new("apimeall");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me",
        Some(json!({
            "name": "Full Save",
            "email": "full@example.com",
            "attrs": [{"label": "city", "value": "Nice"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "PUT /api/auth/me should 200");
    assert_eq!(
        body["profile_saved"], true,
        "valid profile half should save, got {body}"
    );
    assert_eq!(
        body["attrs_saved"], true,
        "valid attrs half should save, got {body}"
    );

    let (_, me) = send_as(&user.uid, Method::GET, "/api/auth/me", None).await;
    let city = me["attrs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == "city")
        .cloned();

    assert_eq!(
        city.unwrap()["value"],
        "Nice",
        "the attr saved through /me should come back"
    );

    user.delete().await;
}

#[tokio::test]
async fn update_me_saves_attrs_even_when_name_is_empty() {
    assert_available().await;
    let user = TestUser::new("apimepart");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me",
        Some(json!({
            "name": "",
            "email": "part@example.com",
            "attrs": [{"label": "age", "value": "30"}]
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "partial save should 200");
    assert_eq!(body["profile_saved"], false, "empty name is refused");
    assert_eq!(
        body["attrs_saved"], true,
        "a failing name must not block saving attrs, got {body}"
    );

    let (_, me) = send_as(&user.uid, Method::GET, "/api/auth/me", None).await;
    let age = me["attrs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["label"] == "age")
        .cloned();

    assert_eq!(
        age.unwrap()["value"],
        "30",
        "attrs must be stored despite the profile failure"
    );

    user.delete().await;
}

#[cfg(feature = "test-api")]
#[tokio::test]
async fn update_me_password_change_password() {
    assert_available().await;
    let user = TestUser::new("apimepw");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me/password",
        Some(json!({ "password": "newpass456" })),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "PUT /api/auth/me/password should 204"
    );

    let (status, body) = send(
        Method::POST,
        &format!("/api/users/{}/authenticate", user.uid),
        Some(json!({ "password": "newpass456" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "authenticate should be 200");
    assert_eq!(
        body["ok"], true,
        "the new password should authenticate the user"
    );

    user.delete().await;
}

#[tokio::test]
async fn update_me_password_rejects_short_password() {
    assert_available().await;
    let user = TestUser::new("apimeshort");
    let (status, _) = user.create().await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me/password",
        Some(json!({ "password": "a" })),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a password under security.password_min_length (2 in test config) should be rejected"
    );

    user.delete().await;
}

#[tokio::test]
async fn update_me_without_token_is_unauthorized() {
    assert_available().await;

    let status = req(
        Method::PUT,
        "/api/auth/me",
        Some(json!({"name": "Hijacker", "email": "h@example.com"})),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "PUT /api/auth/me without a token should 401"
    );
}
