pub mod groups;
pub mod users;

pub use groups::{create_group, delete_group, list_groups};
pub use users::{
    authenticate_user, create_user, delete_user, list_users, update_user, update_user_password,
};

use crate::error::AppError;

pub(crate) fn ldap_url() -> String {
    std::env::var("LDAP_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3890".into())
}

pub(crate) fn ldap_base() -> String {
    std::env::var("LDAP_BASE_DN").unwrap_or_else(|_| "dc=dev,dc=example,dc=com".into())
}

#[cfg(test)]
mod tests {
    use super::ldap_url;

    #[ctor::ctor(unsafe)]
    fn redirect_ldap_to_test() {
        std::env::set_var(
            "LDAP_URL",
            std::env::var("LDAP_TEST_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3891".into()),
        );
        std::env::set_var(
            "LDAP_BASE_DN",
            std::env::var("LDAP_TEST_BASE_DN")
                .unwrap_or_else(|_| "dc=test,dc=example,dc=com".into()),
        );
    }

    #[test]
    fn ldap_url_reads_env() {
        std::env::set_var("LDAP_URL", "ldap://example:1");
        assert_eq!(ldap_url(), "ldap://example:1");
        std::env::remove_var("LDAP_URL");
        assert_eq!(ldap_url(), "ldap://127.0.0.1:3890");
    }
}

pub(crate) async fn connect_admin(base: &str) -> Result<ldap3::Ldap, AppError> {
    let (conn, mut ldap) = ldap3::LdapConnAsync::new(&ldap_url()).await.map_ldap()?;

    ldap3::drive!(conn);
    bind_admin(&mut ldap, base).await?;
    Ok(ldap)
}

pub(crate) async fn bind_admin(ldap: &mut ldap3::Ldap, base: &str) -> Result<(), AppError> {
    let bind_dn = format!("cn=admin,{base}");

    ldap.simple_bind(&bind_dn, "admin")
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    Ok(())
}

pub(crate) fn user_dn(uid: &crate::domain::users::Uid, base: &str) -> String {
    format!("uid={},ou=people,{base}", uid.as_str())
}

pub(crate) trait MapLdap<T> {
    fn map_ldap(self) -> Result<T, AppError>;
}

impl<T> MapLdap<T> for Result<T, ldap3::LdapError> {
    fn map_ldap(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::Ldap(e.to_string()))
    }
}
