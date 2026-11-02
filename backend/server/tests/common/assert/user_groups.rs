use axum::http::Method;
use serde_json::json;

use crate::common::http::send::send;

pub async fn user_groups(uid: &str) -> Vec<String> {
    let (_, users) = send(Method::GET, "/api/users", None).await;
    users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["uid"] == uid)
        .cloned()
        .unwrap_or(json!({}))
        .get("groups")
        .cloned()
        .unwrap_or(json!([]))
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap().to_string())
        .collect()
}
