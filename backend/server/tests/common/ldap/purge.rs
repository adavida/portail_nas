use ldap3::{LdapConnAsync, Scope, SearchEntry};
use portail_core::config::Config;

async fn purge_ou(ou: &str) {
    let config = Config::global();
    let url = config.ldap.url.clone();
    let base = config.ldap.base_dn.clone();
    let admin_pw = config.ldap.admin_pw.clone();
    let search_base = format!("ou={ou},{base}");

    let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await else {
        return;
    };
    ldap3::drive!(conn);

    let bind_dn = format!("cn=admin,{base}");
    if ldap.simple_bind(&bind_dn, &admin_pw).await.is_err() {
        return;
    }

    if let Ok(entries) = ldap
        .search(
            &search_base,
            Scope::OneLevel,
            "(objectClass=*)",
            vec!["cn", "uid"],
        )
        .await
        .and_then(|r| r.success())
    {
        for entry in entries.0 {
            let dn = SearchEntry::construct(entry).dn;
            let _ = ldap.delete(&dn).await;
        }
    }
    let _ = ldap.unbind().await;
}

pub(crate) fn purge_test_ldap() {
    std::thread::spawn(|| {
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };

        rt.block_on(async {
            purge_ou("people").await;
            purge_ou("groups").await;
        });
    })
    .join()
    .ok();
}
