use axum::{
    extract::{Json, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{auth::store, http::oidc};
use portail_core::config::Config;
use portail_core::error::AppError;
use portail_directory::domain::users::{Password, Uid};

#[derive(Serialize)]
pub struct Login {
    redirect: String,
}

#[derive(Deserialize)]
pub struct LoginCredentials {
    #[serde(default)]
    pub req: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

pub async fn login(
    State(config): State<Config>,
    Json(f): Json<LoginCredentials>,
) -> Result<Response, AppError> {
    if f.req.is_empty() {
        return Err(AppError::BadRequest("missing req parameter".into()));
    }
    let Some(mut flow) = store::take_pending(&f.req) else {
        return Err(AppError::BadRequest(
            "unknown or expired login request".into(),
        ));
    };
    let Ok(uid) = Uid::try_new(f.username) else {
        return Err(invalid_credentials());
    };
    let Ok(password) = Password::try_new(f.password) else {
        return Err(invalid_credentials());
    };
    if !portail_directory::controllers::users::authenticate(uid.clone(), password).await? {
        return Err(invalid_credentials());
    }

    flow.sub = uid.as_str().to_string();
    oidc::enrich_flow(&mut flow).await?;
    let sid = store::create_session(&flow.sub).map_err(AppError::Internal)?;
    let redirect_uri = flow.redirect_uri.clone();
    let state = flow.state.clone();
    let code = store::save_code(flow).map_err(AppError::Internal)?;
    let redirect = oidc::callback_location(&redirect_uri, &code, state.as_deref());

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, oidc::session_cookie(&config, &sid))],
        Json(Login { redirect }),
    )
        .into_response())
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized("invalid username or password".into())
}
