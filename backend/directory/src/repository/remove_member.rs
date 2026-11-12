use crate::{
    domain::ldap::{Gid, Uid},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::groups::remove_member(gid, uid).await,
        DirectoryBackend::Sqlite => sqlite::groups::remove_member(gid, uid).await,
    }
}
