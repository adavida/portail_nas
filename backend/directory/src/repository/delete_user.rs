use crate::{
    domain::ldap::Uid,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_user(uid: &Uid) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::delete_user(uid).await,
        DirectoryBackend::Sqlite => sqlite::users::delete_user(uid).await,
    }
}
