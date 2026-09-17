use axum::{extract::Path, http::StatusCode, Json};

use crate::{
    controllers::users::{create, delete, list, update, update_password},
    domain::users::{NewUser, Uid, UpdatePassword, UpdateUser, User},
    error::AppError,
};

pub async fn list_users() -> Result<Json<Vec<User>>, AppError> {
    let users = list().await?;
    Ok(Json(users))
}

pub async fn create_user(
    Json(payload): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = create(payload).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user_password(
    Path(uid): Path<Uid>,
    Json(payload): Json<UpdatePassword>,
) -> Result<StatusCode, AppError> {
    update_password(uid, payload.password).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(Path(uid): Path<Uid>) -> Result<StatusCode, AppError> {
    delete(uid).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_user(
    Path(uid): Path<Uid>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, AppError> {
    update(uid, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}
