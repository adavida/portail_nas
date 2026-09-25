use portail_backend::{
    domain::users::{Email, Name, NewUser, Password, Uid},
    error::AppError,
    repository::ldap::users::create_user,
};

use crate::common::ldap::ensure_clean::ensure_clean;

pub async fn seed_user(uid_str: &str) -> Result<(), AppError> {
    ensure_clean(uid_str).await;

    let new = NewUser {
        uid: Uid::try_new(uid_str.to_string()).unwrap(),
        name: Name::try_new("Seed Member".into()).unwrap(),
        email: Email::try_new("".into()).unwrap(),
        password: Password::try_new("secret123".into()).unwrap(),
    };

    create_user(new).await.map(|_| ())
}
