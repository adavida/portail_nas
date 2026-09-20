use ldap3::{LdapConnAsync, Mod, Scope, SearchEntry};
use std::collections::HashSet;

use crate::domain::groups::{Gid, Group, NewGroup, UpdateGroup};
use crate::domain::users::Uid;
use crate::error::AppError;

use super::{
    MapLdap, connect_admin, group_dn_from, groups_search_base, ldap_base, ldap_url, user_dn,
    user_dn_from,
};

pub async fn list_groups() -> Result<Vec<Group>, AppError> {
    let base = ldap_base();
    let search_base = groups_search_base(&base);

    let (conn, mut ldap) = LdapConnAsync::new(&ldap_url()).await.map_ldap()?;
    ldap3::drive!(conn);

    let (rs, _res) = ldap
        .search(
            &search_base,
            Scope::OneLevel,
            "(objectClass=groupOfNames)",
            vec!["cn", "o", "description", "member"],
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
    let dn = group_dn_from(new.gid.as_str(), &base);
    let mut ldap = connect_admin(&base).await?;

    for uid in new.members.as_slice() {
        if !user_exists(&mut ldap, uid, &base).await {
            let _ = ldap.unbind().await;
            return Err(AppError::NotFound(format!(
                "uid={} not found in ou=people",
                uid.as_str()
            )));
        }
    }

    ldap.add(&dn, new.to_attrs(&base))
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let _ = ldap.unbind().await;

    Ok(Group {
        gid: new.gid,
        name: new.name,
        description: new.description,
        members: new
            .members
            .as_slice()
            .iter()
            .map(|u| u.as_str().to_string())
            .collect(),
    })
}

async fn user_exists(ldap: &mut ldap3::Ldap, uid: &Uid, base: &str) -> bool {
    let dn = user_dn(uid, base);
    let filter = format!("(uid={})", uid.as_str());
    match ldap.search(&dn, Scope::Base, &filter, vec!["uid"]).await {
        Ok(result) => result
            .success()
            .map(|(rs, _)| !rs.is_empty())
            .unwrap_or(false),
        Err(_) => false,
    }
}

pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let base = ldap_base();
    let group_dn = group_dn_from(gid.as_str(), &base);
    let member_dn = user_dn(&uid, &base);
    let mut ldap = connect_admin(&base).await?;

    if !user_exists(&mut ldap, &uid, &base).await {
        let _ = ldap.unbind().await;
        return Err(AppError::NotFound(format!(
            "uid={} not found",
            uid.as_str()
        )));
    }

    ldap.modify(
        &group_dn,
        vec![Mod::Add(
            "member".to_string(),
            [member_dn.clone()].into_iter().collect(),
        )],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let base = ldap_base();
    let group_dn = group_dn_from(gid.as_str(), &base);
    let member_dn = user_dn(&uid, &base);
    let mut ldap = connect_admin(&base).await?;

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

pub async fn update_group(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = group_dn_from(gid.as_str(), &base);
    let mut ldap = connect_admin(&base).await?;

    let set_description = if data.description.as_str().is_empty() {
        HashSet::new()
    } else {
        [data.description.as_str().to_string()]
            .into_iter()
            .collect()
    };

    ldap.modify(
        &dn,
        vec![
            Mod::Replace(
                "o".to_string(),
                [data.name.as_str().to_string()].into_iter().collect(),
            ),
            Mod::Replace("description".to_string(), set_description),
        ],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = group_dn_from(gid.as_str(), &base);
    let mut ldap = connect_admin(&base).await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}
