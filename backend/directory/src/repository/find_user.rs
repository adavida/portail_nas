use crate::{
    domain::ldap::Uid,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_user(uid: &Uid) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::find_user(uid).await,
        DirectoryBackend::Sqlite => sqlite::users::find_user(uid).await,
    }
}
