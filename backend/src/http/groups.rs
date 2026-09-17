use axum::{Json, Router, extract::Path, http::StatusCode, routing::get, routing::put};

use crate::{
    controllers::groups::{create, delete, list, update},
    domain::groups::{Gid, Group, NewGroup, UpdateGroup},
    error::AppError,
};

pub fn groups_router() -> Router {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route("/{gid}", put(update_group).delete(delete_group))
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

async fn update_group(
    Path(gid): Path<Gid>,
    Json(payload): Json<UpdateGroup>,
) -> Result<StatusCode, AppError> {
    update(gid, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_group(Path(gid): Path<Gid>) -> Result<StatusCode, AppError> {
    delete(gid).await?;
    Ok(StatusCode::NO_CONTENT)
}
