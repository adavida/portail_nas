use crate::{
    domain::groups::{Description, Gid, Members, Name, NewGroup},
    domain::users::{NewUser, Password, Uid, UpdateUser, User},
    error::AppError,
    repository::ldap as ldap_repo,
};

const DEFAULT_GROUP: &str = "user";

pub async fn list() -> Result<Vec<User>, AppError> {
    ldap_repo::list_users().await
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
        ldap_repo::add_member(gid, uid.clone()).await
    } else {
        let new = NewGroup {
            gid: gid.clone(),
            name: Name::try_new(DEFAULT_GROUP.into())
                .map_err(|e| AppError::Internal(e.to_string()))?,
            description: Description::try_new(String::new()),
            members: Members::try_new(vec![uid.clone()])
                .map_err(|e| AppError::Internal(e.to_string()))?,
        };
        ldap_repo::create_group(new).await.map(|_| ())
    }
}

pub async fn update_password(uid: Uid, password: Password) -> Result<(), AppError> {
    ldap_repo::update_user_password(uid, password).await
}

pub async fn delete(uid: Uid) -> Result<(), AppError> {
    ldap_repo::delete_user(uid).await
}

pub async fn update(uid: Uid, data: UpdateUser) -> Result<(), AppError> {
    ldap_repo::update_user(uid, data).await
}

pub async fn authenticate(uid: Uid, password: Password) -> Result<bool, AppError> {
    ldap_repo::authenticate_user(uid, password).await
}
