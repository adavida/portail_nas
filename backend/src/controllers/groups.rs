use crate::{
    domain::groups::{Gid, Group, NewGroup, UpdateGroup},
    domain::users::Uid,
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

pub async fn update(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    ldap_repo::update_group(gid, data).await
}

pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    ldap_repo::add_member(gid, uid).await
}

pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    ldap_repo::remove_member(gid, uid).await
}
