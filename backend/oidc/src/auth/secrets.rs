use exacl::{AclEntry, Perm, from_mode, setfacl};
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::auth::store;
use crate::domain::oidc::{Client, ClientSecret};
use portail_core::config::Config;

pub const KEY_FILE: &str = "signing-key.pem";

pub fn key_path(config: &Config) -> PathBuf {
    Path::new(&config.oidc_secrets_dir).join(KEY_FILE)
}

pub fn secret_path(config: &Config, name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    Path::new(&config.oidc_secrets_dir).join(path)
}

pub fn clients(config: &Config) -> Result<Vec<Client>, String> {
    Client::from_entries(&config.oidc_clients).map_err(|e| e.to_string())
}

#[derive(Debug, serde::Deserialize)]
struct ClientOwner {
    user: String,
    group: String,
}

pub fn ensure(config: &Config) -> Result<(), String> {
    let dir = &config.oidc_secrets_dir;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("cannot create OIDC secrets dir {dir}: {e}"))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("cannot set mode on OIDC secrets dir {dir}: {e}"))?;
    write_new(config, &key_path(config), None, || {
        generate_signing_key(config)
    })?;

    let owners = parse_owners(config)?;

    for client in clients(config)? {
        tracing::info!("client {}", client.id.as_str());
        let Some(name) = &client.secret_file else {
            continue;
        };
        let path = secret_path(config, name);
        let owner = owners.get(client.id.as_str());
        write_new(config, &path, owner, store::random_id)?;
        if let Some(owner) = owner {
            grant_dir_traverse(config, dir, owner)?;
        }
    }
    Ok(())
}

fn grant_dir_traverse(config: &Config, dir: &str, owner: &ClientOwner) -> Result<(), String> {
    let mut entries = from_mode(0o700);
    entries.push(AclEntry::allow_user(
        owner.user.as_str(),
        Perm::EXECUTE,
        None,
    ));
    entries.push(AclEntry::allow_group(
        owner.group.as_str(),
        Perm::EXECUTE,
        None,
    ));

    setfacl(&[Path::new(dir)], &entries, None)
        .map_err(|e| format!("cannot set traverse ACL on {dir}: {e}"))?;
    if config.oidc_secret_logs {
        tracing::info!(
            "OIDC secrets dir traversal ACL granted for {} {}",
            owner.user,
            owner.group
        );
    }
    Ok(())
}

fn parse_owners(config: &Config) -> Result<BTreeMap<String, ClientOwner>, String> {
    if config.oidc_client_owners.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(&config.oidc_client_owners)
        .map_err(|e| format!("cannot parse oidc_client_owners: {e}"))
}

fn grant_owner_read(config: &Config, path: &Path, owner: &ClientOwner) -> Result<(), String> {
    ensure_owner_exists("user", &owner.user)?;
    ensure_owner_exists("group", &owner.group)?;
    let mode = std::fs::metadata(path)
        .map_err(|e| format!("cannot stat {}: {e}", path.display()))?
        .permissions()
        .mode()
        & 0o777;
    let mut entries = from_mode(mode);
    entries.push(AclEntry::allow_user(&owner.user, Perm::READ, None));
    entries.push(AclEntry::allow_group(&owner.group, Perm::READ, None));
    setfacl(&[path], &entries, None)
        .map_err(|e| format!("cannot set ACL on {}: {e}", path.display()))?;
    if config.oidc_secret_logs {
        tracing::info!(
            "OIDC secret ACL read granted {}:{} on {}",
            owner.user,
            owner.group,
            path.display()
        );
    }
    Ok(())
}

fn ensure_owner_exists(kind: &str, name: &str) -> Result<(), String> {
    let c_name = CString::new(name).map_err(|_| format!("invalid owner {kind} name {name}"))?;
    let found = if kind == "user" {
        !unsafe { libc::getpwnam(c_name.as_ptr()) }.is_null()
    } else {
        !unsafe { libc::getgrnam(c_name.as_ptr()) }.is_null()
    };
    if !found {
        return Err(format!("unknown OIDC secret owner {kind} {name}"));
    }
    Ok(())
}

pub(crate) fn stored_secret(
    config: &Config,
    client: &Client,
) -> Result<Option<ClientSecret>, String> {
    let Some(name) = &client.secret_file else {
        return Ok(None);
    };
    let path = secret_path(config, name);
    std::fs::read_to_string(&path)
        .map(ClientSecret::new)
        .map(Some)
        .map_err(|e| format!("cannot read client secret file {}: {e}", path.display()))
}

pub(crate) fn read_key(config: &Config) -> Result<String, String> {
    std::fs::read_to_string(key_path(config)).map_err(|e| {
        format!(
            "cannot read signing key {}: {e}",
            key_path(config).display()
        )
    })
}

fn generate_signing_key(config: &Config) -> Result<String, String> {
    let key = RsaPrivateKey::new(&mut rand::rngs::OsRng, config.oidc_key_bits).map_err(|e| {
        format!(
            "cannot generate {}-bit RSA signing key: {e}",
            config.oidc_key_bits
        )
    })?;
    key.to_pkcs8_pem(LineEnding::LF)
        .map(|pem| pem.as_str().to_string())
        .map_err(|e| format!("cannot encode RSA signing key: {e}"))
}

fn write_new(
    config: &Config,
    path: &Path,
    owner: Option<&ClientOwner>,
    content: impl FnOnce() -> Result<String, String>,
) -> Result<(), String> {
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if std::fs::metadata(path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
            {
                if config.oidc_secret_logs {
                    tracing::info!(
                        "OIDC secret file already exists, kept as-is: {}",
                        path.display()
                    );
                }
                if let Some(owner) = owner {
                    grant_owner_read(config, path, owner)?;
                }
                return Ok(());
            }
            std::fs::remove_file(path)
                .map_err(|e| format!("cannot remove empty secret {}: {e}", path.display()))?;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|e| format!("cannot recreate {}: {e}", path.display()))?
        }
        Err(e) => return Err(format!("cannot create {}: {e}", path.display())),
    };
                         tracing::info!("ici: 1");

    let content = match content() {
        Ok(c) => c,
        Err(e) => {
            drop(file);
            let _ = std::fs::remove_file(path);
            return Err(e);
        }
    };
                             tracing::info!("ici: 2");
    file.write_all(content.as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                             tracing::info!("ici: 3");

    drop(file);
    if let Some(owner) = owner {
        grant_owner_read(config, path, owner)?;
    }
    if config.oidc_secret_logs {
        tracing::info!("generated OIDC secret file: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use exacl::getfacl;
    use portail_core::config::ClientEntry;

    fn test_env(tag: &str) -> Config {
        let mut config = Config::global().clone();
        let dir = format!(
            "{}/portail-secrets-test-{}-{}",
            std::env::temp_dir().display(),
            tag,
            std::process::id()
        );
        config.oidc_secrets_dir = dir.clone();
        config
    }

    fn current_owner() -> (String, String) {
        let user = unsafe {
            let passwd = libc::getpwuid(libc::geteuid());
            assert!(!passwd.is_null(), "euid should resolve to a user");
            std::ffi::CStr::from_ptr((*passwd).pw_name)
                .to_string_lossy()
                .into_owned()
        };
        let group = unsafe {
            let grp = libc::getgrgid(libc::getegid());
            assert!(!grp.is_null(), "egid should resolve to a group");
            std::ffi::CStr::from_ptr((*grp).gr_name)
                .to_string_lossy()
                .into_owned()
        };
        (user, group)
    }

    fn entry(id: &str, secret_file: Option<&str>) -> ClientEntry {
        ClientEntry {
            id: id.into(),
            secret_file: secret_file.map(Into::into),
            enabled: true,
            redirect_uris: vec!["http://localhost/cb".into()],
            scopes: vec![],
        }
    }

    fn confidential_clients(config: &Config, ids: &[&str]) -> Config {
        let mut config = config.clone();
        config.oidc_clients = ids
            .iter()
            .map(|id| entry(id, Some(&format!("{id}_secret"))))
            .collect();
        config
    }

    fn secret_file(config: &Config, id: &str) -> PathBuf {
        secret_path(config, &format!("{id}_secret"))
    }

    #[test]
    fn ensure_generates_missing_secret_file_at_0600() {
        let config = confidential_clients(&test_env("gen-secret"), &["app1"]);

        ensure(&config).unwrap();

        let secret = secret_file(&config, "app1");
        let generated = std::fs::read_to_string(&secret).unwrap();
        let mode = std::fs::metadata(&secret).unwrap().permissions().mode() & 0o777;
        assert!(
            !generated.trim().is_empty(),
            "generated secret must be non-empty"
        );
        assert_eq!(mode, 0o600, "generated secret mode must be 0600");
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_keeps_existing_secret_file() {
        let config = confidential_clients(&test_env("keep-secret"), &["app1"]);
        std::fs::create_dir_all(secret_file(&config, "app1").parent().unwrap()).unwrap();
        std::fs::write(secret_file(&config, "app1"), "admin-placed\n").unwrap();

        ensure(&config).unwrap();

        let content = std::fs::read_to_string(secret_file(&config, "app1")).unwrap();
        assert_eq!(
            content, "admin-placed\n",
            "existing secret must be kept as-is"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_regenerates_empty_secret_file() {
        let config = confidential_clients(&test_env("empty-secret"), &["app1"]);
        std::fs::create_dir_all(secret_file(&config, "app1").parent().unwrap()).unwrap();
        std::fs::write(secret_file(&config, "app1"), "").unwrap();

        ensure(&config).unwrap();

        let generated = std::fs::read_to_string(secret_file(&config, "app1")).unwrap();
        assert!(
            !generated.trim().is_empty(),
            "an empty secret file must be regenerated, not kept as-is"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn clients_maps_inline_entries() {
        let config = confidential_clients(&test_env("inline"), &["app1"]);

        let clients = clients(&config).unwrap();

        assert_eq!(clients.len(), 1, "one inline client expected");
        assert_eq!(clients[0].id.as_str(), "app1");
        assert!(clients[0].enabled, "inline enabled flag expected");
    }

    #[test]
    fn clients_rejects_invalid_entry() {
        let mut config = test_env("invalid");
        config.oidc_clients = vec![ClientEntry {
            id: "app1".into(),
            secret_file: None,
            enabled: true,
            redirect_uris: vec![],
            scopes: vec![],
        }];

        let err = clients(&config).unwrap_err();

        assert!(
            err.contains("redirect"),
            "client without redirects should be rejected: {err}"
        );
    }

    #[test]
    fn stored_secret_reads_and_trims_file() {
        let config = confidential_clients(&test_env("stored"), &["app1"]);
        std::fs::create_dir_all(secret_file(&config, "app1").parent().unwrap()).unwrap();
        std::fs::write(secret_file(&config, "app1"), "  s3cr3t\n").unwrap();
        let clients = clients(&config).unwrap();

        let secret = stored_secret(&config, &clients[0]).unwrap();

        assert_eq!(
            secret.as_ref().map(|s| s.as_str()),
            Some("s3cr3t"),
            "secret must be read from the file, whitespace trimmed"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn stored_secret_is_none_for_public_client() {
        let mut config = test_env("public");
        config.oidc_clients = vec![entry("pub1", None)];
        let clients = clients(&config).unwrap();

        let secret = stored_secret(&config, &clients[0]).unwrap();

        assert_eq!(secret, None, "public client has no secret file");
    }

    #[test]
    fn generated_key_is_pkcs8_pem_loadable() {
        let config = test_env("roundtrip");

        ensure(&config).unwrap();
        let pem = read_key(&config).unwrap();
        let key = crate::auth::key::SigningKey::from_pem(&pem).unwrap();

        assert!(
            pem.contains("BEGIN PRIVATE KEY"),
            "PKCS#8 PEM expected: {pem}"
        );
        assert!(!key.n.is_empty(), "modulus should be non-empty");
        assert!(!key.e.is_empty(), "exponent should be non-empty");
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_grants_declared_owner_read_acl() {
        let (user, group) = current_owner();
        let mut config = confidential_clients(&test_env("acl"), &["app1"]);
        config.oidc_client_owners =
            format!(r#"{{"app1": {{"user": "{user}", "group": "{group}"}}}}"#);

        ensure(&config).unwrap();

        let entries = getfacl(secret_file(&config, "app1"), None).unwrap();
        let has_user = entries
            .iter()
            .any(|e| e.name == user && e.perms.contains(Perm::READ));
        let has_group = entries
            .iter()
            .any(|e| e.name == group && e.perms.contains(Perm::READ));
        assert!(has_user, "declared user must have ACL read: {entries:?}");
        assert!(has_group, "declared group must have ACL read: {entries:?}");
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_grants_owner_execute_acl_on_secrets_dir() {
        let (user, group) = current_owner();
        let mut config = confidential_clients(&test_env("dir-acl"), &["app1"]);
        config.oidc_client_owners =
            format!(r#"{{"app1": {{"user": "{user}", "group": "{group}"}}}}"#);

        ensure(&config).unwrap();

        let entries = getfacl(Path::new(&config.oidc_secrets_dir), None).unwrap();
        let has_user = entries
            .iter()
            .any(|e| e.name == user && e.perms.contains(Perm::EXECUTE));
        let has_group = entries
            .iter()
            .any(|e| e.name == group && e.perms.contains(Perm::EXECUTE));
        assert!(
            has_user,
            "owner needs dir traverse to reach its secret: {entries:?}"
        );
        assert!(has_group, "owner group needs dir traverse: {entries:?}");
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_skips_acl_for_absent_declared_secret() {
        let mut config = test_env("acl-absent");
        config.oidc_client_owners =
            r#"{"app2": {"user": "no-such-user-zz", "group": "no-such-group-zz"}}"#.to_string();

        let result = ensure(&config);

        assert!(
            result.is_ok(),
            "owner declared for a client absent from the entries should be skipped: {result:?}"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_rejects_unknown_owner_of_existing_secret() {
        let mut config = confidential_clients(&test_env("acl-unknown"), &["app3"]);
        config.oidc_client_owners =
            r#"{"app3": {"user": "no-such-user-zz", "group": "portail-no-such-zz"}}"#.to_string();

        let err = ensure(&config).unwrap_err();

        assert!(
            err.contains("unknown OIDC secret owner user"),
            "unknown user should be reported: {err}"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn ensure_rejects_unparsable_owners() {
        let mut config = test_env("owners-bad");
        config.oidc_client_owners = "not-json".to_string();

        let err = ensure(&config).unwrap_err();

        assert!(
            err.contains("cannot parse oidc_client_owners"),
            "invalid JSON should be reported: {err}"
        );
    }

    #[test]
    fn secret_path_resolves_relative_in_secrets_dir_and_absolute_as_is() {
        let config = test_env("resolve");

        let relative = secret_path(&config, "app1_secret");
        let absolute = secret_path(&config, "/run/secrets/app2");

        assert_eq!(
            relative,
            PathBuf::from(&config.oidc_secrets_dir).join("app1_secret"),
            "relative secret_file belongs in the secrets dir"
        );
        assert_eq!(
            absolute,
            PathBuf::from("/run/secrets/app2"),
            "absolute secret_file must be used as-is"
        );
    }
}
