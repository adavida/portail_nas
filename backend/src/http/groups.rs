use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};

use crate::{
    controllers::groups::{create, delete, list},
    domain::groups::{Gid, Group, NewGroup},
    error::AppError,
};

pub fn groups_router() -> Router {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route("/{gid}", axum::routing::delete(delete_group))
}

async fn list_groups() -> Result<Json<Vec<Group>>, AppError> {
    let groups = list().await?;
    Ok(Json(groups))
}

async fn create_group(
    Json(payload): Json<NewGroup>,
) -> Result<(StatusCode, Json<Group>), AppError> {
    let group = create(payload).await?;
    Ok((StatusCode::CREATED, Json(group)))
}

async fn delete_group(Path(gid): Path<Gid>) -> Result<StatusCode, AppError> {
    delete(gid).await?;
    Ok(StatusCode::NO_CONTENT)
}
