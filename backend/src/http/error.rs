use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::error::AppError;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            Self::NotFound(m) => {
                tracing::warn!("404 {}", m);
                let body = Json(json!({"error": m}));
                return (StatusCode::NOT_FOUND, body).into_response();
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
