use axum::http::{Method, StatusCode};
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::assert::user_groups::user_groups;
use common::http::send::send;
use common::seed::{fixture::Fixture, nanos::nanos};

#[tokio::test]
async fn add_member_makes_uid_visible_in_group() {
    let fixture = Fixture::new("apim1");
    let (seed_status, (create_status, _)) = fixture.seed().await;

    if seed_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(create_status, StatusCode::CREATED);

    let extra = format!("apim2u{}", &fixture.uid[5..]);
    let (extra_status, _) = send(
        Method::POST,
        "/api/users",
        Some(json!({
            "uid": extra,
            "name": "Extra Member",
            "email": "",
            "password": "secret123",
        })),
    )
    .await;

    if extra_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send(
        Method::POST,
        &format!("/api/groups/{}/members", fixture.gid),
        Some(json!({ "uid": extra.clone() })),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "add member should 204");

    let groups = user_groups(&extra).await;

    assert!(
        groups.contains(&fixture.gid),
        "added member should have the group in GET /api/users, got {groups:?}"
    );
}

#[tokio::test]
async fn remove_member_hides_uid_from_group() {
    let fixture = Fixture::new("apim3");
    let (seed_status, (_create_status, _)) = fixture.seed().await;

    if seed_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let second = format!("apim3x{}", &fixture.uid[5..]);
    let (extra_status, _) = send(
        Method::POST,
        "/api/users",
        Some(json!({
            "uid": second,
            "name": "Second Member",
            "email": "",
            "password": "secret123",
        })),
    )
    .await;

    if extra_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (add_status, _) = send(
        Method::POST,
        &format!("/api/groups/{}/members", fixture.gid),
        Some(json!({ "uid": second.clone() })),
    )
    .await;

    assert_eq!(add_status, StatusCode::NO_CONTENT, "add member should 204");

    let (status, _) = send(
        Method::DELETE,
        &format!("/api/groups/{}/members/{}", fixture.gid, fixture.uid),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "remove member should 204");

    let groups = user_groups(&fixture.uid).await;

    assert!(
        !groups.contains(&fixture.gid),
        "removed member should no longer have the group in GET /api/users, got {groups:?}"
    );
}

#[tokio::test]
async fn add_unknown_uid_is_404() {
    let fixture = Fixture::new("apim4");
    let n = nanos();
    let ghost = format!("apimghost{n}");

    let (seed_status, (create_status, _)) = fixture.seed().await;
    if seed_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(create_status, StatusCode::CREATED);

    let (status, body) = send(
        Method::POST,
        &format!("/api/groups/{}/members", fixture.gid),
        Some(json!({ "uid": ghost })),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "adding non existing uid should 404"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

#[tokio::test]
async fn remove_member_with_unknown_uid_is_404() {
    let n = nanos();

    let (status, body) = send(
        Method::DELETE,
        &format!("/api/groups/apimdead{n}/members/apimghost{n}"),
        None,
    )
    .await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return; // LDAP indisponible => skip
    }

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "unknown uid should 404 before the group is even touched"
    );
    assert!(body.get("error").is_some(), "error body expected");
}
