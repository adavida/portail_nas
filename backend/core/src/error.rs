use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiError {
    error: String,
}

#[derive(Debug)]
pub enum AppError {
    Ldap(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Unauthorized(String),
    Forbidden(String),
    InvalidClient(String),
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
            Self::InvalidClient(m) => write!(f, "invalid client: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg, client_auth_required) = match self {
            Self::NotFound(m) => {
                tracing::warn!("404 {}", m);
                (StatusCode::NOT_FOUND, m, false)
            }
            Self::BadRequest(m) => {
                tracing::warn!("400 {}", m);
                (StatusCode::BAD_REQUEST, m, false)
            }
            Self::Unauthorized(m) => {
                tracing::warn!("401 {}", m);
                (StatusCode::UNAUTHORIZED, m, false)
            }
            Self::InvalidClient(m) => {
                tracing::warn!("401 invalid client {}", m);
                (StatusCode::UNAUTHORIZED, m, true)
            }
            Self::Forbidden(m) => {
                tracing::warn!("403 {}", m);
                (StatusCode::FORBIDDEN, m, false)
            }
            Self::Ldap(m) | Self::Internal(m) => {
                tracing::error!("500 {}", m);
                (StatusCode::INTERNAL_SERVER_ERROR, m, false)
            }
        };
        let mut response = (status, Json(ApiError { error: msg })).into_response();
        if client_auth_required {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static("Basic"),
            );
        }
        response
    }
}
