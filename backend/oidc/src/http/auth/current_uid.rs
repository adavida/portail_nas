use portail_directory::domain::ldap::{Uid, UidError};

use crate::auth::Claims;
use portail_core::error::AppError;

pub(crate) fn current_uid(claims: &Claims) -> Result<Uid, AppError> {
    let raw = claims
        .preferred_username
        .as_deref()
        .ok_or(AppError::Unauthorized(
            "token has no preferred_username".into(),
        ))?;
    Uid::try_new(raw.to_string()).map_err(|e: UidError| AppError::BadRequest(e.to_string()))
}
