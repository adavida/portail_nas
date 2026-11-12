use crate::{
    domain::groups::Group,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_groups() -> Result<Vec<Group>, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::list_groups().await,
        DirectoryBackend::Sqlite => sqlite::groups::list_groups().await,
    }
}
