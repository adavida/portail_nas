use ldap3::{Scope, SearchEntry};

use crate::{
    domain::{groups::group_uids::Members, ldap::Uid},
    repository::ldap::{MapLdap, connect, people_search_base},
};
use portail_core::error::AppError::{self, MemberError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_user_members(uid: &Uid) -> Result<Members, AppError> {
    let mut ldap = connect().await?;
    find_user_dn(&mut ldap, uid)
        .await
        .and_then(|x| Members::from(x.as_str()).ok_or(MemberError(x)))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_user_dn(ldap: &mut ldap3::Ldap, uid: &Uid) -> Result<String, AppError> {
    let filter = format!("(uid={})", uid.as_str());

    let (rs, _res) = ldap
        .search(&people_search_base(), Scope::Subtree, &filter, vec!["dn"])
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;

    rs.into_iter()
        .next()
        .map(|entry| SearchEntry::construct(entry).dn)
        .ok_or_else(|| AppError::NotFound(format!("uid={} not found", uid.as_str(),)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ldap::Cn;
    use crate::repository::ldap::{connect, user_dn};
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};
    use std::collections::HashSet;

    #[tokio::test]
    async fn find_user_dn_returns_dn_of_existing_user() {
        let uid = gen_uid("fdnok");
        if !seed_and_create_user(&uid, "Find OK", "", "pw").await {
            return;
        }

        let mut ldap = connect().await.unwrap();
        let dn = find_user_dn(&mut ldap, &uid).await;
        let _ = ldap.unbind().await;

        assert_eq!(
            dn.unwrap(),
            user_dn(&uid),
            "dn of seeded uid should match the constructed uid= DN"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn find_user_dn_not_found_for_missing_uid() {
        let Ok(mut ldap) = connect().await else {
            return;
        };
        let uid = gen_uid("fdnko");

        let result = find_user_dn(&mut ldap, &uid).await;
        let _ = ldap.unbind().await;

        let err = match result {
            Err(err) => err,
            Ok(dn) => panic!("missing uid should return NotFound, got dn={dn}"),
        };

        assert!(
            matches!(err, AppError::NotFound(_)),
            "missing uid should be NotFound, got {err}"
        );
        assert!(
            err.to_string().contains(uid.as_str()),
            "NotFound message should name uid={}: {err}",
            uid.as_str()
        );
    }

    #[tokio::test]
    async fn find_user_members_returns_uid_member_for_seeded_user() {
        let uid = gen_uid("fmemok");
        if !seed_and_create_user(&uid, "Member OK", "", "pw").await {
            return;
        }

        let member = find_user_members(&uid).await;

        assert_eq!(
            member.unwrap(),
            Members::Uid(uid.clone()),
            "dn uid=... should decode to Members::Uid of the seeded uid"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn find_user_members_not_found_for_missing_uid() {
        let uid = gen_uid("fmemko");

        let result = find_user_members(&uid).await;

        let err = match result {
            Err(err) => err,
            Ok(member) => panic!("missing uid should return NotFound, got member={member:?}"),
        };

        assert!(
            matches!(err, AppError::NotFound(_)),
            "missing uid should be NotFound, got {err}"
        );
    }

    #[tokio::test]
    async fn find_user_members_returns_cn_member_for_cn_rdn_user() {
        let Ok(mut ldap) = connect().await else {
            return;
        };
        let uid = gen_uid("fmemcn");
        let dn = format!(
            "cn={},{}",
            uid.as_str(),
            crate::repository::ldap::people_search_base()
        );

        let mut object_class: HashSet<String> = HashSet::new();
        for class in ["top", "person", "organizationalPerson", "inetOrgPerson"] {
            object_class.insert(class.to_string());
        }
        let attrs = vec![
            ("objectClass".to_string(), object_class),
            (
                "cn".to_string(),
                [uid.as_str().to_string()].into_iter().collect(),
            ),
            (
                "sn".to_string(),
                [format!("cn-{}", uid.as_str())].into_iter().collect(),
            ),
            (
                "uid".to_string(),
                [uid.as_str().to_string()].into_iter().collect(),
            ),
        ];

        if ldap.add(&dn, attrs).await.is_err() {
            let _ = ldap.unbind().await;
            return;
        }

        let member = find_user_members(&uid).await;
        let _ = ldap.delete(&dn).await;
        let _ = ldap.unbind().await;

        assert_eq!(
            member.unwrap(),
            Members::Cn(Cn::try_new(uid.as_str().to_string()).unwrap()),
            "dn cn=... should decode to Members::Cn of the entry cn"
        );
    }

    #[tokio::test]
    async fn find_user_dn_finds_entry_with_cn_rdn() {
        let Ok(mut ldap) = connect().await else {
            return;
        };
        let uid = gen_uid("fdncn");
        let dn = format!(
            "cn={},{}",
            uid.as_str(),
            crate::repository::ldap::people_search_base()
        );

        let mut object_class: HashSet<String> = HashSet::new();
        for class in ["top", "person", "organizationalPerson", "inetOrgPerson"] {
            object_class.insert(class.to_string());
        }
        let attrs = vec![
            ("objectClass".to_string(), object_class),
            (
                "cn".to_string(),
                [uid.as_str().to_string()].into_iter().collect(),
            ),
            (
                "sn".to_string(),
                [format!("cn-{}", uid.as_str())].into_iter().collect(),
            ),
            (
                "uid".to_string(),
                [uid.as_str().to_string()].into_iter().collect(),
            ),
        ];

        if ldap.add(&dn, attrs).await.is_err() {
            let _ = ldap.unbind().await;
            return;
        }

        let found = find_user_dn(&mut ldap, &uid).await;
        let _ = ldap.delete(&dn).await;
        let _ = ldap.unbind().await;

        assert_eq!(
            found.unwrap(),
            dn,
            "search should return the real cn= DN, not a constructed uid= DN"
        );
    }
}
