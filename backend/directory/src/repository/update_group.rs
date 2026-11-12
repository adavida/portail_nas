use crate::{
    domain::{groups::UpdateGroup, ldap::Gid},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_group(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::update_group(gid, data).await,
        DirectoryBackend::Sqlite => sqlite::groups::update_group(gid, data).await,
    }
}
