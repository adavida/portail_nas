use crate::{
    domain::ldap::{Password, Uid},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user_password(uid: &Uid, password: Password) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::update_user_password(uid, password).await,
        DirectoryBackend::Sqlite => sqlite::users::update_user_password(uid, password).await,
    }
}
