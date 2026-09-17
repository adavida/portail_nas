use axum::{extract::Path, http::StatusCode, Json};

use crate::{
    controllers::users::{create, list, update_password},
    domain::users::{NewUser, UpdatePassword, User},
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
    Path(uid): Path<String>,
    Json(payload): Json<UpdatePassword>,
) -> Result<StatusCode, AppError> {
    update_password(uid, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}
