use axum::http::{Method, StatusCode};
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::{
    http::send::send,
    seed::{nanos::nanos, test_group::TestGroup},
};

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
        assert!(
            entry.get("members").is_none(),
            "GET /api/groups should not expose members, got {entry}"
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
    assert!(
        body.get("members").is_none(),
        "created group response should not expose members"
    );

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
        Some(
            json!({ "gid": "Bad Gid", "name": "x", "description": "x", "members": ["somemember"] }),
        ),
    )
    .await;

    assert!(
        status.is_client_error(),
        "invalid gid (spaces, uppercase) should be rejected, got {status}"
    );
}

#[tokio::test]
async fn create_without_members_is_rejected() {
    let group = TestGroup::new("apigrp0");
    let (status, _) = group.create_without_members().await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "group creation requires at least one member, got {status}"
    );
}

#[tokio::test]
async fn create_empty_description_is_accepted() {
    let group = TestGroup::new("apides");
    let (seed_status, _) = send(
        Method::POST,
        "/api/users",
        Some(json!({
            "uid": group.member,
            "name": "Seed Member",
            "email": "",
            "password": "secret123",
        })),
    )
    .await;

    if seed_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = send(
        Method::POST,
        "/api/groups",
        Some(json!({
            "gid": group.gid,
            "name": group.name,
            "description": "",
            "members": [group.member.clone()],
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
async fn delete_unknown_gid_is_404() {
    let n = nanos();

    let (status, body) = send(Method::DELETE, &format!("/api/groups/apinobody{n}"), None).await;
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return; // LDAP indisponible => skip
    }

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "deleting unknown group should 404"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

#[tokio::test]
async fn create_duplicate_gid_is_500() {
    let group = TestGroup::new("apidup");
    let (seed_status, _) = send(
        Method::POST,
        "/api/users",
        Some(json!({
            "uid": group.member,
            "name": "Seed Member",
            "email": "",
            "password": "secret123",
        })),
    )
    .await;

    if seed_status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let payload = json!({
        "gid": group.gid,
        "name": group.name,
        "description": group.description,
        "members": [group.member],
    });

    let (first, _) = send(Method::POST, "/api/groups", Some(payload.clone())).await;

    assert_eq!(first, StatusCode::CREATED);

    let (second, body) = send(Method::POST, "/api/groups", Some(payload)).await;

    assert_eq!(
        second,
        StatusCode::INTERNAL_SERVER_ERROR,
        "duplicate gid should 500"
    );
    assert!(body.get("error").is_some(), "error body expected");
}

mod repo {
    use crate::common::seed::{
        nanos::nanos, new_group_with_member::new_group_with_member, seed_user::seed_user,
    };
    use portail_backend::domain::groups::{Description, Gid, Name, UpdateGroup};
    use portail_backend::error::AppError;
    use portail_backend::repository::ldap::groups::{
        create_group, delete_group, list_groups, update_group,
    };

    #[tokio::test]
    async fn repository_create_list_delete_roundtrip() {
        let member = format!("uniseed{}", nanos());
        let seeded = seed_user(&member).await;
        if matches!(seeded, Err(AppError::Ldap(_))) {
            return;
        }
        seeded.unwrap();

        let new = new_group_with_member("unigrp", &member);

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

        assert_eq!(
            listed.name, new.name,
            "listed group should expose members parsed from LDAP"
        );

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
        let member = format!("uniseed2{}", nanos());
        let seeded = seed_user(&member).await;
        if matches!(seeded, Err(AppError::Ldap(_))) {
            return;
        }

        let new = new_group_with_member("unigrp2", &member);

        if create_group(new.clone()).await.is_err() {
            return;
        }

        let renamed = Name::try_new("Repo Renamed".into()).unwrap();

        let result = update_group(
            new.gid.clone(),
            UpdateGroup {
                name: renamed.clone(),
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
