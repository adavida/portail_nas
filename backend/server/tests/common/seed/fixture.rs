use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use super::nanos::nanos;
use crate::common::http::send::send;

pub struct Fixture {
    pub uid: String,
    pub gid: String,
}

impl Fixture {
    pub fn new(prefix: &str) -> Self {
        let n = nanos();
        Self {
            uid: format!("{prefix}u{n}"),
            gid: format!("{prefix}g{n}"),
        }
    }

    pub async fn seed(&self) -> (StatusCode, (StatusCode, Value)) {
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
