use ldap3::{Mod, Scope, SearchEntry};
use std::collections::HashSet;

use crate::domain::groups::{Gid, Group, NewGroup, UpdateGroup};
use crate::domain::users::Uid;
use crate::error::AppError;

use super::{MapLdap, connect, group_dn_from, groups_search_base, ldap_base, ldap_url, user_dn};

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

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_group(new: NewGroup) -> Result<Group, AppError> {
    let base = ldap_base();
    let dn = group_dn_from(new.gid.as_str());
    let mut ldap = connect().await?;

    for uid in new.members.as_slice() {
        if !user_exists(&mut ldap, uid).await {
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

#[tracing::instrument(level = "debug", skip_all)]
async fn user_exists(ldap: &mut ldap3::Ldap, uid: &Uid) -> bool {
    let dn = user_dn(uid);
    let filter = format!("(uid={})", uid.as_str());
    match ldap.search(&dn, Scope::Base, &filter, vec!["uid"]).await {
        Ok(result) => result
            .success()
            .map(|(rs, _)| !rs.is_empty())
            .unwrap_or(false),
        Err(_) => false,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn add_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let group_dn = group_dn_from(gid.as_str());
    let member_dn = user_dn(&uid);
    let mut ldap = connect().await?;

    if !user_exists(&mut ldap, &uid).await {
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

#[tracing::instrument(level = "debug", skip_all)]
pub async fn remove_member(gid: Gid, uid: Uid) -> Result<(), AppError> {
    let group_dn = group_dn_from(gid.as_str());
    let member_dn = user_dn(&uid);
    let mut ldap = connect().await?;

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

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_group(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    let dn = group_dn_from(gid.as_str());
    let mut ldap = connect().await?;

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

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    let dn = group_dn_from(gid.as_str());
    let mut ldap = connect().await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}
