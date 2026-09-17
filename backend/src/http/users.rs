use axum::{Json, http::StatusCode};

use crate::{
    controllers::users::{create, list},
    domain::users::{NewUser, User},
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
