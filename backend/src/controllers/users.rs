use crate::{
    domain::users::{NewUser, Password, Uid, UpdateUser, User},
    error::AppError,
    repository::ldap as ldap_repo,
};

pub async fn list() -> Result<Vec<User>, AppError> {
    ldap_repo::list_users().await
}

pub async fn create(new: NewUser) -> Result<User, AppError> {
    ldap_repo::create_user(new).await
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
