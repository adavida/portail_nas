use crate::{
    domain::groups::GroupUids,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_uids() -> Result<Vec<GroupUids>, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::list_uids().await,
        DirectoryBackend::Sqlite => sqlite::groups::list_uids().await,
    }
}
