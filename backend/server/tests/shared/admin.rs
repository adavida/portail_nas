use crate::common::directory::assert_available::assert_available;

use portail_directory::controllers::users::{authenticate, delete, ensure_admin};
use portail_directory::domain::ldap::{Password, Uid};
use portail_directory::repository::{find_user, list_uids};

fn admin_uid() -> Uid {
    Uid::try_new("admin".into()).unwrap()
}

#[tokio::test]
async fn ensure_admin_creates_then_keeps_the_existing_admin() {
    assert_available().await;
    let password = Password::for_authentication("pw123456".into()).unwrap();
    let _ = delete(admin_uid()).await;

    ensure_admin(password.clone()).await.unwrap();

    find_user(&admin_uid())
        .await
        .expect("admin user should exist after ensure_admin");
    let groups = list_uids().await.unwrap();
    let admin_group = groups
        .iter()
        .find(|g| g.gid.as_str() == "admin")
        .expect("admin group should exist after ensure_admin");
    assert!(
        admin_group.members.contains(&admin_uid()),
        "admin should belong to the admin group"
    );
    let user_group = groups
        .iter()
        .find(|g| g.gid.as_str() == "user")
        .expect("default user group should exist after ensure_admin");
    assert!(
        user_group.members.contains(&admin_uid()),
        "admin should belong to the default user group"
    );
    assert!(
        authenticate(admin_uid(), password.clone()).await.unwrap(),
        "bootstrap password should authenticate the fresh admin"
    );

    let replacement = Password::for_authentication("second-pw".into()).unwrap();
    ensure_admin(replacement).await.unwrap();

    assert!(
        authenticate(admin_uid(), password).await.unwrap(),
        "a second ensure_admin must not reset the existing admin password"
    );

    let _ = delete(admin_uid()).await;
}
