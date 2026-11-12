use crate::domain::ldap::{Gid, Uid};
use crate::repository::group_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;

    if !portail_db::groups::exists(&connection, gid.as_str()).map_err(AppError::Db)? {
        return Err(group_not_found(&gid));
    }

    portail_db::groups::remove_member(&connection, gid.as_str(), uid.as_str())
        .map_err(AppError::Db)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::groups::{create_group, delete_group, list_uids};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn remove_member_drops_uid_from_group() {
        let drop = gen_uid("srm1");
        let keep = gen_uid("srm2");
        create_user(new_user(&drop, "Drop", "", "pw"))
            .await
            .unwrap();
        create_user(new_user(&keep, "Keep", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("srm");
        create_group(new_group(&gid, &[drop.clone(), keep.clone()]))
            .await
            .unwrap();

        remove_member(gid.clone(), drop.clone()).await.unwrap();

        let groups = list_uids().await.unwrap();
        let group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should still exist");
        assert!(
            group.members.contains(&keep) && !group.members.contains(&drop),
            "kept member should remain, removed member gone, got {:?}",
            group.members
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&drop).await;
        let _ = delete_user(&keep).await;
    }

    #[tokio::test]
    async fn remove_member_fails_for_missing_group() {
        let uid = gen_uid("srmu");

        let result = remove_member(gen_gid("srmko"), uid).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing group should be NotFound, got {result:?}"
        );
    }
}
