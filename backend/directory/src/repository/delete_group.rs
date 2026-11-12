use crate::{
    domain::ldap::Gid,
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::delete_group(gid).await,
        DirectoryBackend::Sqlite => sqlite::groups::delete_group(gid).await,
    }
}
