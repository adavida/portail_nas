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

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use portail_backend::auth::{Claims, middleware::AuthUser};

#[allow(dead_code)]
async fn inject_admin(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = Claims {
        sub: "admin".into(),
        aud: serde_json::json!(env!("OIDC_CLIENT_ID")),
        iss: env!("OIDC_ISSUER_URL").to_string(),
        exp: 9999999999,
        groups: vec!["admin".into()],
        email: Some("admin@example.com".into()),
        preferred_username: Some("admin".into()),
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}

#[allow(dead_code)]
async fn inject_user(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let claims = Claims {
        sub: "user".into(),
        aud: serde_json::json!(env!("OIDC_CLIENT_ID")),
        iss: env!("OIDC_ISSUER_URL").to_string(),
        exp: 9999999999,
        groups: vec![],
        email: Some("user@example.com".into()),
        preferred_username: Some("user".into()),
    };
    req.extensions_mut().insert(AuthUser(claims));
    Ok(next.run(req).await)
}

#[allow(dead_code)]
pub fn test_app() -> axum::Router {
    use axum::middleware;
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_admin))
}

#[allow(dead_code)]
pub fn test_app_as_user() -> axum::Router {
    use axum::middleware;
    let prod = portail_backend::app();
    prod.layer(middleware::from_fn(inject_user))
}
