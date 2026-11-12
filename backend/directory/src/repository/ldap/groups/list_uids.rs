use ldap3::{Scope, SearchEntry};

use crate::{
    domain::groups::GroupUids,
    repository::ldap::{MapLdap, connect, groups_search_base},
};
use portail_core::error::AppError;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_uids() -> Result<Vec<GroupUids>, AppError> {
    let search_base = groups_search_base();
    let mut ldap = connect().await?;

    let (rs, _res) = ldap
        .search(
            &search_base,
            Scope::OneLevel,
            "(objectClass=groupOfNames)",
            vec!["cn", "member"],
        )
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;

    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();
    let out: Vec<GroupUids> = entries
        .into_iter()
        .filter_map(|e| {
            let cn = e.attrs.get("cn").and_then(|v| v.first().cloned());
            let dns = e.attrs.get("member").cloned().unwrap_or_default();
            let members: Vec<String> = dns
                .into_iter()
                .filter(|dn| !dn.eq_ignore_ascii_case(&e.dn))
                .collect();
            GroupUids::from_attrs(cn, &members)
        })
        .collect();
    let _ = ldap.unbind().await;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ldap::{connect, group_dn_from};
    use crate::test_helpers::seed::{cleanup, gen_gid, seed_group, seed_user};
    use ldap3::Mod;

    #[tokio::test]
    async fn list_uids_contains_group_members() {
        let m1 = match seed_user("lum1").await {
            Some(u) => u,
            None => return,
        };
        let m2 = match seed_user("lum2").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("lu");
        assert!(
            seed_group(&gid, &[m1.clone(), m2.clone()]).await,
            "group should be created"
        );

        let groups = list_uids().await.unwrap();
        let found = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");

        assert!(found.members.contains(&m1), "first member should be listed");
        assert!(
            found.members.contains(&m2),
            "second member should be listed"
        );

        cleanup(&gid, &[m1, m2]).await;
    }

    #[tokio::test]
    async fn list_uids_ignores_self_referencing_member() {
        let member = match seed_user("luself").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("luse");

        assert!(
            seed_group(&gid, std::slice::from_ref(&member)).await,
            "group should be created"
        );

        let mut ldap = match connect().await {
            Ok(ldap) => ldap,
            Err(_) => {
                cleanup(&gid, std::slice::from_ref(&member)).await;
                return;
            }
        };
        let added = ldap
            .modify(
                &group_dn_from(&gid),
                vec![Mod::Add(
                    "member".to_string(),
                    [group_dn_from(&gid)].into_iter().collect(),
                )],
            )
            .await
            .map_ldap()
            .and_then(|res| res.success().map_ldap());
        let _ = ldap.unbind().await;
        if added.is_err() {
            cleanup(&gid, std::slice::from_ref(&member)).await;
            return;
        }

        let groups = list_uids().await.unwrap();
        let found = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");

        assert!(
            found.members.contains(&member),
            "real member should be listed"
        );
        assert!(
            !found.members.iter().any(|m| m.as_str() == gid.as_str()),
            "self-referencing placeholder must not be listed as a member"
        );

        cleanup(&gid, std::slice::from_ref(&member)).await;
    }
}
