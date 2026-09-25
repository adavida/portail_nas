use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use super::nanos::nanos;
use crate::common::http::send::send;

pub struct TestGroup {
    pub gid: String,
    pub name: String,
    pub description: String,
    pub member: String,
}

impl TestGroup {
    pub fn new(prefix: &str) -> Self {
        let n = nanos();
        Self {
            gid: format!("{prefix}{n}"),
            name: "Test Group".into(),
            description: "Test description".into(),
            member: format!("{prefix}m{n}"),
        }
    }

    pub async fn create(&self) -> (StatusCode, Value) {
        let (user_status, _) = send(
            Method::POST,
            "/api/users",
            Some(json!({
                "uid": self.member,
                "name": "Seed Member",
                "email": "",
                "password": "secret123",
            })),
        )
        .await;

        if user_status == StatusCode::INTERNAL_SERVER_ERROR {
            return (user_status, json!({}));
        }

        send(
            Method::POST,
            "/api/groups",
            Some(json!({
                "gid": self.gid,
                "name": self.name,
                "description": self.description,
                "members": [self.member.clone()],
            })),
        )
        .await
    }

    pub async fn create_without_members(&self) -> (StatusCode, Value) {
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

    pub async fn delete(&self) -> (StatusCode, Value) {
        send(Method::DELETE, &format!("/api/groups/{}", self.gid), None).await
    }
}
