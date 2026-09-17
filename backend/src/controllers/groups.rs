use crate::{
    domain::groups::{Gid, Group, NewGroup},
    error::AppError,
    repository::ldap as ldap_repo,
};

pub async fn list() -> Result<Vec<Group>, AppError> {
    ldap_repo::list_groups().await
}

pub async fn create(new: NewGroup) -> Result<Group, AppError> {
    ldap_repo::create_group(new).await
}

pub async fn delete(gid: Gid) -> Result<(), AppError> {
    ldap_repo::delete_group(gid).await
}
