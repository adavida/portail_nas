use ldap3::Mod;

use crate::{
    domain::groups::Gid,
    domain::users::Uid,
    error::AppError,
    repository::ldap::{MapLdap, connect, find_group_dn, find_user_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let group_dn = find_group_dn(&mut ldap, &gid).await?;
    let member_dn = find_user_dn(&mut ldap, &uid).await?;

    ldap.modify(
        &group_dn,
        vec![Mod::Add(
            "member".to_string(),
            [member_dn].into_iter().collect(),
        )],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::groups::group_uids::Members;
    use crate::repository::ldap::list_uids;
    use crate::test_helpers::seed::{cleanup, gen_gid, seed_group, seed_user};

    #[tokio::test]
    async fn add_member_adds_uid_to_group() {
        let seed = match seed_user("amseed").await {
            Some(u) => u,
            None => return,
        };
        let added = match seed_user("amadd").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("amg");
        assert!(
            seed_group(&gid, std::slice::from_ref(&seed)).await,
            "group should be created"
        );

        add_member(gid.clone(), added.clone()).await.unwrap();

        let groups = list_uids().await.unwrap();
        let group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");

        assert!(
            group.members.contains(&Members::Uid(added.clone())),
            "group should contain the added uid"
        );

        cleanup(&gid, &[seed, added]).await;
    }

    #[tokio::test]
    async fn add_member_fails_for_missing_uid() {
        let seed = match seed_user("amnseed").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("amng");
        assert!(
            seed_group(&gid, std::slice::from_ref(&seed)).await,
            "group should be created"
        );

        let ghost = Uid::try_new(format!("amnghost{}", std::process::id())).unwrap();

        let result = add_member(gid.clone(), ghost).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing uid should be NotFound, got {result:?}"
        );

        cleanup(&gid, &[seed]).await;
    }
}
