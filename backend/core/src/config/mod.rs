use serde::Deserialize;
use std::time::Duration;

pub mod client;
pub mod errors;
mod global;
pub use client::ClientEntry;
pub use errors::ConfigError;

pub const DEFAULT_KEY_BITS: usize = 2048;
pub const DEFAULT_PENDING_TTL_SECS: u64 = 300;
pub const DEFAULT_CODE_TTL_SECS: u64 = 60;
pub const DEFAULT_ACCESS_TTL_SECS: u64 = 3600;
pub const DEFAULT_SESSION_TTL_SECS: u64 = 12 * 3600;
pub const DEFAULT_PASSWORD_MIN_LENGTH: usize = 8;
pub const DEFAULT_LOGIN_MAX_ATTEMPTS: u32 = 10;
pub const DEFAULT_OIDC_ID_BYTES: usize = 16;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_url: String,
    pub bind_addr: String,
    pub ldap_url: String,
    pub ldap_base_dn: String,
    pub ldap_people_ou: String,
    pub ldap_groups_ou: String,
    pub ldap_admin_pw: String,
    pub ldap_tls_ca: String,
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_redirect_uri: String,
    pub oidc_secrets_dir: String,
    pub oidc_clients: Vec<ClientEntry>,
    pub oidc_key_bits: usize,
    pub oidc_secret_logs: bool,
    pub oidc_client_owners: String,
    pub oidc_pending_ttl: Duration,
    pub oidc_code_ttl: Duration,
    pub oidc_session_ttl: Duration,
    pub oidc_access_ttl: Duration,
    pub password_min_length: usize,
    pub login_max_attempts: u32,
    pub oidc_id_bytes: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    app_url: String,
    bind_addr: String,
    ldap_url: String,
    ldap_base_dn: String,
    #[serde(default = "default_people_ou")]
    ldap_people_ou: String,
    #[serde(default = "default_groups_ou")]
    ldap_groups_ou: String,
    #[serde(default)]
    ldap_admin_pw_file: Option<String>,
    #[serde(default)]
    ldap_tls_ca: String,
    oidc_issuer_url: String,
    oidc_client_id: String,
    oidc_redirect_uri: String,
    oidc_secrets_dir: String,
    #[serde(default)]
    oidc_clients: Vec<ClientEntry>,
    #[serde(default)]
    oidc_key_bits: Option<usize>,
    #[serde(default = "default_true")]
    oidc_secret_logs: bool,
    #[serde(default)]
    oidc_client_owners: String,
    #[serde(default)]
    oidc_pending_ttl_secs: Option<u64>,
    #[serde(default)]
    oidc_code_ttl_secs: Option<u64>,
    #[serde(default)]
    oidc_session_ttl_secs: Option<u64>,
    #[serde(default)]
    oidc_access_ttl_secs: Option<u64>,
    #[serde(default)]
    password_min_length: Option<usize>,
    #[serde(default)]
    login_max_attempts: Option<u32>,
    #[serde(default)]
    oidc_id_bytes: Option<usize>,
}

fn default_people_ou() -> String {
    "people".to_string()
}

fn default_groups_ou() -> String {
    "groups".to_string()
}

fn default_true() -> bool {
    true
}

fn read_secret_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("cannot read secret file {path}: {e}"))
        .and_then(|s| {
            if s.is_empty() {
                Err(format!("secret file {path} is empty"))
            } else {
                Ok(s)
            }
        })
}

fn resolve_path(base_dir: &str, value: &str) -> String {
    if value.is_empty() || value.starts_with('/') || base_dir.is_empty() {
        value.to_string()
    } else {
        format!("{base_dir}/{value}")
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ConfigError(format!("cannot read config file {path}: {e}")))?;
        let parsed: RawConfig = toml::from_str(&raw)
            .map_err(|e| ConfigError(format!("invalid config file {path}: {e}")))?;
        let base_dir = std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");
        let ldap_admin_pw = match parsed.ldap_admin_pw_file {
            Some(file) => read_secret_file(&resolve_path(base_dir, &file))
                .map_err(|e| ConfigError(format!("ldap_admin_pw_file: {e}")))?,
            None => "admin".to_string(),
        };
        let oidc_id_bytes = parsed.oidc_id_bytes.unwrap_or(DEFAULT_OIDC_ID_BYTES);
        if oidc_id_bytes < DEFAULT_OIDC_ID_BYTES {
            return Err(ConfigError(format!(
                "oidc_id_bytes must stay at least {DEFAULT_OIDC_ID_BYTES} (got {oidc_id_bytes})"
            )));
        }
        Ok(Self {
            app_url: parsed.app_url,
            bind_addr: parsed.bind_addr,
            ldap_url: parsed.ldap_url,
            ldap_base_dn: parsed.ldap_base_dn,
            ldap_people_ou: parsed.ldap_people_ou,
            ldap_groups_ou: parsed.ldap_groups_ou,
            ldap_admin_pw,
            ldap_tls_ca: resolve_path(base_dir, &parsed.ldap_tls_ca),
            oidc_issuer_url: parsed.oidc_issuer_url,
            oidc_client_id: parsed.oidc_client_id,
            oidc_redirect_uri: parsed.oidc_redirect_uri,
            oidc_secrets_dir: resolve_path(base_dir, &parsed.oidc_secrets_dir),
            oidc_clients: parsed.oidc_clients,
            oidc_key_bits: parsed.oidc_key_bits.unwrap_or(DEFAULT_KEY_BITS),
            oidc_secret_logs: parsed.oidc_secret_logs,
            oidc_client_owners: parsed.oidc_client_owners,
            oidc_pending_ttl: Duration::from_secs(
                parsed
                    .oidc_pending_ttl_secs
                    .unwrap_or(DEFAULT_PENDING_TTL_SECS),
            ),
            oidc_code_ttl: Duration::from_secs(
                parsed.oidc_code_ttl_secs.unwrap_or(DEFAULT_CODE_TTL_SECS),
            ),
            oidc_session_ttl: Duration::from_secs(
                parsed
                    .oidc_session_ttl_secs
                    .unwrap_or(DEFAULT_SESSION_TTL_SECS),
            ),
            oidc_access_ttl: Duration::from_secs(
                parsed
                    .oidc_access_ttl_secs
                    .unwrap_or(DEFAULT_ACCESS_TTL_SECS),
            ),
            password_min_length: parsed
                .password_min_length
                .unwrap_or(DEFAULT_PASSWORD_MIN_LENGTH),
            login_max_attempts: parsed
                .login_max_attempts
                .unwrap_or(DEFAULT_LOGIN_MAX_ATTEMPTS),
            oidc_id_bytes,
        })
    }

    pub fn log_summary(&self) {
        tracing::info!(
            app_url = %self.app_url,
            bind_addr = %self.bind_addr,
            ldap_url = %self.ldap_url,
            ldap_base_dn = %self.ldap_base_dn,
            ldap_people_ou = %self.ldap_people_ou,
            ldap_groups_ou = %self.ldap_groups_ou,
            ldap_admin_pw_configured = %!self.ldap_admin_pw.is_empty(),
            ldap_tls_ca = %self.ldap_tls_ca,
            oidc_issuer_url = %self.oidc_issuer_url,
            oidc_client_id = %self.oidc_client_id,
            oidc_secrets_dir = %self.oidc_secrets_dir,
            oidc_client_ids = %self.oidc_clients.iter().map(|c| c.id.as_str()).collect::<Vec<_>>().join(","),
            oidc_key_bits = %self.oidc_key_bits,
            oidc_secret_logs = %self.oidc_secret_logs,
            oidc_redirect_uri = %self.oidc_redirect_uri,
            oidc_pending_ttl_secs = %self.oidc_pending_ttl.as_secs(),
            oidc_code_ttl_secs = %self.oidc_code_ttl.as_secs(),
            oidc_session_ttl_secs = %self.oidc_session_ttl.as_secs(),
            oidc_access_ttl_secs = %self.oidc_access_ttl.as_secs(),
            password_min_length = %self.password_min_length,
            login_max_attempts = %self.login_max_attempts,
            oidc_id_bytes = %self.oidc_id_bytes,
            "backend config loaded"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_config(name: &str, contents: &str) -> String {
        let path =
            std::env::temp_dir().join(format!("portail-config-{name}-{}.toml", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        path.to_str().unwrap().to_string()
    }

    fn minimal_toml() -> String {
        r#"
app_url = "http://localhost:5173"
bind_addr = "0.0.0.0:3000"
ldap_url = "ldap://127.0.0.1:3890"
ldap_base_dn = "dc=dev,dc=example,dc=com"
oidc_issuer_url = "http://127.0.0.1:3000"
oidc_client_id = "portail-dev"
oidc_redirect_uri = "http://localhost:5173/callback"
oidc_secrets_dir = "/tmp/portail-test-secrets"
oidc_clients = [{ id = "portail-dev", secret_file = "portail-dev_secret", enabled = true, redirect_uris = ["http://localhost:5173/callback"], scopes = ["openid"] }]
"#
        .to_string()
    }

    #[test]
    fn load_applies_defaults_for_optional_fields() {
        let path = write_temp_config("defaults", &minimal_toml());

        let config = Config::load(&path).unwrap();

        assert_eq!(config.ldap_people_ou, "people", "default OU expected");
        assert_eq!(config.ldap_groups_ou, "groups", "default OU expected");
        assert_eq!(
            config.ldap_admin_pw, "admin",
            "default admin password expected"
        );
        assert_eq!(config.ldap_tls_ca, "", "default empty TLS CA expected");
        assert_eq!(
            config.oidc_key_bits, DEFAULT_KEY_BITS,
            "default RSA key size expected"
        );
        assert!(config.oidc_secret_logs, "secret logs default on");
        assert_eq!(
            config.oidc_pending_ttl,
            Duration::from_secs(DEFAULT_PENDING_TTL_SECS),
            "default pending ttl expected"
        );
        assert_eq!(
            config.oidc_code_ttl,
            Duration::from_secs(DEFAULT_CODE_TTL_SECS),
            "default code ttl expected"
        );
        assert_eq!(
            config.oidc_session_ttl,
            Duration::from_secs(DEFAULT_SESSION_TTL_SECS),
            "default session ttl expected"
        );
        assert_eq!(
            config.oidc_access_ttl,
            Duration::from_secs(DEFAULT_ACCESS_TTL_SECS),
            "default access token ttl expected"
        );
        assert_eq!(
            config.password_min_length, DEFAULT_PASSWORD_MIN_LENGTH,
            "default password min length expected"
        );
        assert_eq!(
            config.login_max_attempts, DEFAULT_LOGIN_MAX_ATTEMPTS,
            "default login attempt cap expected"
        );
        assert_eq!(
            config.oidc_id_bytes, DEFAULT_OIDC_ID_BYTES,
            "default id entropy expected"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_id_bytes_below_floor() {
        let path = write_temp_config("weakid", &format!("{}oidc_id_bytes = 4\n", minimal_toml()));

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("oidc_id_bytes") && err.0.contains("at least"),
            "id entropy below the floor should be rejected: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reads_custom_values() {
        let pw_file =
            std::env::temp_dir().join(format!("portail-config-pw-{}", std::process::id()));
        std::fs::write(&pw_file, "p4ss\n").unwrap();
        let path = write_temp_config(
            "custom",
            &format!(
                "{}ldap_admin_pw_file = \"{}\"\nldap_people_ou = \"humains\"\noidc_code_ttl_secs = 90\noidc_key_bits = 4096\npassword_min_length = 12\nlogin_max_attempts = 5\noidc_access_ttl_secs = 120\noidc_id_bytes = 32\n",
                minimal_toml(),
                pw_file.to_str().unwrap()
            ),
        );

        let config = Config::load(&path).unwrap();

        assert_eq!(
            config.ldap_admin_pw, "p4ss",
            "secret file content (newline trimmed) expected"
        );
        assert_eq!(config.ldap_people_ou, "humains", "custom OU expected");
        assert_eq!(
            config.oidc_code_ttl,
            Duration::from_secs(90),
            "custom code ttl expected"
        );
        assert_eq!(config.oidc_key_bits, 4096, "custom key size expected");
        assert_eq!(
            config.password_min_length, 12,
            "custom password min length expected"
        );
        assert_eq!(
            config.login_max_attempts, 5,
            "custom login attempt cap expected"
        );
        assert_eq!(
            config.oidc_access_ttl,
            Duration::from_secs(120),
            "custom access token ttl expected"
        );
        assert_eq!(
            config.oidc_id_bytes, 32,
            "custom id entropy should be allowed upward"
        );
        assert_eq!(config.oidc_clients.len(), 1, "one inline client expected");
        assert_eq!(
            config.oidc_clients[0].id, "portail-dev",
            "inline client id expected"
        );
        assert_eq!(
            config.oidc_clients[0].secret_file.as_deref(),
            Some("portail-dev_secret"),
            "inline client secret_file expected"
        );
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&pw_file);
    }

    #[test]
    fn load_resolves_relative_paths_against_config_dir() {
        let path = write_temp_config(
            "relative",
            &minimal_toml().replace(
                r#"oidc_secrets_dir = "/tmp/portail-test-secrets""#,
                r#"oidc_secrets_dir = ".devenv/state/secrets""#,
            ),
        );
        let config_dir = std::path::Path::new(&path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let config = Config::load(&path).unwrap();

        assert_eq!(
            config.oidc_secrets_dir,
            format!("{config_dir}/.devenv/state/secrets"),
            "relative path should anchor at the config file directory"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reports_missing_required_field() {
        let path = write_temp_config(
            "missing",
            &minimal_toml().replace("bind_addr = \"0.0.0.0:3000\"\n", ""),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("bind_addr"),
            "missing required field should be named: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reports_unknown_field() {
        let path = write_temp_config("unknown", &format!("{}typo_key = 1\n", minimal_toml()));

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("typo_key"),
            "unknown field should be named: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reports_invalid_toml() {
        let path = write_temp_config("broken", "not = = valid toml");

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("invalid config file"),
            "broken toml should be reported: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reports_unreadable_file() {
        let err = Config::load("/tmp/portail-no-such-config-file").unwrap_err();

        assert!(
            err.0.contains("cannot read config file"),
            "missing config file should be reported: {err}"
        );
    }

    #[test]
    fn load_reports_broken_secret_file() {
        let path = write_temp_config(
            "brokenpw",
            &format!(
                "{}ldap_admin_pw_file = \"/tmp/portail-no-such-secret-file\"\n",
                minimal_toml()
            ),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("ldap_admin_pw_file") && err.0.contains("cannot read secret file"),
            "unreadable secret file should be reported: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
