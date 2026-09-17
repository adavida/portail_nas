use ldap3::{LdapConnAsync, Scope, SearchEntry};

async fn purge_ou(ou: &str) {
    let url = std::env::var("LDAP_TEST_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3891".into());
    let base =
        std::env::var("LDAP_TEST_BASE_DN").unwrap_or_else(|_| "dc=test,dc=example,dc=com".into());
    let search_base = format!("ou={ou},{base}");

    let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await else {
        return;
    };
    ldap3::drive!(conn);

    let bind_dn = format!("cn=admin,{base}");
    if ldap.simple_bind(&bind_dn, "admin").await.is_err() {
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

#[ctor::ctor(unsafe)]
fn redirect_ldap_env_to_test() {
    std::env::set_var(
        "LDAP_URL",
        std::env::var("LDAP_TEST_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3891".into()),
    );
    std::env::set_var(
        "LDAP_BASE_DN",
        std::env::var("LDAP_TEST_BASE_DN").unwrap_or_else(|_| "dc=test,dc=example,dc=com".into()),
    );
}

#[ctor::ctor(unsafe)]
fn purge_test_ldap() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime");

    rt.block_on(async {
        purge_ou("people").await;
        purge_ou("groups").await;
    });
}

#[ctor::ctor(unsafe)]
fn purge_test_ldap_after() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime");

    rt.block_on(async {
        purge_ou("people").await;
        purge_ou("groups").await;
    });
}
