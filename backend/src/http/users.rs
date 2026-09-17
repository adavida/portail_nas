use axum::Json;

use crate::{controllers::users::list, domain::users::User, error::AppError};

pub async fn list_users() -> Result<Json<Vec<User>>, AppError> {
    let users = list().await?;
    Ok(Json(users))
}
