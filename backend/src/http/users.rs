use axum::{Json, Router, extract::Path, http::StatusCode, routing::get, routing::put};
use serde_json::json;

use crate::{
    controllers::users::{authenticate, create, delete, list, update, update_password},
    domain::users::{NewUser, Uid, UpdatePassword, UpdateUser, User},
    error::AppError,
};

pub(crate) fn users_router() -> Router<crate::env::Env> {
    let r = Router::<crate::env::Env>::new()
        .route("/", get(list_users).post(create_user))
        .route("/{uid}", put(update_user).delete(delete_user))
        .route("/{uid}/password", put(update_user_password));

    #[cfg(feature = "test-api")]
    let r = r.route(
        "/{uid}/authenticate",
        axum::routing::post(authenticate_user),
    );

    r
}

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

pub async fn authenticate_user(
    Path(uid): Path<Uid>,
    Json(payload): Json<UpdatePassword>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ok = authenticate(uid, payload.password).await?;
    Ok(Json(json!({ "ok": ok })))
}
