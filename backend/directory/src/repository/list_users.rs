use crate::{
    domain::users::User,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_users() -> Result<Vec<User>, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::list_users().await,
        DirectoryBackend::Sqlite => sqlite::users::list_users().await,
    }
}
