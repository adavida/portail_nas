use ldap3::LdapConnAsync;
use portail_core::config::Config;

pub async fn ensure_clean(uid_str: &str) {
    let config = Config::global();
    let url = config.ldap_url.clone();
    let base = config.ldap_base_dn.clone();
    let admin_pw = config.ldap_admin_pw.clone();
    if let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await {
        ldap3::drive!(conn);
        let dn = format!("uid={uid_str},ou=people,{base}");
        let _ = ldap
            .simple_bind(&format!("cn=admin,{base}"), &admin_pw)
            .await;
        let _ = ldap.delete(&dn).await;
        let _ = ldap.unbind().await;
    }
}
