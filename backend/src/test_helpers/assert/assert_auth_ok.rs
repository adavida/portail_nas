use crate::{
    domain::users::{Password, Uid},
    repository::ldap::authenticate_user,
};

pub(crate) async fn assert_auth_ok(uid: &str, password: &str) {
    let ok = authenticate_user(
        &Uid::try_new(uid.to_string()).unwrap(),
        Password::try_new(password.to_string()).unwrap(),
    )
    .await
    .unwrap();

    assert!(
        ok,
        "authenticate should succeed for uid={uid} with correct password"
    );
}
