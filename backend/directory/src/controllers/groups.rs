use crate::{
    domain::groups::{Group, NewGroup, UpdateGroup},
    domain::ldap::{Gid, Uid},
    repository::ldap as ldap_repo,
};
use portail_core::error::AppError;

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
    let maybe_group = ldap_repo::list_uids()
        .await?
        .into_iter()
        .find(|g| g.gid == gid);
    let user_member = ldap_repo::find_user_members(&uid).await?;
    if let Some(group) = maybe_group {
        let is_last = group.members.iter().all(|m| *m == user_member);
        tracing::info!("{:#?}", group.members);
        if is_last && is_protected(&gid) {
            return Err(AppError::Forbidden(format!(
                "group {} must keep at least one member",
                gid.as_str()
            )));
        }
        if is_last {
            ldap_repo::delete_group(gid).await
        } else {
            ldap_repo::remove_member(gid, uid).await
        }
    } else {
        Ok(())
    }
}
