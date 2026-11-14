use axum::{Extension, Json, http::StatusCode};
use portail_core::error::AppError;
use portail_directory::{controllers::users::update_password, domain::ldap::UpdatePassword};

use super::current_uid::current_uid;
use crate::auth::middleware::AuthUser;

pub async fn update_me_password(
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(payload): Json<UpdatePassword>,
) -> Result<StatusCode, AppError> {
    let uid = current_uid(&claims)?;
    update_password(uid, payload.password).await?;
    Ok(StatusCode::NO_CONTENT)
}
