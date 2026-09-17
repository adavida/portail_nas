use crate::{
    domain::users::{NewUser, UpdatePassword, User},
    error::AppError,
    repository::ldap as ldap_repo,
};

pub async fn list() -> Result<Vec<User>, AppError> {
    ldap_repo::list_users().await
}

pub async fn create(new: NewUser) -> Result<User, AppError> {
    ldap_repo::create_user(new).await
}

pub async fn update_password(uid: String, req: UpdatePassword) -> Result<(), AppError> {
    ldap_repo::update_user_password(uid, req).await
}
