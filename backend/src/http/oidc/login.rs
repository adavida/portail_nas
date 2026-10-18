use axum::{
    extract::{Json, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::store,
    controllers,
    domain::users::{Password, Uid},
    env::Env,
    error::AppError,
    http::oidc,
};

#[derive(Deserialize)]
pub struct LoginCredentials {
    #[serde(default)]
    pub req: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

/// JSON login endpoint of the provider (the portal is its own IdP) — the
/// SPA posts credentials and receives the callback URL to follow. LDAP
/// bind via the users controller, then session cookie + authorization code.
pub async fn login(
    State(env): State<Env>,
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
    if !controllers::users::authenticate(uid.clone(), password).await? {
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
        [(header::SET_COOKIE, oidc::session_cookie(&env, &sid))],
        Json(json!({ "redirect": redirect })),
    )
        .into_response())
}

fn invalid_credentials() -> AppError {
    AppError::Unauthorized("invalid username or password".into())
}
