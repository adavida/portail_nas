use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::error::AppError;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let msg = match self {
            Self::Ldap(m) | Self::Internal(m) => m,
        };
        let body = Json(json!({"error": msg}));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
