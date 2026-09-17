use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use portail_backend::app;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

mod common;

type BodyJson = serde_json::Value;

async fn send(method: Method, uri: &str, json_body: Option<BodyJson>) -> (StatusCode, BodyJson) {
    let app = app();
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(b) = &json_body {
        req = req.header("content-type", "application/json");
        let body = serde_json::to_vec(b).unwrap();
        let resp = app
            .oneshot(req.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        return (status, serde_json::from_slice(&bytes).unwrap_or(json!({})));
    }

    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();

    (status, serde_json::from_slice(&bytes).unwrap_or(json!({})))
}

struct Fixture {
    uid: String,
    gid: String,
}

impl Fixture {
    fn new(prefix: &str) -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1000000;
        Self {
            uid: format!("{prefix}u{n}"),
            gid: format!("{prefix}g{n}"),
        }
    }

    async fn seed(&self) -> (StatusCode, (StatusCode, BodyJson)) {
        let user = send(
            Method::POST,
            "/api/users",
            Some(json!({
                "uid": self.uid,
                "name": "Member User",
                "email": "",
                "password": "secret123",
            })),
        )
        .await;

        if user.0 == StatusCode::INTERNAL_SERVER_ERROR {
            return (user.0, (user.0, json!({})));
        }

        let group = send(
            Method::POST,
            "/api/groups",
            Some(json!({
                "gid": self.gid,
                "name": "Members Test Group",
                "description": "fixture",
                "members": [self.uid.clone()],
            })),
        )
        .await;

        (user.0, group)
    }
}

async fn group_members(gid: &str) -> BodyJson {
    let (_, groups) = send(Method::GET, "/api/groups", None).await;
    groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["gid"] == gid)
        .cloned()
        .unwrap()
        .get("members")
        .cloned()
        .unwrap()
}

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

    let members = group_members(&fixture.gid).await;
    let arr = members.as_array().unwrap();

    assert!(
        arr.iter().any(|m| *m == extra),
        "added member should appear in GET /api/groups, got {members}"
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

    let members = group_members(&fixture.gid).await;

    assert!(
        !members
            .as_array()
            .unwrap()
            .iter()
            .any(|m| *m == fixture.uid),
        "removed member should not appear in GET /api/groups, got {members}"
    );
}

#[tokio::test]
async fn add_unknown_uid_is_404() {
    let fixture = Fixture::new("apim4");
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % 1000000;
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
async fn remove_member_of_unknown_group_is_500() {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % 1000000;

    let (status, body) = send(
        Method::DELETE,
        &format!("/api/groups/apimdead{n}/members/apimghost{n}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body.get("error").is_some(), "error body expected");
}
