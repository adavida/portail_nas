use ldap3::{LdapConnAsync, Scope, SearchEntry};

use crate::{domain::users::User, error::AppError};

fn ldap_url() -> String {
    if cfg!(test) {
        std::env::var("LDAP_TEST_URL")
            .or_else(|_| std::env::var("LDAP_URL"))
            .unwrap_or_else(|_| "ldap://127.0.0.1:3891".into())
    } else {
        std::env::var("LDAP_URL")
            .or_else(|_| std::env::var("LDAP_TEST_URL"))
            .unwrap_or_else(|_| "ldap://127.0.0.1:3890".into())
    }
}

fn ldap_base() -> String {
    if cfg!(test) {
        std::env::var("LDAP_TEST_BASE_DN")
            .or_else(|_| std::env::var("LDAP_BASE_DN"))
            .unwrap_or_else(|_| "dc=test,dc=example,dc=com".into())
    } else {
        std::env::var("LDAP_BASE_DN")
            .or_else(|_| std::env::var("LDAP_TEST_BASE_DN"))
            .unwrap_or_else(|_| "dc=dev,dc=example,dc=com".into())
    }
}

pub async fn list() -> Result<Vec<User>, AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let search_base = format!("ou=people,{base}");

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);

    let (rs, _res) = ldap
        .search(
            &search_base,
            Scope::Subtree,
            "(objectClass=inetOrgPerson)",
            vec!["uid", "cn", "displayName", "mail"],
        )
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;

    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();
    let users = User::from_search(entries);
    let _ = ldap.unbind().await;
    Ok(users)
}
