use crate::{
    domain::groups::{Group, NewGroup, UpdateGroup},
    domain::ldap::{Gid, Uid},
    repository as repo,
};
use portail_core::error::AppError;

const MASTER_GID: &str = "user";
const PROTECTED_GIDS: [&str; 2] = ["admin", "user"];

fn is_protected(gid: &Gid) -> bool {
    PROTECTED_GIDS.contains(&gid.as_str())
}

pub async fn list() -> Result<Vec<Group>, AppError> {
    repo::list_groups().await
}

pub async fn create(new: NewGroup) -> Result<Group, AppError> {
    repo::create_group(new).await
}

pub async fn delete(gid: Gid) -> Result<(), AppError> {
    if is_protected(&gid) {
        return Err(AppError::Forbidden(format!(
            "group {} is protected and cannot be deleted",
            gid.as_str()
        )));
    }
    repo::delete_group(gid).await
}

pub async fn update(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    repo::update_group(gid, data).await
}

pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    repo::add_member(gid, uid).await
}

pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    if gid.as_str() == MASTER_GID {
        return Err(AppError::Forbidden(
            "group user is mandatory: members cannot be removed".to_string(),
        ));
    }
    let maybe_group = repo::list_uids().await?.into_iter().find(|g| g.gid == gid);
    repo::find_user(&uid).await?;
    if let Some(group) = maybe_group {
        let is_last = group.members.iter().all(|m| m == &uid);
        if is_last && is_protected(&gid) {
            return Err(AppError::Forbidden(format!(
                "group {} must keep at least one member",
                gid.as_str()
            )));
        }
        if is_last {
            repo::delete_group(gid).await
        } else {
            repo::remove_member(gid, uid).await
        }
    } else {
        Ok(())
    }
}
