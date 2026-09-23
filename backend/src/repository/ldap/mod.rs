pub mod groups;
pub mod users;

pub use groups::{
    add_member, create_group, delete_group, list_groups, remove_member, update_group,
};
pub use users::{
    authenticate_user, create_user, delete_user, list_users, update_user, update_user_password,
};

use crate::error::AppError;

pub(crate) fn ldap_url() -> String {
    crate::env::Env::global().ldap_url.clone()
}

pub(crate) fn ldap_base() -> String {
    crate::env::Env::global().ldap_base_dn.clone()
}

pub(crate) async fn connect_admin(base: &str) -> Result<ldap3::Ldap, AppError> {
    let (conn, mut ldap) = ldap3::LdapConnAsync::new(&ldap_url()).await.map_ldap()?;

    ldap3::drive!(conn);
    bind_admin(&mut ldap, base).await?;
    Ok(ldap)
}

pub(crate) async fn bind_admin(ldap: &mut ldap3::Ldap, base: &str) -> Result<(), AppError> {
    let bind_dn = format!("cn=admin,{base}");
    let pw = crate::env::Env::global().ldap_admin_pw.clone();

    ldap.simple_bind(&bind_dn, &pw)
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    Ok(())
}

pub(crate) fn user_dn(uid: &crate::domain::users::Uid, base: &str) -> String {
    user_dn_from(uid.as_str(), base)
}

pub(crate) fn user_dn_from(uid: &str, base: &str) -> String {
    format!(
        "uid={},ou={},{}",
        uid,
        crate::env::Env::global().ldap_people_ou,
        base
    )
}

pub(crate) fn group_dn_from(gid: &str, base: &str) -> String {
    format!(
        "cn={},ou={},{}",
        gid,
        crate::env::Env::global().ldap_groups_ou,
        base
    )
}

pub(crate) fn people_search_base(base: &str) -> String {
    format!("ou={},{}", crate::env::Env::global().ldap_people_ou, base)
}

pub(crate) fn groups_search_base(base: &str) -> String {
    format!("ou={},{}", crate::env::Env::global().ldap_groups_ou, base)
}

pub(crate) trait MapLdap<T> {
    fn map_ldap(self) -> Result<T, AppError>;
}

impl<T> MapLdap<T> for Result<T, ldap3::LdapError> {
    fn map_ldap(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::Ldap(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    #[ctor::ctor(unsafe)]
    fn redirect_ldap_to_test() {
        crate::env::Env::ensure_init();
        std::env::set_var("LDAP_URL", crate::env::ldap_test_url());
        std::env::set_var("LDAP_BASE_DN", crate::env::ldap_test_base_dn());
    }

    #[test]
    fn ldap_url_reads_env() {
        // Env::create is called once; ldap_url reflects the global Env, not a
        // dynamic env var after init.
        let url = super::ldap_url();
        assert!(!url.is_empty(), "ldap_url should be non-empty");
        assert!(
            url.starts_with("ldap://"),
            "ldap_url should be ldap://..., got {url}"
        );
    }
}
