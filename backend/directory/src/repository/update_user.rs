use crate::{
    domain::{ldap::Uid, users::UpdateUser},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user(uid: &Uid, data: UpdateUser) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::update_user(uid, data).await,
        DirectoryBackend::Sqlite => sqlite::users::update_user(uid, data).await,
    }
}
