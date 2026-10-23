use ldap3::LdapConnAsync;

pub async fn ensure_clean(uid_str: &str) {
    let url = std::env::var("LDAP_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3891".into());
    let base = std::env::var("LDAP_BASE_DN").unwrap_or_else(|_| "dc=dev,dc=example,dc=com".into());
    if let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await {
        ldap3::drive!(conn);
        let dn = format!("uid={uid_str},ou=people,{base}");
        let _ = ldap.simple_bind(&format!("cn=admin,{base}"), "admin").await;
        let _ = ldap.delete(&dn).await;
        let _ = ldap.unbind().await;
    }
}
