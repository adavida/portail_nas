use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Ldap(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Unauthorized(String),
    Forbidden(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ldap(m) => write!(f, "ldap: {m}"),
            Self::NotFound(m) => write!(f, "not found: {m}"),
            Self::BadRequest(m) => write!(f, "bad request: {m}"),
            Self::Internal(m) => write!(f, "internal: {m}"),
            Self::Unauthorized(m) => write!(f, "unauthorized: {m}"),
            Self::Forbidden(m) => write!(f, "forbidden: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            Self::NotFound(m) => {
                tracing::warn!("404 {}", m);
                let body = Json(json!({"error": m}));
                return (StatusCode::NOT_FOUND, body).into_response();
            }
            Self::BadRequest(m) => {
                tracing::warn!("400 {}", m);
                let body = Json(json!({"error": m}));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }
            Self::Unauthorized(m) => {
                tracing::warn!("401 {}", m);
                let body = Json(json!({"error": m}));
                return (StatusCode::UNAUTHORIZED, body).into_response();
            }
            Self::Forbidden(m) => {
                tracing::warn!("403 {}", m);
                let body = Json(json!({"error": m}));
                return (StatusCode::FORBIDDEN, body).into_response();
            }
            Self::Ldap(m) | Self::Internal(m) => {
                tracing::error!("500 {}", m);
                m
            }
        };
        let body = Json(json!({"error": msg}));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
