use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use super::nanos::nanos;
use crate::common::http::send::send;

pub struct TestUser {
    pub uid: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

impl TestUser {
    pub fn new(prefix: &str) -> Self {
        let n = nanos();
        Self {
            uid: format!("{prefix}{n}"),
            name: "Api User".into(),
            email: "api@example.com".into(),
            password: "secret123".into(),
        }
    }

    pub async fn create(&self) -> (StatusCode, Value) {
        send(
            Method::POST,
            "/api/users",
            Some(json!({
                "uid": self.uid,
                "name": self.name,
                "email": self.email,
                "password": self.password,
            })),
        )
        .await
    }

    pub async fn delete(&self) -> (StatusCode, Value) {
        send(Method::DELETE, &format!("/api/users/{}", self.uid), None).await
    }
}
