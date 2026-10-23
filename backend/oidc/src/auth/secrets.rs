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
use portail_core::env::Env;

pub const KEY_FILE: &str = "signing-key.pem";

pub fn clients_path(env: &Env) -> PathBuf {
    PathBuf::from(&env.oidc_clients_file)
}

pub fn key_path(env: &Env) -> PathBuf {
    Path::new(&env.oidc_secrets_dir).join(KEY_FILE)
}

/// Resolves a `secret_file` from the clients JSON: absolute paths are used
/// as-is (e.g. agenix/sops `secretFile` overrides), relative names live next
/// to the clients file.
pub fn secret_path(env: &Env, name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    clients_path(env)
        .parent()
        .map(|dir| dir.join(path))
        .unwrap_or_else(|| path.to_path_buf())
}

/// `{ "<clientId>": {"user": "...", "group": "..."} }` from
/// `OIDC_CLIENT_OWNERS` (empty = disabled). Only generated secret files are
/// concerned: `secretFile` overrides belong to their secret manager.
#[derive(Debug, serde::Deserialize)]
struct ClientOwner {
    user: String,
    group: String,
}

/// Ensures the OIDC secrets at startup. The clients file is **read-only** —
/// written by nix activation / devenv bootstrap, its absence is a startup
/// error. The RSA signing key (PKCS#8 PEM) and every confidential client's
/// `secret_file` are **generated on first startup** if missing (`O_EXCL`,
/// 0600, never overwritten). Declared Linux owners (`OIDC_CLIENT_OWNERS`) get
/// read access through a POSIX ACL, re-applied on every start. Idempotent.
/// Called once by `main` and by the test app builders.
pub fn ensure(env: &Env) -> Result<(), String> {
    let dir = &env.oidc_secrets_dir;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("cannot create OIDC secrets dir {dir}: {e}"))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("cannot set mode on OIDC secrets dir {dir}: {e}"))?;
    let clients = clients_path(env);
    if !clients.exists() {
        return Err(format!(
            "clients file missing: {} — it is written by nix, see README",
            clients.display()
        ));
    }
    write_new(env, &key_path(env), || generate_signing_key(env))?;

    let owners = parse_owners(env)?;
    let parsed = Client::parse_all(&read_clients(env)?).map_err(|e| e.to_string())?;
    for client in parsed {
        let Some(name) = &client.secret_file else {
            continue;
        };
        let path = secret_path(env, name);
        write_new(env, &path, store::random_id)?;
        if let Some(owner) = owners.get(client.id.as_str()) {
            grant_owner_read(env, &path, owner)?;
        }
    }
    Ok(())
}

fn parse_owners(env: &Env) -> Result<BTreeMap<String, ClientOwner>, String> {
    if env.oidc_client_owners.trim().is_empty() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(&env.oidc_client_owners)
        .map_err(|e| format!("cannot parse OIDC_CLIENT_OWNERS: {e}"))
}

/// Grants `user` and `group` read access to the secret file through a POSIX
/// ACL, keeping the file's mode bits as base entries. The service user owns
/// the generated files, so no capability is needed.
fn grant_owner_read(env: &Env, path: &Path, owner: &ClientOwner) -> Result<(), String> {
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
    if env.oidc_secret_logs {
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

/// The client's real secret, read from its file on every use — the clients
/// JSON never holds it. `None` = public client.
pub(crate) fn stored_secret(env: &Env, client: &Client) -> Result<Option<ClientSecret>, String> {
    let Some(name) = &client.secret_file else {
        return Ok(None);
    };
    let path = secret_path(env, name);
    std::fs::read_to_string(&path)
        .map(ClientSecret::new)
        .map(Some)
        .map_err(|e| format!("cannot read client secret file {}: {e}", path.display()))
}

pub(crate) fn read_clients(env: &Env) -> Result<String, String> {
    std::fs::read_to_string(clients_path(env)).map_err(|e| {
        format!(
            "cannot read clients file {}: {e}",
            clients_path(env).display()
        )
    })
}

pub(crate) fn read_key(env: &Env) -> Result<String, String> {
    std::fs::read_to_string(key_path(env))
        .map_err(|e| format!("cannot read signing key {}: {e}", key_path(env).display()))
}

fn generate_signing_key(env: &Env) -> Result<String, String> {
    let key = RsaPrivateKey::new(&mut rand::rngs::OsRng, env.oidc_key_bits).map_err(|e| {
        format!(
            "cannot generate {}-bit RSA signing key: {e}",
            env.oidc_key_bits
        )
    })?;
    key.to_pkcs8_pem(LineEnding::LF)
        .map(|pem| pem.as_str().to_string())
        .map_err(|e| format!("cannot encode RSA signing key: {e}"))
}

/// Atomic 0600 creation (`O_EXCL`): if the file already exists (placed by
/// the admin), it is kept as is and `content` is never called.
fn write_new(
    env: &Env,
    path: &Path,
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
            if env.oidc_secret_logs {
                tracing::info!(
                    "OIDC secret file already exists, kept as-is: {}",
                    path.display()
                );
            }
            return Ok(());
        }
        Err(e) => return Err(format!("cannot create {}: {e}", path.display())),
    };
    let content = match content() {
        Ok(c) => c,
        Err(e) => {
            drop(file);
            let _ = std::fs::remove_file(path);
            return Err(e);
        }
    };
    file.write_all(content.as_bytes())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    if env.oidc_secret_logs {
        tracing::info!("generated OIDC secret file: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use exacl::getfacl;

    fn test_env(tag: &str) -> Env {
        let mut env = portail_core::env::Env::global().clone();
        let dir = format!(
            "{}/portail-secrets-test-{}-{}",
            std::env::temp_dir().display(),
            tag,
            std::process::id()
        );
        env.oidc_secrets_dir = dir.clone();
        env.oidc_clients_file = format!("{dir}/clients.json");
        env
    }

    /// Nom login du user/groupe courant — owner valide pour les tests d'ACL
    /// (le proprietaire du fichier peut toujours poser son ACL sans capacite).
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

    fn write_clients(env: &Env, json: &str) {
        std::fs::create_dir_all(clients_path(env).parent().unwrap()).unwrap();
        std::fs::write(clients_path(env), json).unwrap();
    }

    fn secret_file(env: &Env, id: &str) -> PathBuf {
        secret_path(env, &format!("{id}_secret"))
    }

    fn confidential_clients(env: &Env, ids: &[&str]) {
        let entries: Vec<serde_json::Value> = ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "id": id,
                    "secret_file": format!("{id}_secret"),
                    "enabled": true,
                    "redirect_uris": ["http://localhost/cb"],
                })
            })
            .collect();
        write_clients(env, &serde_json::to_string(&entries).unwrap());
    }

    #[test]
    fn ensure_fails_when_clients_file_missing() {
        let env = test_env("missing-clients");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);

        let err = ensure(&env).unwrap_err();

        assert!(
            err.contains("clients file missing"),
            "absent clients JSON must be a startup error: {err}"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_never_rewrites_clients_file() {
        let env = test_env("readonly-json");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        confidential_clients(&env, &["app1"]);

        ensure(&env).unwrap();
        let first = std::fs::read_to_string(clients_path(&env)).unwrap();

        ensure(&env).unwrap();
        let second = std::fs::read_to_string(clients_path(&env)).unwrap();

        assert_eq!(
            first, second,
            "existing clients file must never be rewritten"
        );
        assert!(
            key_path(&env).exists(),
            "signing key must be generated in the secrets dir"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_generates_missing_secret_file_at_0600() {
        let env = test_env("gen-secret");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        confidential_clients(&env, &["app1"]);

        ensure(&env).unwrap();

        let secret = secret_file(&env, "app1");
        let generated = std::fs::read_to_string(&secret).unwrap();
        let mode = std::fs::metadata(&secret).unwrap().permissions().mode() & 0o777;
        assert!(
            !generated.trim().is_empty(),
            "generated secret must be non-empty"
        );
        assert_eq!(mode, 0o600, "generated secret mode must be 0600");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_keeps_existing_secret_file() {
        let env = test_env("keep-secret");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        confidential_clients(&env, &["app1"]);
        std::fs::create_dir_all(secret_file(&env, "app1").parent().unwrap()).unwrap();
        std::fs::write(secret_file(&env, "app1"), "admin-placed\n").unwrap();

        ensure(&env).unwrap();

        let content = std::fs::read_to_string(secret_file(&env, "app1")).unwrap();
        assert_eq!(
            content, "admin-placed\n",
            "existing secret must be kept as-is"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn stored_secret_reads_and_trims_file() {
        let env = test_env("stored");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        confidential_clients(&env, &["app1"]);
        std::fs::create_dir_all(secret_file(&env, "app1").parent().unwrap()).unwrap();
        std::fs::write(secret_file(&env, "app1"), "  s3cr3t\n").unwrap();
        let clients = Client::parse_all(&read_clients(&env).unwrap()).unwrap();

        let secret = stored_secret(&env, &clients[0]).unwrap();

        assert_eq!(
            secret.as_ref().map(|s| s.as_str()),
            Some("s3cr3t"),
            "secret must be read from the file, whitespace trimmed"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn stored_secret_is_none_for_public_client() {
        let env = test_env("public");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        write_clients(
            &env,
            r#"[{"id":"pub1","enabled":true,"redirect_uris":["http://localhost/cb"]}]"#,
        );
        let clients = Client::parse_all(&read_clients(&env).unwrap()).unwrap();

        let secret = stored_secret(&env, &clients[0]).unwrap();

        assert_eq!(secret, None, "public client has no secret file");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn generated_key_is_pkcs8_pem_loadable() {
        let env = test_env("roundtrip");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        write_clients(&env, "[]");

        ensure(&env).unwrap();
        let pem = read_key(&env).unwrap();
        let key = crate::auth::key::SigningKey::from_pem(&pem).unwrap();

        assert!(
            pem.contains("BEGIN PRIVATE KEY"),
            "PKCS#8 PEM expected: {pem}"
        );
        assert!(!key.n.is_empty(), "modulus should be non-empty");
        assert!(!key.e.is_empty(), "exponent should be non-empty");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_grants_declared_owner_read_acl() {
        let mut env = test_env("acl");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        let (user, group) = current_owner();
        env.oidc_client_owners = format!(r#"{{"app1": {{"user": "{user}", "group": "{group}"}}}}"#);
        confidential_clients(&env, &["app1"]);

        ensure(&env).unwrap();

        let entries = getfacl(secret_file(&env, "app1"), None).unwrap();
        let has_user = entries
            .iter()
            .any(|e| e.name == user && e.perms.contains(Perm::READ));
        let has_group = entries
            .iter()
            .any(|e| e.name == group && e.perms.contains(Perm::READ));
        assert!(has_user, "declared user must have ACL read: {entries:?}");
        assert!(has_group, "declared group must have ACL read: {entries:?}");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_skips_acl_for_absent_declared_secret() {
        let mut env = test_env("acl-absent");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        env.oidc_client_owners =
            r#"{"app2": {"user": "no-such-user-zz", "group": "no-such-group-zz"}}"#.to_string();
        write_clients(&env, "[]");

        let result = ensure(&env);

        assert!(
            result.is_ok(),
            "owner declared for a client absent from the JSON should be skipped: {result:?}"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_rejects_unknown_owner_of_existing_secret() {
        let mut env = test_env("acl-unknown");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        env.oidc_client_owners =
            r#"{"app3": {"user": "no-such-user-zz", "group": "portail-no-such-zz"}}"#.to_string();
        confidential_clients(&env, &["app3"]);

        let err = ensure(&env).unwrap_err();

        assert!(
            err.contains("unknown OIDC secret owner user"),
            "unknown user should be reported: {err}"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn ensure_rejects_unparsable_owners() {
        let mut env = test_env("owners-bad");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
        env.oidc_client_owners = "not-json".to_string();
        write_clients(&env, "[]");

        let err = ensure(&env).unwrap_err();

        assert!(
            err.contains("cannot parse OIDC_CLIENT_OWNERS"),
            "invalid JSON should be reported: {err}"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn secret_path_resolves_relative_next_to_clients_absolute_as_is() {
        let env = test_env("resolve");

        let relative = secret_path(&env, "app1_secret");
        let absolute = secret_path(&env, "/run/secrets/app2");

        assert_eq!(
            relative,
            clients_path(&env).parent().unwrap().join("app1_secret"),
            "relative secret_file belongs next to the clients file"
        );
        assert_eq!(
            absolute,
            PathBuf::from("/run/secrets/app2"),
            "absolute secret_file must be used as-is"
        );
    }
}
