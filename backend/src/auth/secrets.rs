use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::auth::store;
use crate::env::Env;

pub const CLIENTS_FILE: &str = "clients.json";
pub const KEY_FILE: &str = "signing-key.pem";

pub fn clients_path(env: &Env) -> PathBuf {
    Path::new(&env.oidc_secrets_dir).join(CLIENTS_FILE)
}

pub fn key_path(env: &Env) -> PathBuf {
    Path::new(&env.oidc_secrets_dir).join(KEY_FILE)
}

/// Ensures the secrets directory: RSA key (PKCS#8 PEM) and default
/// clients.json (portal client with a random secret) are **generated on
/// first startup** if missing, never overwritten if they exist. Idempotent.
/// Called once by `main` and by the test app builders — reads never create.
pub fn ensure(env: &Env) -> Result<(), String> {
    let dir = &env.oidc_secrets_dir;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("cannot create OIDC secrets dir {dir}: {e}"))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("cannot set mode on OIDC secrets dir {dir}: {e}"))?;
    write_new(&key_path(env), || generate_signing_key(env))?;
    write_new(&clients_path(env), || default_clients(env))?;
    Ok(())
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

fn default_clients(env: &Env) -> Result<String, String> {
    let secret = store::random_id()?;
    let value = serde_json::json!([{
        "id": env.oidc_client_id,
        "secret": secret,
        "redirect_uris": [env.oidc_redirect_uri],
        "scopes": ["openid", "groups", "email", "profile"],
    }]);
    serde_json::to_string_pretty(&value).map_err(|e| format!("cannot build default clients: {e}"))
}

/// Atomic 0600 creation (`O_EXCL`): if the file already exists (placed by
/// the admin), it is kept as is and `content` is never called.
fn write_new(path: &Path, content: impl FnOnce() -> Result<String, String>) -> Result<(), String> {
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
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
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::oidc::Client;

    fn test_env(tag: &str) -> Env {
        let mut env = crate::env::Env::global().clone();
        env.oidc_secrets_dir = format!(
            "{}/portail-secrets-test-{}-{}",
            std::env::temp_dir().display(),
            tag,
            std::process::id()
        );
        env
    }

    #[test]
    fn ensure_generates_key_and_clients_once() {
        let env = test_env("gen");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);

        ensure(&env).unwrap();
        let first = std::fs::read_to_string(clients_path(&env)).unwrap();

        ensure(&env).unwrap();
        let second = std::fs::read_to_string(clients_path(&env)).unwrap();

        assert!(
            key_path(&env).exists(),
            "signing key must be generated in the secrets dir"
        );
        assert_eq!(
            first, second,
            "existing clients file must never be rewritten"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn default_clients_match_env_and_are_valid() {
        let env = test_env("default");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);

        ensure(&env).unwrap();
        let clients = Client::parse_all(&read_clients(&env).unwrap()).unwrap();

        let client = Client::find_in(&clients, &env.oidc_client_id)
            .expect("the portal client must be declared in the generated file");
        assert!(
            !client.secret.as_str().is_empty(),
            "generated secret must be non-empty"
        );
        assert!(
            client.allows_redirect(&env.oidc_redirect_uri),
            "the env redirect_uri must be registered"
        );
        assert!(
            client.grants("groups"),
            "default scopes must include groups"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn generated_key_is_pkcs8_pem_loadable() {
        let env = test_env("roundtrip");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);

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
}
