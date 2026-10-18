use ldap3::{LdapConnAsync, Scope, SearchEntry};

use super::redirect_env::redirect_ldap_env_to_test;

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

// synchronous purge (ctor/dtor cannot fire-and-forget: a thread is killed
// at process exit before it finishes). Dedicated thread + join: at exit the
// main tokio TLSes are destroyed, a fresh thread leaves them intact.
fn purge_test_ldap() {
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

// Purge at test process startup: every run starts from scratch.
#[ctor::ctor(unsafe)]
fn purge_test_ldap_at_start() {
    redirect_ldap_env_to_test();
    purge_test_ldap();
}

// Purge at process shutdown: the test base is clean after the run
// (LDAP down => silent no-op).
#[dtor::dtor(unsafe)]
fn purge_test_ldap_at_exit() {
    purge_test_ldap();
}
