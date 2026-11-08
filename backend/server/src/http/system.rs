use axum::Json;

use portail_core::error::AppError;

pub async fn system() -> Result<Json<portail_metrics::System>, AppError> {
    Ok(Json(portail_metrics::snapshot().await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn system_handler_returns_serializable_snapshot() {
        let json = system()
            .await
            .expect("snapshot should succeed on linux procfs");

        let value = serde_json::to_value(json.0).expect("system should serialize");
        assert!(
            value["ram_total"]
                .as_str()
                .is_some_and(|ram| !ram.starts_with('0')),
            "ram total should serialize as a positive sized string"
        );
    }
}
