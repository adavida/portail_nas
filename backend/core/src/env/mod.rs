pub mod errors;
pub use errors::EnvError;

use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Env {
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_redirect_uri: String,
    pub oidc_secrets_dir: String,
    /// Full path of the OIDC clients file (`OIDC_CLIENTS_FILE`). Written by
    /// nix only — the backend reads it and never modifies it.
    pub oidc_clients_file: String,
    pub oidc_key_bits: usize,
    /// INFO logging of OIDC secret file events (`OIDC_SECRET_LOGS`).
    /// `false` = silent; errors stay logged either way.
    pub oidc_secret_logs: bool,
    pub app_url: String,
    pub bind_addr: String,
    pub ldap_url: String,
    pub ldap_base_dn: String,
    pub ldap_people_ou: String,
    pub ldap_groups_ou: String,
    pub ldap_admin_pw: String,
    /// Optional TLS trust for LDAPS: PEM file path of the CA / self-signed
    /// server cert. Empty (default) = system trust store only.
    pub ldap_tls_ca: String,
    /// JSON map `{ "<clientId>": {"user": "...", "group": "..."} }`
    /// (`OIDC_CLIENT_OWNERS`) declaring the Linux owner of each generated
    /// `<clientId>_secret`. Empty (default) = no ownership enforcement; the
    /// files stay owned by the service user.
    pub oidc_client_owners: String,
}

static ENV: OnceLock<Env> = OnceLock::new();

pub const DEFAULT_KEY_BITS: usize = 2048;

/// Secret vars (`LDAP_ADMIN_PW`) hold the **path** of a file whose content is
/// the secret. Trims whitespace/newline around the content.
fn read_secret(name: &'static str) -> Option<Result<String, String>> {
    let path = std::env::var(name).ok()?;
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    Some(
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("{name}: cannot read secret file {path}: {e}"))
            .and_then(|s| {
                if s.is_empty() {
                    Err(format!("{name}: secret file {path} is empty"))
                } else {
                    Ok(s)
                }
            }),
    )
}

impl Env {
    /// Reads all required vars from `std::env`; collects every missing/empty
    /// name and returns a single error listing them all.
    /// Secret vars contain the path of a file whose content is the secret.
    /// Optional vars fall back to documented defaults (see field docs).
    pub fn create() -> Result<Self, EnvError> {
        let mut missing: Vec<&'static str> = Vec::new();
        let mut file_errors: Vec<String> = Vec::new();
        let mut invalid: Vec<String> = Vec::new();

        let take = |name: &'static str, missing: &mut Vec<&'static str>| -> String {
            match std::env::var(name) {
                Ok(v) if !v.trim().is_empty() => v,
                _ => {
                    missing.push(name);
                    String::new()
                }
            }
        };

        let take_opt = |name: &'static str, default: &str| -> String {
            std::env::var(name)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| default.to_string())
        };

        let take_secret_opt =
            |name: &'static str, default: &str, file_errors: &mut Vec<String>| -> String {
                match read_secret(name) {
                    Some(Ok(v)) => v,
                    Some(Err(e)) => {
                        file_errors.push(e);
                        String::new()
                    }
                    None => default.to_string(),
                }
            };

        let oidc_issuer_url = take("OIDC_ISSUER_URL", &mut missing);
        let oidc_client_id = take("OIDC_CLIENT_ID", &mut missing);
        let oidc_secrets_dir = take("OIDC_SECRETS_DIR", &mut missing);
        let oidc_clients_file = take("OIDC_CLIENTS_FILE", &mut missing);
        let oidc_key_bits =
            match take_opt("OIDC_KEY_BITS", &DEFAULT_KEY_BITS.to_string()).parse::<usize>() {
                Ok(n) => n,
                Err(e) => {
                    invalid.push(format!("OIDC_KEY_BITS: {e}"));
                    DEFAULT_KEY_BITS
                }
            };
        let oidc_redirect_uri = take("OIDC_REDIRECT_URI", &mut missing);
        let oidc_secret_logs = !take_opt("OIDC_SECRET_LOGS", "true").eq_ignore_ascii_case("false");
        let app_url = take("APP_URL", &mut missing);
        let bind_addr = take("BIND_ADDR", &mut missing);
        let ldap_url = take("LDAP_URL", &mut missing);
        let ldap_base_dn = take("LDAP_BASE_DN", &mut missing);
        let ldap_people_ou = take_opt("LDAP_PEOPLE_OU", "people");
        let ldap_groups_ou = take_opt("LDAP_GROUPS_OU", "groups");
        let ldap_admin_pw = take_secret_opt("LDAP_ADMIN_PW", "admin", &mut file_errors);
        let ldap_tls_ca = take_opt("LDAP_TLS_CA", "");
        let oidc_client_owners = take_opt("OIDC_CLIENT_OWNERS", "");

        if !missing.is_empty() {
            return Err(EnvError(format!(
                "missing or empty env vars: {}. set them in devenv.nix env",
                missing.join(", ")
            )));
        }
        if !file_errors.is_empty() {
            return Err(EnvError(format!(
                "invalid secret files: {}",
                file_errors.join("; ")
            )));
        }
        if !invalid.is_empty() {
            return Err(EnvError(format!(
                "invalid env values: {}. set them in devenv.nix env",
                invalid.join("; ")
            )));
        }

        Ok(Self {
            oidc_issuer_url,
            oidc_client_id,
            oidc_redirect_uri,
            oidc_secrets_dir,
            oidc_clients_file,
            oidc_key_bits,
            oidc_secret_logs,
            app_url,
            bind_addr,
            ldap_url,
            ldap_base_dn,
            ldap_people_ou,
            ldap_groups_ou,
            ldap_admin_pw,
            ldap_tls_ca,
            oidc_client_owners,
        })
    }

    /// Logs the non-secret values at startup for production debugging. Secret
    /// contents are never logged, only the paths that hold them.
    pub fn log_summary(&self) {
        tracing::info!(
            oidc_issuer_url = %self.oidc_issuer_url,
            oidc_client_id = %self.oidc_client_id,
            oidc_secrets_dir = %self.oidc_secrets_dir,
            oidc_clients_file = %self.oidc_clients_file,
            oidc_key_bits = %self.oidc_key_bits,
            oidc_secret_logs = %self.oidc_secret_logs,
            oidc_redirect_uri = %self.oidc_redirect_uri,
            app_url = %self.app_url,
            bind_addr = %self.bind_addr,
            ldap_url = %self.ldap_url,
            ldap_base_dn = %self.ldap_base_dn,
            ldap_people_ou = %self.ldap_people_ou,
            ldap_groups_ou = %self.ldap_groups_ou,
            ldap_admin_pw_configured = %!self.ldap_admin_pw.is_empty(),
            ldap_tls_ca = %self.ldap_tls_ca,
            "backend env loaded"
        );
    }

    /// Stores the Env created once in `main`; second call is ignored.
    pub fn set(env: Env) -> &'static Env {
        let _ = ENV.set(env);
        ENV.get().unwrap()
    }

    /// Returns the Env stored via `set`; panics if not yet initialized.
    pub fn global() -> &'static Env {
        ENV.get()
            .expect("env not initialized: call Env::set in main")
    }

    /// For tests / fallback: ensure global is initialized (from env or compile-time fallback).
    pub fn ensure_init() {
        if ENV.get().is_none() {
            if let Ok(env) = Self::create() {
                let _ = ENV.set(env);
            } else {
                let env = Self::from_compile_fallback();
                let _ = ENV.set(env);
            }
        }
    }

    fn from_compile_fallback() -> Self {
        // ponytail: test fallback outside devenv — the secrets dir points to a
        // temp path where the test helpers write the clients file and
        // `secrets::ensure` creates the signing key + secret files.
        let oidc_secrets_dir = option_env!("OIDC_SECRETS_DIR")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "{}/portail-oidc-secrets-{}",
                    std::env::temp_dir().display(),
                    std::process::id()
                )
            });
        let oidc_clients_file = option_env!("OIDC_CLIENTS_FILE")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{oidc_secrets_dir}/clients.json"));
        Self {
            oidc_issuer_url: option_env!("OIDC_ISSUER_URL")
                .unwrap_or("http://127.0.0.1:3000")
                .to_string(),
            oidc_client_id: option_env!("OIDC_CLIENT_ID")
                .unwrap_or("portail-dev")
                .to_string(),
            oidc_redirect_uri: option_env!("OIDC_REDIRECT_URI")
                .unwrap_or("http://localhost:5173/callback")
                .to_string(),
            oidc_secrets_dir,
            oidc_clients_file,
            oidc_key_bits: option_env!("OIDC_KEY_BITS")
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(DEFAULT_KEY_BITS),
            oidc_secret_logs: option_env!("OIDC_SECRET_LOGS")
                .map(|s| !s.trim().eq_ignore_ascii_case("false"))
                .unwrap_or(true),
            app_url: option_env!("APP_URL")
                .unwrap_or("http://localhost:5173")
                .to_string(),
            bind_addr: option_env!("BIND_ADDR")
                .unwrap_or("0.0.0.0:3000")
                .to_string(),
            ldap_url: option_env!("LDAP_URL")
                .unwrap_or("ldap://127.0.0.1:3890")
                .to_string(),
            ldap_base_dn: option_env!("LDAP_BASE_DN")
                .unwrap_or("dc=dev,dc=example,dc=com")
                .to_string(),
            ldap_people_ou: option_env!("LDAP_PEOPLE_OU")
                .unwrap_or("people")
                .to_string(),
            ldap_groups_ou: option_env!("LDAP_GROUPS_OU")
                .unwrap_or("groups")
                .to_string(),
            ldap_admin_pw: option_env!("LDAP_ADMIN_PW").unwrap_or("admin").to_string(),
            ldap_tls_ca: option_env!("LDAP_TLS_CA").unwrap_or("").to_string(),
            oidc_client_owners: option_env!("OIDC_CLIENT_OWNERS").unwrap_or("").to_string(),
        }
    }
}

pub fn ldap_test_url() -> String {
    std::env::var("LDAP_TEST_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "ldap://127.0.0.1:3891".to_string())
}
pub fn ldap_test_base_dn() -> String {
    std::env::var("LDAP_TEST_BASE_DN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "dc=test,dc=example,dc=com".to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    static LOCK: Mutex<()> = Mutex::new(());

    /// Set all required vars to fixed values (secrets dir = fresh temp path).
    /// Returns the saved values to restore afterwards.
    fn save_and_set_all() -> Vec<(String, Option<String>)> {
        let names = [
            "OIDC_ISSUER_URL",
            "OIDC_CLIENT_ID",
            "OIDC_SECRETS_DIR",
            "OIDC_CLIENTS_FILE",
            "OIDC_REDIRECT_URI",
            "APP_URL",
            "BIND_ADDR",
            "LDAP_URL",
            "LDAP_BASE_DN",
            "LDAP_ADMIN_PW",
        ];
        let saved: Vec<(String, Option<String>)> = names
            .iter()
            .map(|k| (k.to_string(), std::env::var(k).ok()))
            .collect();
        let secrets_dir = format!("/tmp/portail-test-env-{}", std::process::id());
        let clients_file = format!("{secrets_dir}/clients.json");
        for (k, v) in [
            ("OIDC_ISSUER_URL", "http://127.0.0.1:3000"),
            ("OIDC_CLIENT_ID", "portail-dev"),
            ("OIDC_SECRETS_DIR", &secrets_dir),
            ("OIDC_CLIENTS_FILE", &clients_file),
            ("OIDC_REDIRECT_URI", "http://localhost:5173/callback"),
            ("APP_URL", "http://localhost:5173"),
            ("BIND_ADDR", "0.0.0.0:3000"),
            ("LDAP_URL", "ldap://127.0.0.1:3890"),
            ("LDAP_BASE_DN", "dc=dev,dc=example,dc=com"),
        ] {
            unsafe { std::env::set_var(k, v) };
        }
        unsafe { std::env::remove_var("LDAP_ADMIN_PW") };
        saved
    }

    fn restore_env(saved: Vec<(String, Option<String>)>) {
        for (k, v) in saved {
            if let Some(v) = v {
                unsafe { std::env::set_var(&k, v) };
            } else {
                unsafe { std::env::remove_var(&k) };
            }
        }
    }

    #[test]
    fn create_reports_all_missing() {
        let _g = LOCK.lock().unwrap();
        let vars = [
            "OIDC_ISSUER_URL",
            "OIDC_CLIENT_ID",
            "OIDC_SECRETS_DIR",
            "OIDC_CLIENTS_FILE",
            "OIDC_REDIRECT_URI",
            "APP_URL",
            "BIND_ADDR",
            "LDAP_URL",
            "LDAP_BASE_DN",
        ];
        let saved: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|k| (k.to_string(), std::env::var(k).ok()))
            .collect();

        for k in vars {
            unsafe { std::env::remove_var(k) };
        }

        let err = super::Env::create().unwrap_err();

        for k in vars {
            assert!(err.0.contains(k), "error missing {k}: {err}");
        }
        assert!(err.0.contains("devenv.nix env"), "hint missing: {err}");

        for (k, v) in saved {
            if let Some(val) = v {
                unsafe { std::env::set_var(&k, val) };
            }
        }
    }

    #[test]
    fn create_rejects_empty_string() {
        let _g = LOCK.lock().unwrap();
        let saved = std::env::var("APP_URL").ok();
        unsafe { std::env::set_var("APP_URL", "") };
        let err = super::Env::create().unwrap_err();
        assert!(
            err.0.contains("APP_URL"),
            "empty should be treated as missing: {err}"
        );
        if let Some(v) = saved {
            unsafe { std::env::set_var("APP_URL", v) };
        } else {
            unsafe { std::env::remove_var("APP_URL") };
        }
    }

    #[test]
    fn optional_vars_default_when_absent() {
        let _g = LOCK.lock().unwrap();
        let saved = (
            std::env::var("LDAP_PEOPLE_OU").ok(),
            std::env::var("LDAP_GROUPS_OU").ok(),
            std::env::var("LDAP_ADMIN_PW").ok(),
            std::env::var("LDAP_TLS_CA").ok(),
            std::env::var("OIDC_KEY_BITS").ok(),
            std::env::var("OIDC_CLIENT_OWNERS").ok(),
        );
        unsafe { std::env::remove_var("LDAP_PEOPLE_OU") };
        unsafe { std::env::remove_var("LDAP_GROUPS_OU") };
        unsafe { std::env::remove_var("LDAP_ADMIN_PW") };
        unsafe { std::env::remove_var("LDAP_TLS_CA") };
        unsafe { std::env::remove_var("OIDC_KEY_BITS") };
        unsafe { std::env::remove_var("OIDC_CLIENT_OWNERS") };

        let env = super::Env::create().unwrap();

        assert_eq!(env.ldap_people_ou, "people", "default people OU expected");
        assert_eq!(env.ldap_groups_ou, "groups", "default groups OU expected");
        assert_eq!(env.ldap_admin_pw, "admin", "default admin pw expected");
        assert_eq!(env.ldap_tls_ca, "", "default empty TLS CA expected");
        assert_eq!(
            env.oidc_key_bits,
            super::DEFAULT_KEY_BITS,
            "default RSA key size expected"
        );
        assert_eq!(
            env.oidc_client_owners, "",
            "default empty client owners expected"
        );

        if let Some(v) = saved.0 {
            unsafe { std::env::set_var("LDAP_PEOPLE_OU", v) };
        }
        if let Some(v) = saved.1 {
            unsafe { std::env::set_var("LDAP_GROUPS_OU", v) };
        }
        if let Some(v) = saved.2 {
            unsafe { std::env::set_var("LDAP_ADMIN_PW", v) };
        }
        if let Some(v) = saved.3 {
            unsafe { std::env::set_var("LDAP_TLS_CA", v) };
        }
        if let Some(v) = saved.4 {
            unsafe { std::env::set_var("OIDC_KEY_BITS", v) };
        }
        if let Some(v) = saved.5 {
            unsafe { std::env::set_var("OIDC_CLIENT_OWNERS", v) };
        }
    }

    #[test]
    fn optional_vars_custom_and_empty_falls_back() {
        let _g = LOCK.lock().unwrap();
        let saved = (
            std::env::var("LDAP_PEOPLE_OU").ok(),
            std::env::var("LDAP_GROUPS_OU").ok(),
            std::env::var("LDAP_TLS_CA").ok(),
        );
        unsafe { std::env::set_var("LDAP_PEOPLE_OU", "humains") };
        unsafe { std::env::set_var("LDAP_GROUPS_OU", "") };
        unsafe { std::env::set_var("LDAP_TLS_CA", "/tmp/fake-ca.crt") };

        let env = super::Env::create().unwrap();

        assert_eq!(env.ldap_people_ou, "humains", "custom OU expected");
        assert_eq!(env.ldap_groups_ou, "groups", "empty should fall back");
        assert_eq!(
            env.ldap_tls_ca, "/tmp/fake-ca.crt",
            "custom TLS CA path expected"
        );

        if let Some(v) = saved.0 {
            unsafe { std::env::set_var("LDAP_PEOPLE_OU", v) };
        }
        if let Some(v) = saved.1 {
            unsafe { std::env::set_var("LDAP_GROUPS_OU", v) };
        } else {
            unsafe { std::env::remove_var("LDAP_GROUPS_OU") };
        }
        if let Some(v) = saved.2 {
            unsafe { std::env::set_var("LDAP_TLS_CA", v) };
        }
    }

    #[test]
    fn required_oidc_secrets_dir_is_reported() {
        let _g = LOCK.lock().unwrap();
        let saved = std::env::var("OIDC_SECRETS_DIR").ok();
        unsafe { std::env::remove_var("OIDC_SECRETS_DIR") };

        let err = super::Env::create().unwrap_err();

        assert!(
            err.0.contains("OIDC_SECRETS_DIR"),
            "missing secrets dir should be reported: {err}"
        );

        if let Some(v) = saved {
            unsafe { std::env::set_var("OIDC_SECRETS_DIR", v) };
        }
    }

    #[test]
    fn required_oidc_clients_file_is_reported() {
        let _g = LOCK.lock().unwrap();
        let saved = std::env::var("OIDC_CLIENTS_FILE").ok();
        unsafe { std::env::remove_var("OIDC_CLIENTS_FILE") };

        let err = super::Env::create().unwrap_err();

        assert!(
            err.0.contains("OIDC_CLIENTS_FILE"),
            "missing clients file path should be reported: {err}"
        );

        if let Some(v) = saved {
            unsafe { std::env::set_var("OIDC_CLIENTS_FILE", v) };
        }
    }

    #[test]
    fn oidc_key_bits_custom_and_invalid() {
        let _g = LOCK.lock().unwrap();
        let saved = std::env::var("OIDC_KEY_BITS").ok();
        unsafe { std::env::set_var("OIDC_KEY_BITS", "4096") };

        let env = super::Env::create().unwrap();

        assert_eq!(env.oidc_key_bits, 4096, "custom key size expected");

        unsafe { std::env::set_var("OIDC_KEY_BITS", "not-a-number") };
        let err = super::Env::create().unwrap_err();

        assert!(
            err.0.contains("OIDC_KEY_BITS"),
            "non-numeric value should be reported: {err}"
        );

        match saved {
            Some(v) => unsafe { std::env::set_var("OIDC_KEY_BITS", v) },
            None => unsafe { std::env::remove_var("OIDC_KEY_BITS") },
        }
    }

    #[test]
    fn oidc_secret_logs_false_disables_and_absent_defaults_on() {
        let _g = LOCK.lock().unwrap();
        let saved = std::env::var("OIDC_SECRET_LOGS").ok();
        unsafe { std::env::set_var("OIDC_SECRET_LOGS", "false") };

        let env = super::Env::create().unwrap();

        assert!(!env.oidc_secret_logs, "false should disable secret logs");

        unsafe { std::env::remove_var("OIDC_SECRET_LOGS") };
        let env = super::Env::create().unwrap();

        assert!(env.oidc_secret_logs, "absent should default to enabled");

        match saved {
            Some(v) => unsafe { std::env::set_var("OIDC_SECRET_LOGS", v) },
            None => unsafe { std::env::remove_var("OIDC_SECRET_LOGS") },
        }
    }

    #[test]
    fn optional_secret_reads_file_content_and_reports_broken_file() {
        let _g = LOCK.lock().unwrap();
        let pw_file = std::env::temp_dir().join(format!("portail-ldap-pw-{}", std::process::id()));
        std::fs::write(&pw_file, "p4ss\n").unwrap();
        let saved = save_and_set_all();
        unsafe { std::env::set_var("LDAP_ADMIN_PW", pw_file.to_str().unwrap()) };

        let env = super::Env::create().unwrap();

        assert_eq!(
            env.ldap_admin_pw, "p4ss",
            "optional secret file content (newline trimmed) expected"
        );

        unsafe { std::env::set_var("LDAP_ADMIN_PW", "/tmp/portail-no-such-secret-file") };
        let err = super::Env::create().unwrap_err();
        assert!(
            err.0.contains("LDAP_ADMIN_PW") && err.0.contains("cannot read secret file"),
            "unreadable secret file should be reported: {err}"
        );

        restore_env(saved);
        let _ = std::fs::remove_file(&pw_file);
    }
}
