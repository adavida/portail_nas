use axum::http::{Method, StatusCode};

use crate::common::http::send::send;

pub async fn uid_exists(uid: &str) -> bool {
    let (status, users) = send(Method::GET, "/api/users", None).await;

    assert_eq!(status, StatusCode::OK, "GET /api/users should be 200");
    users.as_array().unwrap().iter().any(|u| u["uid"] == uid)
}
