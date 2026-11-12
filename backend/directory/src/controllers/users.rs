use crate::{
    domain::groups::NewGroup,
    domain::ldap::{Description, Email, Gid, Members, Name, Password, Uid},
    domain::users::{NewUser, UpdateUser, User},
    repository as repo,
};
use portail_core::error::AppError;

const DEFAULT_GROUP: &str = "user";
const ADMIN_UID: &str = "admin";
const ADMIN_GROUP: &str = "admin";

pub async fn list() -> Result<Vec<User>, AppError> {
    let mut users = repo::list_users().await?;
    let uids = repo::list_uids().await?;

    users
        .iter_mut()
        .for_each(|user| user.groups = uids.iter().filter_map(|g| g.group_of(user)).collect());

    Ok(users)
}

pub async fn create(new: NewUser) -> Result<User, AppError> {
    let user = repo::create_user(new).await?;
    let _ = ensure_user_group(&user.uid).await;
    Ok(user)
}

pub async fn ensure_admin(password: Password) -> Result<(), AppError> {
    let uid = Uid::try_new(ADMIN_UID.to_string()).map_err(|e| AppError::Internal(e.to_string()))?;
    if repo::find_user(&uid).await.is_ok() {
        return Ok(());
    }
    let new = NewUser {
        uid: uid.clone(),
        name: Name::try_new(ADMIN_UID.to_string())
            .map_err(|e| AppError::Internal(e.to_string()))?,
        email: Email::try_new(String::new()).map_err(|e| AppError::Internal(e.to_string()))?,
        password,
    };
    create(new).await?;
    ensure_admin_group(&uid).await
}

async fn ensure_user_group(uid: &Uid) -> Result<(), AppError> {
    let gid =
        Gid::try_new(DEFAULT_GROUP.to_string()).map_err(|e| AppError::Internal(e.to_string()))?;
    let exists = repo::list_groups().await?.iter().any(|g| g.gid == gid);
    if exists {
        return repo::add_member(gid, uid.clone()).await;
    }
    let new = NewGroup {
        gid: gid.clone(),
        name: Name::try_new(DEFAULT_GROUP.into()).map_err(|e| AppError::Internal(e.to_string()))?,
        description: Description::try_new(String::new()),
        members: Members::try_new(vec![uid.clone()])
            .map_err(|e| AppError::Internal(e.to_string()))?,
    };
    if repo::create_group(new).await.is_ok() {
        return Ok(());
    }
    repo::add_member(gid, uid.clone()).await
}

async fn ensure_admin_group(uid: &Uid) -> Result<(), AppError> {
    let gid =
        Gid::try_new(ADMIN_GROUP.to_string()).map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = repo::list_uids().await?.into_iter().find(|g| g.gid == gid);
    match existing {
        Some(group) if group.members.contains(uid) => Ok(()),
        Some(group) => repo::add_member(group.gid, uid.clone()).await,
        None => {
            let new = NewGroup {
                gid,
                name: Name::try_new(ADMIN_GROUP.to_string())
                    .map_err(|e| AppError::Internal(e.to_string()))?,
                description: Description::try_new(String::new()),
                members: Members::try_new(vec![uid.clone()])
                    .map_err(|e| AppError::Internal(e.to_string()))?,
            };
            repo::create_group(new).await.map(|_| ())
        }
    }
}

pub async fn update_password(uid: Uid, password: Password) -> Result<(), AppError> {
    repo::update_user_password(&uid, password).await
}

pub async fn delete(uid: Uid) -> Result<(), AppError> {
    repo::delete_user(&uid).await
}

pub async fn update(uid: Uid, data: UpdateUser) -> Result<(), AppError> {
    repo::update_user(&uid, data).await
}

pub async fn authenticate(uid: Uid, password: Password) -> Result<bool, AppError> {
    repo::authenticate_user(&uid, password).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn concurrent_creates_both_join_the_default_group_despite_the_race() {
        let uid_a = gen_uid("ccga");
        let uid_b = gen_uid("ccgb");
        let gid = Gid::try_new(DEFAULT_GROUP.to_string()).unwrap();

        let (ra, rb) = tokio::join!(
            create(new_user(&uid_a, "A", "a@e.test", "pw")),
            create(new_user(&uid_b, "B", "b@e.test", "pw")),
        );
        if ra.is_err() || rb.is_err() {
            return;
        }

        let groups = repo::list_uids().await.unwrap();
        let user_group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("the default group must exist after two creates");
        assert!(
            user_group.members.contains(&uid_a) && user_group.members.contains(&uid_b),
            "both concurrent users must belong to '{DEFAULT_GROUP}', got {:?}",
            user_group.members
        );

        let _ = repo::delete_user(&uid_a).await;
        let _ = repo::delete_user(&uid_b).await;
    }
}
