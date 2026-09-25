use crate::{
    domain::users::{Password, Uid},
    error::AppError,
    repository::ldap::authenticate_user,
};

pub(crate) async fn assert_auth_err(uid: &str, password: &str) {
    let uid_res = Uid::try_new(uid.to_string());
    let pw_res = Password::try_new(password.to_string());

    let result = match (uid_res, pw_res) {
        (Ok(u), Ok(p)) => authenticate_user(&u, p).await,
        _ => Err(AppError::Internal("missing uid/password".into())),
    };

    assert!(
        result.is_err(),
        "authenticate should error for uid={uid} with empty password"
    );
}
