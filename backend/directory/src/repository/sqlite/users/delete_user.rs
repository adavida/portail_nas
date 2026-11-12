use crate::domain::ldap::Uid;
use crate::repository::user_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_user(uid: &Uid) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let rows = portail_db::users::delete(&connection, uid.as_str()).map_err(AppError::Db)?;
    if rows == 0 {
        return Err(user_not_found(uid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::groups::{create_group, delete_group, list_uids};
    use crate::repository::sqlite::users::{create_user, find_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn delete_user_removes_row() {
        let uid = gen_uid("sdel");
        create_user(new_user(&uid, "Delete Me", "", "pw"))
            .await
            .unwrap();

        delete_user(&uid).await.unwrap();

        let again = find_user(&uid).await;
        assert!(
            matches!(again, Err(AppError::NotFound(_))),
            "after delete the user should be NotFound, got {again:?}"
        );
    }

    #[tokio::test]
    async fn delete_user_cascades_memberships() {
        let uid = gen_uid("sdelc");
        create_user(new_user(&uid, "Cascader", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("sdelc");
        create_group(new_group(&gid, std::slice::from_ref(&uid)))
            .await
            .unwrap();

        delete_user(&uid).await.unwrap();

        let groups = list_uids().await.unwrap();
        let group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should still exist");
        assert!(
            !group.members.contains(&uid),
            "membership of the deleted uid should be gone"
        );

        let _ = delete_group(gid).await;
    }
}
