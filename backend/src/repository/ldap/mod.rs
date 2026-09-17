pub mod users;

pub use users::{authenticate_user, create_user, delete_user, list_users, update_user_password};

use crate::error::AppError;

pub(crate) fn ldap_url() -> String {
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

pub(crate) fn ldap_base() -> String {
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

pub(crate) async fn bind_admin(ldap: &mut ldap3::Ldap, base: &str) -> Result<(), AppError> {
    let bind_dn = format!("cn=admin,{base}");
    ldap.simple_bind(&bind_dn, "admin")
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    Ok(())
}
