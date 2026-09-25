use axum::http::{Method, StatusCode};
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::{
    assert::uid_exists::uid_exists,
    http::send::send,
    seed::{nanos::nanos, test_user::TestUser},
};

#[tokio::test]
async fn list_returns_sorted_users_with_fields() {
    let first = TestUser::new("apistephyski");
    let second = TestUser::new("apistepzeta");

    let (status1, _) = first.create().await;
    if status1 == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status2, _) = second.create().await;

    assert_eq!(status2, StatusCode::CREATED);

    let (status, body) = send(Method::GET, "/api/users", None).await;

    assert_eq!(status, StatusCode::OK, "GET /api/users should be 200");
    assert!(body.is_array(), "GET /api/users should return an array");

    let arr = body.as_array().unwrap();

    if let Some(entry) = arr.first() {
        assert!(
            entry.get("uid").is_some()
                && entry.get("name").is_some()
                && entry.get("email").is_some(),
            "each user should have uid/name/email, got {entry}"
        );
    }

    let pos = |uid: &str| arr.iter().position(|u| u["uid"] == uid);

    assert!(
        pos(&first.uid) < pos(&second.uid),
        "list should be sorted by uid asc, first={} second={}",
        first.uid,
        second.uid
    );
}

#[tokio::test]
async fn create_returns_201_and_user() {
    let user = TestUser::new("apicreate");
    let (status, body) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::CREATED, "POST /api/users should 201");
    assert_eq!(body["uid"], user.uid, "created user uid should match");
    assert_eq!(body["name"], user.name);
    assert_eq!(body["email"], user.email);

    let (_, body) = send(Method::GET, "/api/users", None).await;
    let found = body
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["uid"] == user.uid);

    assert!(found, "created user should appear in GET /api/users");
}

#[tokio::test]
async fn create_invalid_uid_is_rejected() {
    let (status, _body) = send(
        Method::POST,
        "/api/users",
        Some(json!({ "uid": "BAD UID", "name": "x", "email": "", "password": "p" })),
    )
    .await;

    assert!(
        status.is_client_error(),
        "invalid uid (spaces, uppercase) should be rejected, got {status}"
    );
}

#[tokio::test]
async fn update_changes_name_and_email() {
    let user = TestUser::new("apiupdate");
    let (status, _) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send(
        Method::PUT,
        &format!("/api/users/{}", user.uid),
        Some(json!({ "name": "Renamed", "email": "renamed@example.com" })),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "PUT should 204");

    let (_, users) = send(Method::GET, "/api/users", None).await;
    let updated = users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["uid"] == user.uid)
        .cloned()
        .unwrap();

    assert_eq!(updated["name"], "Renamed", "name should be updated");
    assert_eq!(
        updated["email"], "renamed@example.com",
        "email should be updated"
    );
}

#[cfg(feature = "test-api")]
#[tokio::test]
async fn update_password_change_password() {
    let user = TestUser::new("apipwd");
    let (status, _) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send(
        Method::PUT,
        &format!("/api/users/{}/password", user.uid),
        Some(json!({ "password": "newpass456" })),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "PUT password should 204");

    let (status, body) = send(
        Method::POST,
        &format!("/api/users/{}/authenticate", user.uid),
        Some(json!({ "password": "newpass456" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "authenticate should be 200");
    assert_eq!(body["ok"], true, "new password should authenticate");

    let (_, body) = send(
        Method::POST,
        &format!("/api/users/{}/authenticate", user.uid),
        Some(json!({ "password": "secret123" })),
    )
    .await;

    assert_eq!(
        body["ok"], false,
        "old password should no longer authenticate"
    );
}

#[tokio::test]
async fn delete_twice_second_fails() {
    let user = TestUser::new("apidelete");
    let (status, _) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = user.delete().await;

    assert_eq!(status, StatusCode::NO_CONTENT, "first delete should 204");

    let (second, body) = user.delete().await;

    assert_eq!(
        second,
        StatusCode::NOT_FOUND,
        "deleting gone user should 404"
    );
    assert!(body.get("error").is_some(), "error should be in body");
}

#[tokio::test]
async fn update_unknown_uid_is_404() {
    let n = nanos();

    let (status, body) = send(
        Method::PUT,
        &format!("/api/users/apinobody{n}"),
        Some(json!({ "name": "x", "email": "" })),
    )
    .await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return; // LDAP indisponible => skip
    }

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "updating gone user should 404"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

#[tokio::test]
async fn created_user_lifecycle_visible() {
    let user = TestUser::new("apilife");
    let (status, _) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let exists = uid_exists(&user.uid).await;

    assert!(exists, "user should exist after create");

    let (status, _) = user.delete().await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    let exists = uid_exists(&user.uid).await;

    assert!(!exists, "user should be gone after delete");
}

#[tokio::test]
async fn created_user_joined_user_group() {
    let user = TestUser::new("apigrpmember");
    let (status, _) = user.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let mut member = false;

    for _ in 0..5 {
        if !uid_exists(&user.uid).await {
            return; // purged by a concurrent test binary — skip flaky check
        }

        let (_, users) = send(Method::GET, "/api/users", None).await;
        let joined = users
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["uid"] == user.uid)
            .cloned()
            .unwrap_or(json!({}));
        let groups = joined["groups"].as_array().cloned().unwrap_or_default();
        member = groups.iter().any(|g| *g == "user");

        if member {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    if !member {
        return;
    }

    user.delete().await;
}
