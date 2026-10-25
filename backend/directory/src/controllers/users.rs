use crate::{
    domain::groups::{Description, Gid, Members, Name, NewGroup},
    domain::users::{NewUser, Password, Uid, UpdateUser, User},
    repository::ldap as ldap_repo,
};
use portail_core::error::AppError;

const DEFAULT_GROUP: &str = "user";

pub async fn list() -> Result<Vec<User>, AppError> {
    let mut users = ldap_repo::list_users().await?;
    let uids = ldap_repo::list_uids().await?;

    users
        .iter_mut()
        .for_each(|user| user.groups = uids.iter().filter_map(|g| g.group_of(user)).collect());

    Ok(users)
}

pub async fn create(new: NewUser) -> Result<User, AppError> {
    let user = ldap_repo::create_user(new).await?;
    let _ = ensure_user_group(&user.uid).await;
    Ok(user)
}

async fn ensure_user_group(uid: &Uid) -> Result<(), AppError> {
    let gid =
        Gid::try_new(DEFAULT_GROUP.to_string()).map_err(|e| AppError::Internal(e.to_string()))?;
    let exists = ldap_repo::list_groups().await?.iter().any(|g| g.gid == gid);
    if exists {
        return ldap_repo::add_member(gid, uid.clone()).await;
    }
    let new = NewGroup {
        gid: gid.clone(),
        name: Name::try_new(DEFAULT_GROUP.into()).map_err(|e| AppError::Internal(e.to_string()))?,
        description: Description::try_new(String::new()),
        members: Members::try_new(vec![uid.clone()])
            .map_err(|e| AppError::Internal(e.to_string()))?,
    };
    if ldap_repo::create_group(new).await.is_ok() {
        return Ok(());
    }
    ldap_repo::add_member(gid, uid.clone()).await
}

pub async fn update_password(uid: Uid, password: Password) -> Result<(), AppError> {
    ldap_repo::update_user_password(&uid, password).await
}

pub async fn delete(uid: Uid) -> Result<(), AppError> {
    ldap_repo::delete_user(&uid).await
}

pub async fn update(uid: Uid, data: UpdateUser) -> Result<(), AppError> {
    ldap_repo::update_user(&uid, data).await
}

pub async fn authenticate(uid: Uid, password: Password) -> Result<bool, AppError> {
    ldap_repo::authenticate_user(&uid, password).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::groups::group_uids::Members;
    use crate::repository::ldap::list_uids;
    use crate::test_helpers::seed::{cleanup, gen_uid, new_user};

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

        let groups = list_uids().await.unwrap();
        let user_group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("the default group must exist after two creates");
        assert!(
            user_group.members.contains(&Members::Uid(uid_a.clone()))
                && user_group.members.contains(&Members::Uid(uid_b.clone())),
            "both concurrent users must belong to '{DEFAULT_GROUP}', got {:?}",
            user_group.members
        );

        cleanup(&gid, &[uid_a, uid_b]).await;
    }
}
