use ldap3::Mod;

use crate::{
    domain::groups::Gid,
    domain::users::Uid,
    error::AppError,
    repository::ldap::{MapLdap, connect, find_group_dn, find_user_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let group_dn = find_group_dn(&mut ldap, &gid).await?;
    let member_dn = find_user_dn(&mut ldap, &uid).await?;

    ldap.modify(
        &group_dn,
        vec![Mod::Delete(
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
    async fn remove_member_removes_uid_from_group() {
        let keep = match seed_user("rmk").await {
            Some(u) => u,
            None => return,
        };
        let drop = match seed_user("rmd").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("rm");
        assert!(
            seed_group(&gid, &[keep.clone(), drop.clone()]).await,
            "group should be created"
        );

        remove_member(gid.clone(), drop.clone()).await.unwrap();

        let groups = list_uids().await.unwrap();
        let found = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should still exist");

        assert!(
            found.members.contains(&Members::Uid(keep.clone())),
            "kept member should remain"
        );
        assert!(
            !found.members.contains(&Members::Uid(drop.clone())),
            "removed member should be gone"
        );

        cleanup(&gid, &[keep, drop]).await;
    }
}
