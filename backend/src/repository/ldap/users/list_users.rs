use ldap3::{Scope, SearchEntry};

use crate::{
    domain::users::User,
    error::AppError,
    repository::ldap::{MapLdap, connect, people_search_base},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_users() -> Result<Vec<User>, AppError> {
    let search_base = people_search_base();

    let mut ldap = connect().await?;

    let (rs, _res) = ldap_conn_search(&mut ldap, &search_base).await?;
    let _ = ldap.unbind().await;

    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();

    let mut users: Vec<User> = entries
        .into_iter()
        .filter_map(|e| User::from_search(e.attrs).ok())
        .collect();

    users.sort_by(|a, b| a.uid.as_str().cmp(b.uid.as_str()));

    Ok(users)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn ldap_conn_search(
    ldap: &mut ldap3::Ldap,
    base: &str,
) -> Result<(Vec<ldap3::ResultEntry>, ldap3::LdapResult), AppError> {
    ldap.search(
        base,
        Scope::Subtree,
        "(objectClass=inetOrgPerson)",
        vec!["uid", "cn", "displayName", "mail"],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};

    #[tokio::test]
    async fn list_users_contains_created_user() {
        let uid = gen_uid("lst");
        if !seed_and_create_user(&uid, "List Test", "l@ex.com", "pw").await {
            return;
        }

        let users = list_users().await.unwrap();
        let found = users.iter().find(|u| u.uid.as_str() == uid.as_str());

        assert!(found.is_some(), "created uid should be listed");

        let found = found.unwrap();

        assert_eq!(found.name.as_str(), "List Test", "listed name should match");
        assert_eq!(
            found.email.as_str(),
            "l@ex.com",
            "listed email should match"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn list_users_sorted_by_uid() {
        let users = match list_users().await {
            Ok(u) => u,
            Err(AppError::Ldap(_)) => return, // LDAP indisponible => skip
            Err(e) => panic!("list_users should succeed, got {e}"),
        };

        let sorted = users
            .windows(2)
            .all(|w| w[0].uid.as_str() <= w[1].uid.as_str());

        assert!(sorted, "list_users should return users sorted by uid");
    }
}
