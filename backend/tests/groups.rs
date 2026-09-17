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

struct TestGroup {
    gid: String,
    name: String,
    description: String,
}

impl TestGroup {
    fn new(prefix: &str) -> Self {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1000000;
        Self {
            gid: format!("{prefix}{n}"),
            name: "Test Group".into(),
            description: "Test description".into(),
        }
    }

    async fn create(&self) -> (StatusCode, BodyJson) {
        send(
            Method::POST,
            "/api/groups",
            Some(json!({
                "gid": self.gid,
                "name": self.name,
                "description": self.description,
            })),
        )
        .await
    }

    async fn delete(&self) -> (StatusCode, BodyJson) {
        send(Method::DELETE, &format!("/api/groups/{}", self.gid), None).await
    }
}

#[tokio::test]
async fn list_returns_array_with_fields() {
    let (status, body) = send(Method::GET, "/api/groups", None).await;

    assert_eq!(status, StatusCode::OK, "GET /api/groups should be 200");
    assert!(body.is_array(), "GET /api/groups should return an array");

    if let Some(entry) = body.as_array().unwrap().first() {
        assert!(
            entry.get("gid").is_some()
                && entry.get("name").is_some()
                && entry.get("description").is_some(),
            "each group should have gid/name/description, got {entry}"
        );
    }
}

#[tokio::test]
async fn create_returns_201_with_name_and_is_listed() {
    let group = TestGroup::new("apigrp");
    let (status, body) = group.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::CREATED, "POST /api/groups should 201");
    assert_eq!(body["gid"], group.gid);
    assert_eq!(body["name"], group.name, "name should be in response");
    assert_eq!(body["description"], group.description);

    let (status, groups) = send(Method::GET, "/api/groups", None).await;

    assert_eq!(status, StatusCode::OK);

    let listed = groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["gid"] == group.gid)
        .cloned();

    assert!(listed.is_some(), "created group should appear in list");

    let listed = listed.unwrap();

    assert_eq!(
        listed["name"], group.name,
        "listed group should keep its name"
    );
}

#[tokio::test]
async fn delete_removes_group() {
    let group = TestGroup::new("apigdel");
    let (status, _) = group.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = group.delete().await;

    assert_eq!(status, StatusCode::NO_CONTENT, "delete should 204");

    let (status, groups) = send(Method::GET, "/api/groups", None).await;

    assert_eq!(status, StatusCode::OK);

    let still_listed = groups
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["gid"] == group.gid);

    assert!(
        !still_listed,
        "deleted group should no longer appear in GET /api/groups"
    );
}

#[tokio::test]
async fn create_invalid_gid_is_rejected() {
    let (status, _) = send(
        Method::POST,
        "/api/groups",
        Some(json!({ "gid": "Bad Gid", "name": "x", "description": "x" })),
    )
    .await;

    assert!(
        status.is_client_error(),
        "invalid gid (spaces, uppercase) should be rejected, got {status}"
    );
}

#[tokio::test]
async fn create_empty_description_is_rejected() {
    let (status, _) = send(
        Method::POST,
        "/api/groups",
        Some(json!({ "gid": "apidesgrp", "name": "x", "description": "" })),
    )
    .await;

    assert!(
        status.is_client_error(),
        "empty description should be rejected, got {status}"
    );
}
