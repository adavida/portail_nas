use crate::domain::groups::GroupUids;
use crate::domain::ldap::{Gid, Uid};
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_uids() -> Result<Vec<GroupUids>, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;

    let records = portail_db::groups::find_all(&connection).map_err(AppError::Db)?;
    let mut groups = Vec::with_capacity(records.len());
    for record in records {
        let gid = Gid::try_new(record.gid).map_err(|e| AppError::Db(e.to_string()))?;
        let member_rows =
            portail_db::groups::members_of(&connection, gid.as_str()).map_err(AppError::Db)?;
        groups.push(GroupUids {
            gid,
            members: member_rows
                .into_iter()
                .filter_map(|uid| Uid::try_new(uid).ok())
                .collect(),
        });
    }
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::groups::{create_group, delete_group};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn list_uids_contains_group_members() {
        let uid = gen_uid("slum");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("slu");
        let new = new_group(&gid, std::slice::from_ref(&uid));

        create_group(new.clone()).await.unwrap();

        let groups = list_uids().await.unwrap();
        let found = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");

        assert!(
            found.members.contains(&uid),
            "member uid should be listed, got {:?}",
            found.members
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&uid).await;
    }
}
