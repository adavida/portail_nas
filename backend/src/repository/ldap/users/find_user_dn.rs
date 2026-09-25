use ldap3::{Scope, SearchEntry};

use crate::{
    domain::users::Uid,
    error::AppError,
    repository::ldap::{MapLdap, people_search_base},
};

/// Retourne le DN reel de l'entree portant `uid=<uid>` sous `ou=people`.
/// Les entrees heritees peuvent avoir un RDN `cn=...` : seul un search
/// retrouve le DN exact, d'ou cette fonction au lieu de construire le DN.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_user_dn(ldap: &mut ldap3::Ldap, uid: &Uid) -> Result<String, AppError> {
    let filter = format!("(uid={})", uid.as_str());

    let (rs, _res) = ldap
        .search(
            &people_search_base(),
            Scope::Subtree,
            &filter,
            Vec::<String>::new(),
        )
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
            return; // LDAP indisponible => skip
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
    async fn find_user_dn_finds_entry_with_cn_rdn() {
        let Ok(mut ldap) = connect().await else {
            return; // LDAP indisponible => skip
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
            return; // add impossible (contraintes) => skip
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
