use ldap3::{Mod, Scope, SearchEntry};

use crate::{
    domain::ldap::Uid,
    repository::ldap::{MapLdap, connect, find_user_dn, groups_search_base},
};
use portail_core::error::AppError;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_user(uid: &Uid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_user_dn(&mut ldap, uid).await?;
    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let (rs, _res) = ldap
        .search(
            &groups_search_base(),
            Scope::OneLevel,
            &format!("(member={dn})"),
            vec!["member"],
        )
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    for entry in rs {
        let group = SearchEntry::construct(entry);
        let members = group.attrs.get("member").cloned().unwrap_or_default();
        if members.len() > 1 {
            let to_delete: std::collections::HashSet<String> = [dn.clone()].into_iter().collect();
            ldap.modify(
                &group.dn,
                vec![Mod::Delete("member".to_string(), to_delete)],
            )
            .await
            .map_ldap()?
            .success()
            .map_ldap()?;
        } else {
            ldap.delete(&group.dn)
                .await
                .map_ldap()?
                .success()
                .map_ldap()?;
        }
    }
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ldap::list_uids;
    use crate::test_helpers::seed::{
        ensure_clean, gen_gid, gen_uid, seed_and_create_user, seed_group, seed_user,
    };

    #[tokio::test]
    async fn delete_user_removes_entry() {
        let uid = gen_uid("del");
        if !seed_and_create_user(&uid, "Delete Me", "", "pw").await {
            return;
        }

        delete_user(&uid).await.unwrap();

        let again = delete_user(&uid).await;

        assert!(
            again.is_err(),
            "second delete should fail: entry no longer exists"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn delete_user_keeps_other_group_members() {
        let keep = match seed_user("delk").await {
            Some(u) => u,
            None => return,
        };
        let drop = gen_uid("deld");
        if !seed_and_create_user(&drop, "Drop Me", "", "pw").await {
            return;
        }
        let gid = gen_gid("delgrp");
        if !seed_group(&gid, &[keep.clone(), drop.clone()]).await {
            let _ = delete_user(&drop).await;
            ensure_clean(&keep).await;
            return;
        }

        delete_user(&drop).await.unwrap();

        let groups = list_uids().await.unwrap();
        let members = groups
            .iter()
            .find(|g| g.gid == gid)
            .map(|g| g.members.clone())
            .unwrap_or_default();
        assert!(
            members.contains(&keep),
            "other member should remain, got {members:?}"
        );
        assert!(
            !members.contains(&drop),
            "deleted user must no longer be a member, got {members:?}"
        );

        let _ = delete_user(&keep).await;
    }

    #[tokio::test]
    async fn delete_user_of_last_member_removes_group() {
        let only = gen_uid("delo");
        if !seed_and_create_user(&only, "Only Me", "", "pw").await {
            return;
        }
        let gid = gen_gid("dellast");
        if !seed_group(&gid, std::slice::from_ref(&only)).await {
            let _ = delete_user(&only).await;
            return;
        }

        delete_user(&only).await.unwrap();

        let groups = list_uids().await.unwrap();
        assert!(
            groups.iter().all(|g| g.gid != gid),
            "group with no remaining member must be gone"
        );

        let _ = delete_user(&only).await;
    }
}
