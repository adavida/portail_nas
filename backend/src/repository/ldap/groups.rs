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
