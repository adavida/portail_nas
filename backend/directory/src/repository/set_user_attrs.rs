use crate::{
    domain::{ldap::Uid, users::UserAttr},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn set_user_attrs(uid: &Uid, attrs: Vec<UserAttr>) -> Result<(), AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::set_user_attrs(uid, attrs).await,
        DirectoryBackend::Sqlite => sqlite::users::set_user_attrs(uid, attrs).await,
    }
}
