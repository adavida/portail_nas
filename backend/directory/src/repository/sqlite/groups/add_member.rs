use crate::domain::ldap::{Gid, Uid};
use crate::repository::{group_not_found, user_not_found};
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;

    if !portail_db::users::exists(&connection, uid.as_str()).map_err(AppError::Db)? {
        return Err(user_not_found(&uid));
    }
    if !portail_db::groups::exists(&connection, gid.as_str()).map_err(AppError::Db)? {
        return Err(group_not_found(&gid));
    }

    portail_db::groups::add_member(&connection, gid.as_str(), uid.as_str())
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
    async fn add_member_adds_uid_to_group() {
        let seed = gen_uid("samseed");
        let added = gen_uid("samadd");
        create_user(new_user(&seed, "Seed", "", "pw"))
            .await
            .unwrap();
        create_user(new_user(&added, "Added", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("samg");
        create_group(new_group(&gid, std::slice::from_ref(&seed)))
            .await
            .unwrap();

        add_member(gid.clone(), added.clone()).await.unwrap();

        let groups = list_uids().await.unwrap();
        let group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");
        assert!(
            group.members.contains(&added),
            "group should contain the added uid"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&added).await;
        let _ = delete_user(&seed).await;
    }

    #[tokio::test]
    async fn add_member_fails_for_missing_uid() {
        let seed = gen_uid("samnseed");
        create_user(new_user(&seed, "Seed", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("samng");
        create_group(new_group(&gid, std::slice::from_ref(&seed)))
            .await
            .unwrap();

        let ghost = gen_uid("samghost");

        let result = add_member(gid.clone(), ghost).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing uid should be NotFound, got {result:?}"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&seed).await;
    }
}
