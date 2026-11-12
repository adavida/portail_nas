use crate::{
    domain::users::{NewUser, User},
    repository::{ldap, sqlite},
};
use portail_core::{
    config::{Config, DirectoryBackend},
    error::AppError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_user(new: NewUser) -> Result<User, AppError> {
    match Config::global().directory.backend {
        DirectoryBackend::Ldap => ldap::users::create_user(new).await,
        DirectoryBackend::Sqlite => sqlite::users::create_user(new).await,
    }
}
