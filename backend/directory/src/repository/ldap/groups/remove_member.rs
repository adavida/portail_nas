use ldap3::{Mod, Scope, SearchEntry};

use crate::{
    domain::groups::group_uids::Members,
    domain::ldap::{Gid, Uid},
    repository::ldap::{MapLdap, connect, find_group_dn},
};
use portail_core::error::AppError;
use std::collections::HashSet;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let group_dn = find_group_dn(&mut ldap, &gid).await?;

    let (rs, _res) = ldap
        .search(&group_dn, Scope::Base, "(objectClass=*)", vec!["member"])
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let stored: HashSet<String> = rs
        .into_iter()
        .next()
        .map(|entry| SearchEntry::construct(entry).attrs)
        .and_then(|mut attrs| attrs.remove("member"))
        .unwrap_or_default()
        .into_iter()
        .collect();
    let to_delete: HashSet<String> = stored
        .into_iter()
        .filter(|dn| Members::from(dn).is_some_and(|m| m.name() == uid.as_str()))
        .collect();

    if !to_delete.is_empty() {
        ldap.modify(
            &group_dn,
            vec![Mod::Delete("member".to_string(), to_delete)],
        )
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    }
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::groups::group_uids::Members;
    use crate::repository::ldap::{group_dn_from, list_uids, people_search_base, user_dn};
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

    #[tokio::test]
    async fn remove_member_removes_cn_formatted_member() {
        let keep = match seed_user("rmck").await {
            Some(u) => u,
            None => return,
        };
        let drop = match seed_user("rmcd").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("rmcn");
        assert!(
            seed_group(&gid, &[keep.clone(), drop.clone()]).await,
            "group should be created"
        );

        let cn_dn = format!("cn={},{}", drop.as_str(), people_search_base());
        let mut ldap = match connect().await {
            Ok(ldap) => ldap,
            Err(_) => {
                cleanup(&gid, &[keep, drop]).await;
                return;
            }
        };
        let swapped = ldap
            .modify(
                &group_dn_from(&gid),
                vec![
                    Mod::Delete("member".to_string(), [user_dn(&drop)].into_iter().collect()),
                    Mod::Add("member".to_string(), [cn_dn].into_iter().collect()),
                ],
            )
            .await
            .map_ldap()
            .and_then(|res| res.success().map_ldap());
        let _ = ldap.unbind().await;
        if swapped.is_err() {
            cleanup(&gid, &[keep, drop]).await;
            return;
        }

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
            !found.members.iter().any(|m| m.name() == drop.as_str()),
            "cn-form member should be gone even though the entry dn is uid-form"
        );

        cleanup(&gid, &[keep, drop]).await;
    }
}
