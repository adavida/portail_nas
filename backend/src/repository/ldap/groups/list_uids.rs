use ldap3::{Scope, SearchEntry};

use crate::{
    domain::groups::GroupUids,
    error::AppError,
    repository::ldap::{MapLdap, connect, groups_search_base},
};

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
            GroupUids::from_attrs(cn, &dns)
        })
        .collect();
    let _ = ldap.unbind().await;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::groups::group_uids::Members;
    use crate::test_helpers::seed::{cleanup, gen_gid, seed_group, seed_user};

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

        assert!(
            found.members.contains(&Members::Uid(m1.clone())),
            "first member should be listed"
        );
        assert!(
            found.members.contains(&Members::Uid(m2.clone())),
            "second member should be listed"
        );

        cleanup(&gid, &[m1, m2]).await;
    }
}
