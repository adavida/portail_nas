use axum::Json;

use crate::domain::health::{Health, health_status};

pub async fn health() -> Json<Health> {
    Json(health_status())
}
