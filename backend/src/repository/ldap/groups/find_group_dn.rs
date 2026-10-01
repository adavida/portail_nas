use ldap3::{Scope, SearchEntry};

use crate::{
    domain::groups::Gid,
    error::AppError,
    repository::ldap::{MapLdap, groups_search_base},
};

/// Retourne le DN reel de l'entree portant `cn=<gid>` sous `ou=groups`.
/// Les entrees heritees peuvent avoir un RDN different (ex: `o=...`) :
/// seul un search retrouve le DN exact, d'ou cette fonction au lieu de
/// construire le DN.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_group_dn(ldap: &mut ldap3::Ldap, gid: &Gid) -> Result<String, AppError> {
    let filter = format!("(cn={})", gid.as_str());

    let (rs, _res) = ldap
        .search(&groups_search_base(), Scope::Subtree, &filter, vec!["dn"])
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;

    rs.into_iter()
        .next()
        .map(|entry| SearchEntry::construct(entry).dn)
        .ok_or_else(|| AppError::NotFound(format!("gid={} not found", gid.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ldap::{connect, group_dn_from, user_dn};
    use crate::test_helpers::seed::{cleanup, gen_gid, seed_group, seed_user};
    use std::collections::HashSet;

    #[tokio::test]
    async fn find_group_dn_returns_dn_of_existing_group() {
        let member = match seed_user("fgdok").await {
            Some(u) => u,
            None => return,
        };
        let gid = gen_gid("fgok");
        assert!(
            seed_group(&gid, std::slice::from_ref(&member)).await,
            "group should be created"
        );

        let mut ldap = connect().await.unwrap();
        let dn = find_group_dn(&mut ldap, &gid).await;
        let _ = ldap.unbind().await;

        assert_eq!(
            dn.unwrap(),
            group_dn_from(&gid),
            "dn of seeded gid should match the constructed cn= DN"
        );

        cleanup(&gid, &[member]).await;
    }

    #[tokio::test]
    async fn find_group_dn_not_found_for_missing_gid() {
        let Ok(mut ldap) = connect().await else {
            return; // LDAP indisponible => skip
        };
        let gid = gen_gid("fgnko");

        let result = find_group_dn(&mut ldap, &gid).await;
        let _ = ldap.unbind().await;

        let err = match result {
            Err(err) => err,
            Ok(dn) => panic!("missing gid should return NotFound, got dn={dn}"),
        };

        assert!(
            matches!(err, AppError::NotFound(_)),
            "missing gid should be NotFound, got {err}"
        );
        assert!(
            err.to_string().contains(gid.as_str()),
            "NotFound message should name gid={}: {err}",
            gid.as_str()
        );
    }

    #[tokio::test]
    async fn find_group_dn_finds_entry_with_legacy_rdn() {
        let member = match seed_user("fglmem").await {
            Some(u) => u,
            None => return,
        };
        let Ok(mut ldap) = connect().await else {
            return; // LDAP indisponible => skip
        };
        let gid = gen_gid("fgl");
        let dn = format!("o={},{}", gid.as_str(), groups_search_base());

        let mut object_class: HashSet<String> = HashSet::new();
        for class in ["top", "groupOfNames"] {
            object_class.insert(class.to_string());
        }
        let attrs = vec![
            ("objectClass".to_string(), object_class),
            (
                "cn".to_string(),
                [gid.as_str().to_string()].into_iter().collect(),
            ),
            (
                "member".to_string(),
                [user_dn(&member)].into_iter().collect(),
            ),
        ];

        let added = ldap.add(&dn, attrs).await;
        if !matches!(&added, Ok(res) if res.rc == 0) {
            let _ = ldap.unbind().await;
            cleanup(&gid, &[]).await;
            return; // add impossible (contraintes) => skip
        }

        let found = find_group_dn(&mut ldap, &gid).await;
        let _ = ldap.delete(&dn).await;
        let _ = ldap.unbind().await;

        assert_eq!(
            found.unwrap(),
            dn,
            "search should return the real o= DN, not a constructed cn= DN"
        );

        cleanup(&gid, &[member]).await;
    }
}
