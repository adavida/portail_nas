use axum::{Extension, Json, http::StatusCode};
use portail_core::error::AppError;
use portail_directory::{controllers::users::update, domain::users::UpdateUser};

use super::current_uid::current_uid;
use crate::auth::middleware::AuthUser;

pub async fn update_me(
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(payload): Json<UpdateUser>,
) -> Result<StatusCode, AppError> {
    let uid = current_uid(&claims)?;
    update(uid, payload).await?;
    Ok(StatusCode::NO_CONTENT)
}
