use crate::{
    domain::groups::{Gid, Group, NewGroup, UpdateGroup, group_uids::Members},
    domain::users::Uid,
    error::AppError,
    repository::ldap as ldap_repo,
};

const MASTER_GID: &str = "user";
const PROTECTED_GIDS: [&str; 2] = ["admin", "user"];

fn is_protected(gid: &Gid) -> bool {
    PROTECTED_GIDS.contains(&gid.as_str())
}

pub async fn list() -> Result<Vec<Group>, AppError> {
    ldap_repo::list_groups().await
}

pub async fn create(new: NewGroup) -> Result<Group, AppError> {
    ldap_repo::create_group(new).await
}

pub async fn delete(gid: Gid) -> Result<(), AppError> {
    if is_protected(&gid) {
        return Err(AppError::Forbidden(format!(
            "group {} is protected and cannot be deleted",
            gid.as_str()
        )));
    }
    ldap_repo::delete_group(gid).await
}

pub async fn update(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    ldap_repo::update_group(gid, data).await
}

pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    ldap_repo::add_member(gid, uid).await
}

pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    if gid.as_str() == MASTER_GID {
        return Err(AppError::Forbidden(
            "group user is mandatory: members cannot be removed".to_string(),
        ));
    }
    let group = ldap_repo::list_uids()
        .await?
        .into_iter()
        .find(|g| g.gid == gid);
    if let Some(group) = group {
        let is_last = group
            .members
            .iter()
            .all(|m| *m == Members::Uid(uid.clone()));
        if is_last && is_protected(&gid) {
            return Err(AppError::Forbidden(format!(
                "group {} must keep at least one member",
                gid.as_str()
            )));
        }
        if is_last {
            return ldap_repo::delete_group(gid).await;
        }
    }
    ldap_repo::remove_member(gid, uid).await
}
