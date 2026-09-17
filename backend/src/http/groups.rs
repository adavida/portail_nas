use axum::{Json, Router, extract::Path, http::StatusCode, routing::get, routing::put};

use crate::{
    controllers::groups::{add_member, create, delete, list, remove_member, update},
    domain::groups::{Gid, Group, NewGroup, UpdateGroup},
    domain::users::Uid,
    error::AppError,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct MemberPayload {
    uid: Uid,
}

pub fn groups_router() -> Router {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route("/{gid}", put(update_group).delete(delete_group))
        .route("/{gid}/members", axum::routing::post(post_member))
        .route("/{gid}/members/{uid}", axum::routing::delete(delete_member))
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

async fn post_member(
    Path(gid): Path<Gid>,
    Json(payload): Json<MemberPayload>,
) -> Result<StatusCode, AppError> {
    add_member(gid, payload.uid).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_member(Path((gid, uid)): Path<(Gid, Uid)>) -> Result<StatusCode, AppError> {
    remove_member(gid, uid).await?;
    Ok(StatusCode::NO_CONTENT)
}
