use ldap3::{Scope, SearchEntry};

use crate::{
    domain::groups::Group,
    error::AppError,
    repository::ldap::{MapLdap, connect, groups_search_base, ldap_url},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_groups() -> Result<Vec<Group>, AppError> {
    let search_base = groups_search_base();

    let mut ldap = connect().await?;

    tracing::info!(
        "list_groups url = {} search = {}",
        &ldap_url(),
        &search_base
    );
    let (rs, _res) = ldap
        .search(
            &search_base,
            Scope::OneLevel,
            "(objectClass=groupOfNames)",
            vec!["cn", "o", "description"],
        )
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;

    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();
    let attrs_list: Vec<std::collections::HashMap<String, Vec<String>>> =
        entries.into_iter().map(|e| e.attrs).collect();
    let groups = Group::from_search(attrs_list);
    let _ = ldap.unbind().await;
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::seed::{cleanup, gen_gid, seed_group, seed_user};

    #[tokio::test]
    async fn list_groups_contains_created_group() {
        let member = match seed_user("lgm").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("lg");
        assert!(
            seed_group(&gid, std::slice::from_ref(&member)).await,
            "group should be created"
        );

        let groups = list_groups().await.unwrap();
        let found = groups.iter().find(|g| g.gid == gid);

        assert!(found.is_some(), "created gid should be listed");

        let found = found.unwrap();

        assert_eq!(
            found.name.as_str(),
            "Group Seed",
            "listed name should match"
        );

        cleanup(&gid, &[member]).await;
    }
}
