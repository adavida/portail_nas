use ldap3::LdapConnAsync;

use crate::{
    domain::users::Uid,
    repository::ldap::{ldap_base, ldap_url, user_dn},
};

pub(crate) async fn ensure_clean(uid: &Uid) {
    if let Ok((conn, mut ldap)) = LdapConnAsync::new(ldap_url()).await {
        ldap3::drive!(conn);
        let bind_dn = format!("cn=admin,{}", ldap_base());
        if ldap.simple_bind(&bind_dn, "admin").await.is_ok() {
            let dn = user_dn(uid);
            let _ = ldap.delete(&dn).await;
            let _ = ldap.unbind().await;
        }
    }
}
