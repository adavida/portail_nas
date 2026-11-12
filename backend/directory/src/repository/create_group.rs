use crate::{
    domain::groups::{Group, NewGroup},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_group(new: NewGroup) -> Result<Group, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::create_group(new).await,
        DirectoryBackend::Sqlite => sqlite::groups::create_group(new).await,
    }
}
