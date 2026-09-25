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

/// Adds a trusted root certificate to the connection settings when
/// `LDAP_TLS_CA` points to a PEM file (self-signed server cert or CA).
/// Empty path = system trust store only.
pub(crate) fn with_tls_ca(
    settings: ldap3::LdapConnSettings,
    ca_path: &str,
) -> Result<ldap3::LdapConnSettings, AppError> {
    let path = ca_path.trim();
    if path.is_empty() {
        return Ok(settings);
    }
    let pem = std::fs::read(path)
        .map_err(|e| AppError::Ldap(format!("cannot read LDAP_TLS_CA {path}: {e}")))?;
    let cert = native_tls::Certificate::from_pem(&pem)
        .map_err(|e| AppError::Ldap(format!("invalid LDAP_TLS_CA PEM {path}: {e}")))?;
    let connector = native_tls::TlsConnector::builder()
        .add_root_certificate(cert)
        .build()
        .map_err(|e| AppError::Ldap(format!("tls connector error: {e}")))?;
    Ok(settings.set_connector(connector))
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn connect() -> Result<ldap3::Ldap, AppError> {
    let env = crate::env::Env::global();
    let settings = with_tls_ca(ldap3::LdapConnSettings::new(), &env.ldap_tls_ca)?;
    let (conn, mut ldap) = ldap3::LdapConnAsync::with_settings(settings, &ldap_url())
        .await
        .map_ldap()?;
    ldap3::drive!(conn);
    bind_admin(&mut ldap).await?;
    Ok(ldap)
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn bind_admin(ldap: &mut ldap3::Ldap) -> Result<(), AppError> {
    let bind_dn = format!("cn=admin,{}", ldap_base());
    tracing::info!(bind_dn = %bind_dn, url = %ldap_url(), "ldap admin bind");
    let pw = crate::env::Env::global().ldap_admin_pw.clone();

    ldap.simple_bind(&bind_dn, &pw)
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    Ok(())
}

pub(crate) fn user_dn(uid: &crate::domain::users::Uid) -> String {
    user_dn_from(uid.as_str())
}

pub(crate) fn user_dn_from(uid: &str) -> String {
    format!(
        "uid={},ou={},{}",
        uid,
        crate::env::Env::global().ldap_people_ou,
        ldap_base()
    )
}

pub(crate) fn group_dn_from(gid: &str) -> String {
    format!(
        "cn={},ou={},{}",
        gid,
        crate::env::Env::global().ldap_groups_ou,
        ldap_base()
    )
}

pub(crate) fn people_search_base() -> String {
    format!(
        "ou={},{}",
        crate::env::Env::global().ldap_people_ou,
        ldap_base()
    )
}

pub(crate) fn groups_search_base() -> String {
    format!(
        "ou={},{}",
        crate::env::Env::global().ldap_groups_ou,
        ldap_base()
    )
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
            url.starts_with("ldap"),
            "ldap_url should be ldap:// or ldaps://..., got {url}"
        );
    }

    #[test]
    fn tls_ca_empty_is_noop() {
        let settings = ldap3::LdapConnSettings::new();

        let result = super::with_tls_ca(settings, "");

        assert!(result.is_ok(), "empty CA path should be a no-op");
    }

    #[test]
    fn tls_ca_missing_file_fails() {
        let settings = ldap3::LdapConnSettings::new();

        let err = match super::with_tls_ca(settings, "/tmp/portail-no-such-ca.crt") {
            Err(err) => err,
            Ok(_) => panic!("missing CA file should fail"),
        };

        assert!(
            err.to_string().contains("cannot read LDAP_TLS_CA"),
            "missing CA file should be reported: {err}"
        );
    }

    #[test]
    fn tls_ca_invalid_pem_fails() {
        let path = std::env::temp_dir().join(format!("portail-bogus-ca-{}", std::process::id()));
        std::fs::write(&path, "not a pem").unwrap();

        let err = match super::with_tls_ca(ldap3::LdapConnSettings::new(), path.to_str().unwrap()) {
            Err(err) => err,
            Ok(_) => panic!("garbage PEM should fail"),
        };

        assert!(
            err.to_string().contains("invalid LDAP_TLS_CA PEM"),
            "garbage PEM should be reported: {err}"
        );
    }
}
