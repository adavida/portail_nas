use axum::Json;

use crate::domain::health::Health;

pub async fn health() -> Json<Health> {
    Json(Health::ok())
}
