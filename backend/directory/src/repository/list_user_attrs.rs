use crate::{
    domain::{ldap::Uid, users::UserAttr},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_user_attrs(uid: &Uid) -> Result<Vec<UserAttr>, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::list_user_attrs(uid).await,
        DirectoryBackend::Sqlite => sqlite::users::list_user_attrs(uid).await,
    }
}
