use crate::{
    domain::ldap::{Password, Uid},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn authenticate_user(uid: &Uid, password: Password) -> Result<bool, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::authenticate_user(uid, password).await,
        DirectoryBackend::Sqlite => sqlite::users::authenticate_user(uid, password).await,
    }
}
