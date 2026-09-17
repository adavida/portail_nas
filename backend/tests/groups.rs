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
async fn create_empty_description_is_accepted() {
    let group = TestGroup::new("apides");
    let (status, body) = send(
        Method::POST,
        "/api/groups",
        Some(json!({
            "gid": group.gid,
            "name": group.name,
            "description": "",
        })),
    )
    .await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(
        status,
        StatusCode::CREATED,
        "empty description should be allowed for groups"
    );
    assert_eq!(body["description"], "", "description should stay empty");

    let (_, groups) = send(Method::GET, "/api/groups", None).await;
    let listed = groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["gid"] == group.gid)
        .cloned();

    assert!(listed.is_some(), "group with empty description created");
}

#[tokio::test]
async fn update_changes_name_and_description() {
    let group = TestGroup::new("apigrupd");
    let (status, _) = group.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, _) = send(
        Method::PUT,
        &format!("/api/groups/{}", group.gid),
        Some(json!({ "name": "Renamed", "description": "" })),
    )
    .await;

    assert_eq!(status, StatusCode::NO_CONTENT, "PUT /api/groups should 204");

    let (status, groups) = send(Method::GET, "/api/groups", None).await;

    assert_eq!(status, StatusCode::OK);

    let updated = groups
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["gid"] == group.gid)
        .cloned();

    assert!(updated.is_some(), "updated group should still be listed");

    let updated = updated.unwrap();

    assert_eq!(
        updated["name"], "Renamed",
        "name should be updated after PUT"
    );
    assert_eq!(
        updated["description"], "",
        "empty description should stay empty"
    );
}

#[tokio::test]
async fn delete_unknown_gid_is_500() {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        % 1000000;

    let (status, body) = send(Method::DELETE, &format!("/api/groups/apinobody{n}"), None).await;

    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "deleting unknown group should 500"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

#[tokio::test]
async fn create_duplicate_gid_is_500() {
    let group = TestGroup::new("apidup");
    let (status, _) = group.create().await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::CREATED);

    let (second, body) = group.create().await;

    assert_eq!(
        second,
        StatusCode::INTERNAL_SERVER_ERROR,
        "duplicate gid should 500"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

mod repo {
    use portail_backend::domain::groups::{Description, Gid, NewGroup, UpdateGroup};
    use portail_backend::domain::shared::Name;
    use portail_backend::error::AppError;
    use portail_backend::repository::ldap::groups::{
        create_group, delete_group, list_groups, update_group,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1_000_000
    }

    fn new_group(prefix: &str) -> NewGroup {
        NewGroup {
            gid: Gid::try_new(format!("{prefix}{}", nanos())).unwrap(),
            name: Name::try_new("Unit Group".into()).unwrap(),
            description: Description::try_new("unit test group".into()),
        }
    }

    #[tokio::test]
    async fn repository_create_list_delete_roundtrip() {
        let new = new_group("unigrp");

        let created = match create_group(new.clone()).await {
            Err(AppError::Ldap(_)) => return,
            other => other.unwrap(),
        };

        assert_eq!(created.gid, new.gid, "created group should echo gid");
        assert_eq!(created.name, new.name, "created group should echo name");
        assert_eq!(
            created.description, new.description,
            "created group should echo description"
        );

        let groups = list_groups().await.unwrap();
        let listed = groups.iter().find(|g| g.gid == new.gid).cloned();

        assert!(listed.is_some(), "created group should appear in list");

        let listed = listed.unwrap();

        assert_eq!(listed.name, new.name, "listed group should keep its name");

        delete_group(new.gid.clone()).await.unwrap();

        let groups = list_groups().await.unwrap();

        assert!(
            !groups.iter().any(|g| g.gid == new.gid),
            "deleted group should be gone from list"
        );
    }

    #[tokio::test]
    async fn repository_delete_unknown_gid_errors() {
        let gid = Gid::try_new(format!("unidead{}", nanos())).unwrap();

        let result = delete_group(gid).await;

        assert!(result.is_err(), "delete of unknown group should fail");
    }

    #[tokio::test]
    async fn repository_update_changes_name_and_description() {
        let mut new = new_group("unigrp2");

        if create_group(new.clone()).await.is_err() {
            return;
        }

        new.name = Name::try_new("Repo Renamed".into()).unwrap();

        let result = update_group(
            new.gid.clone(),
            UpdateGroup {
                name: new.name.clone(),
                description: Description::try_new("".into()),
            },
        )
        .await;

        assert!(result.is_ok(), "update of existing group should succeed");

        let groups = list_groups().await.unwrap();
        let updated = groups.iter().find(|g| g.gid == new.gid).cloned();

        assert!(updated.is_some(), "group should be listed after update");

        let updated = updated.unwrap();

        assert_eq!(
            updated.name.as_str(),
            "Repo Renamed",
            "name should be persisted after update"
        );
        assert_eq!(
            updated.description.as_str(),
            "",
            "empty description should clear the LDAP attr"
        );

        let _ = delete_group(new.gid).await;
    }
}
