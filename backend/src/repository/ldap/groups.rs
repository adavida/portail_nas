use ldap3::{LdapConnAsync, Scope, SearchEntry};

use crate::{
    domain::groups::{Gid, Group, NewGroup},
    error::AppError,
};

use super::{MapLdap, connect_admin, ldap_base, ldap_url};

pub async fn list_groups() -> Result<Vec<Group>, AppError> {
    let base = ldap_base();
    let search_base = format!("ou=groups,{base}");

    let (conn, mut ldap) = LdapConnAsync::new(&ldap_url()).await.map_ldap()?;
    ldap3::drive!(conn);

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

pub async fn create_group(new: NewGroup) -> Result<Group, AppError> {
    let base = ldap_base();
    let dn = new.dn(&base);
    let mut ldap = connect_admin(&base).await?;

    ldap.add(&dn, new.to_attrs())
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let _ = ldap.unbind().await;

    Ok(Group {
        gid: new.gid,
        name: new.name,
        description: new.description,
    })
}

pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = format!("cn={},ou=groups,{base}", gid.as_str());
    let mut ldap = connect_admin(&base).await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::groups::Description;
    use crate::domain::shared::Name;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1_000_000
    }

    fn new_group(prefix: &str) -> NewGroup {
        NewGroup {
            gid: Gid::try_new(format!("{prefix}{}", nanos())).unwrap(),
            name: Name::try_new("Unit Group".into()).unwrap(),
            description: Description::try_new("unit test group".into()).unwrap(),
        }
    }

    #[tokio::test]
    async fn create_list_delete_roundtrip() {
        let new = new_group("unigrp");

        let created = match create_group(new.clone()).await {
            Err(AppError::Ldap(_)) => return,
            other => other.unwrap(),
        };

        assert_eq!(created.gid, new.gid, "created group should echo gid");
        assert_eq!(created.name, new.name, "created group should echo name");
        assert_eq!(
            created.description, new.description,
            "created group should echo description"
        );

        let groups = list_groups().await.unwrap();
        let listed = groups.iter().find(|g| g.gid == new.gid).cloned();

        assert!(listed.is_some(), "created group should appear in list");

        let listed = listed.unwrap();

        assert_eq!(listed.name, new.name, "listed group should keep its name");

        delete_group(new.gid.clone()).await.unwrap();

        let groups = list_groups().await.unwrap();

        assert!(
            !groups.iter().any(|g| g.gid == new.gid),
            "deleted group should be gone from list"
        );
    }

    #[tokio::test]
    async fn delete_unknown_gid_errors() {
        let gid = Gid::try_new(format!("unidead{}", nanos())).unwrap();

        let result = delete_group(gid).await;

        assert!(result.is_err(), "delete of unknown group should fail");
    }
}
