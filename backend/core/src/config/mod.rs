use serde::Deserialize;
use std::time::Duration;

pub mod client;
pub mod errors;
mod global;
pub use client::{ClientEntry, CustomClaim};
pub use errors::ConfigError;

pub const DEFAULT_KEY_BITS: usize = 2048;
pub const DEFAULT_PENDING_TTL_SECS: u64 = 300;
pub const DEFAULT_CODE_TTL_SECS: u64 = 60;
pub const DEFAULT_ACCESS_TTL_SECS: u64 = 3600;
pub const DEFAULT_SESSION_TTL_SECS: u64 = 12 * 3600;
pub const DEFAULT_PASSWORD_MIN_LENGTH: usize = 8;
pub const DEFAULT_LOGIN_MAX_ATTEMPTS: u32 = 10;
pub const DEFAULT_OIDC_ID_BYTES: usize = 16;
pub const DEFAULT_DB_PATH: &str = "portail.db";
pub const SYSTEM_SENSOR_NAMES: [&str; 14] = [
    "memory",
    "cpu",
    "cores",
    "load",
    "pressure",
    "temps",
    "fans",
    "network",
    "tcp",
    "interfaces",
    "disk",
    "diskio",
    "raid",
    "uptime",
];
pub const DEFAULT_SYSTEM_SENSORS: [&str; 4] = ["memory", "cpu", "network", "disk"];
pub const RESERVED_CLAIM_NAMES: [&str; 11] = [
    "iss",
    "sub",
    "aud",
    "exp",
    "iat",
    "nbf",
    "nonce",
    "azp",
    "email",
    "groups",
    "preferred_username",
];

#[derive(Debug, Clone)]
pub struct LdapConfig {
    pub url: String,
    pub base_dn: String,
    pub people_ou: String,
    pub groups_ou: String,
    pub admin_pw: String,
    pub tls_ca: String,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub secrets_dir: String,
    pub clients: Vec<ClientEntry>,
    pub key_bits: usize,
    pub secret_logs: bool,
    pub client_owners: String,
    pub pending_ttl: Duration,
    pub code_ttl: Duration,
    pub session_ttl: Duration,
    pub access_ttl: Duration,
    pub id_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub password_min_length: usize,
    pub login_max_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub app_url: String,
    pub bind_addr: String,
    pub db_path: String,
    pub ldap: LdapConfig,
    pub oidc: OidcConfig,
    pub security: SecurityConfig,
    pub system_sensors: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    app_url: String,
    bind_addr: String,
    #[serde(default = "default_db_path")]
    db_path: String,
    ldap: RawLdap,
    oidc: RawOidc,
    #[serde(default)]
    security: RawSecurity,
    #[serde(default = "default_system_sensors")]
    system_sensors: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLdap {
    url: String,
    base_dn: String,
    #[serde(default = "default_people_ou")]
    people_ou: String,
    #[serde(default = "default_groups_ou")]
    groups_ou: String,
    #[serde(default)]
    admin_pw_file: Option<String>,
    #[serde(default)]
    tls_ca: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOidc {
    issuer_url: String,
    client_id: String,
    redirect_uri: String,
    secrets_dir: String,
    #[serde(default)]
    clients: Vec<ClientEntry>,
    #[serde(default)]
    key_bits: Option<usize>,
    #[serde(default = "default_true")]
    secret_logs: bool,
    #[serde(default)]
    client_owners: String,
    #[serde(default)]
    pending_ttl_secs: Option<u64>,
    #[serde(default)]
    code_ttl_secs: Option<u64>,
    #[serde(default)]
    session_ttl_secs: Option<u64>,
    #[serde(default)]
    access_ttl_secs: Option<u64>,
    #[serde(default)]
    id_bytes: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawSecurity {
    #[serde(default)]
    password_min_length: Option<usize>,
    #[serde(default)]
    login_max_attempts: Option<u32>,
}

fn default_people_ou() -> String {
    "people".to_string()
}

fn default_groups_ou() -> String {
    "groups".to_string()
}

fn default_db_path() -> String {
    DEFAULT_DB_PATH.to_string()
}

fn default_true() -> bool {
    true
}

fn default_system_sensors() -> Vec<String> {
    DEFAULT_SYSTEM_SENSORS
        .iter()
        .map(|s| s.to_string())
        .collect()
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
        let admin_pw = match parsed.ldap.admin_pw_file {
            Some(file) => read_secret_file(&resolve_path(base_dir, &file))
                .map_err(|e| ConfigError(format!("ldap.admin_pw_file: {e}")))?,
            None => "admin".to_string(),
        };
        let id_bytes = parsed.oidc.id_bytes.unwrap_or(DEFAULT_OIDC_ID_BYTES);
        if id_bytes < DEFAULT_OIDC_ID_BYTES {
            return Err(ConfigError(format!(
                "oidc.id_bytes must stay at least {DEFAULT_OIDC_ID_BYTES} (got {id_bytes})"
            )));
        }
        if let Some(unknown) = parsed
            .system_sensors
            .iter()
            .find(|sensor| !SYSTEM_SENSOR_NAMES.contains(&sensor.as_str()))
        {
            return Err(ConfigError(format!(
                "system_sensors contains unknown sensor '{unknown}' (known: {})",
                SYSTEM_SENSOR_NAMES.join(", ")
            )));
        }
        for entry in &parsed.oidc.clients {
            for rule in &entry.custom_claims {
                if rule.name.is_empty() {
                    return Err(ConfigError(format!(
                        "client {}: custom claim rule needs a non-empty name",
                        entry.id
                    )));
                }
                if RESERVED_CLAIM_NAMES.contains(&rule.name.as_str()) {
                    return Err(ConfigError(format!(
                        "client {}: custom claim '{}' shadows a reserved claim",
                        entry.id, rule.name
                    )));
                }
                if !rule.group.is_empty() && !rule.not_group.is_empty() {
                    return Err(ConfigError(format!(
                        "client {}: custom claim '{}' cannot set both group and not_group",
                        entry.id, rule.name
                    )));
                }
            }
            if !entry.required_group.is_empty() && entry.required_group == entry.forbidden_group {
                return Err(ConfigError(format!(
                    "client {}: required_group and forbidden_group are the same group, nobody could ever connect",
                    entry.id
                )));
            }
        }
        Ok(Self {
            app_url: parsed.app_url,
            bind_addr: parsed.bind_addr,
            db_path: resolve_path(base_dir, &parsed.db_path),
            ldap: LdapConfig {
                url: parsed.ldap.url,
                base_dn: parsed.ldap.base_dn,
                people_ou: parsed.ldap.people_ou,
                groups_ou: parsed.ldap.groups_ou,
                admin_pw,
                tls_ca: resolve_path(base_dir, &parsed.ldap.tls_ca),
            },
            oidc: OidcConfig {
                issuer_url: parsed.oidc.issuer_url,
                client_id: parsed.oidc.client_id,
                redirect_uri: parsed.oidc.redirect_uri,
                secrets_dir: resolve_path(base_dir, &parsed.oidc.secrets_dir),
                clients: parsed.oidc.clients,
                key_bits: parsed.oidc.key_bits.unwrap_or(DEFAULT_KEY_BITS),
                secret_logs: parsed.oidc.secret_logs,
                client_owners: parsed.oidc.client_owners,
                pending_ttl: Duration::from_secs(
                    parsed
                        .oidc
                        .pending_ttl_secs
                        .unwrap_or(DEFAULT_PENDING_TTL_SECS),
                ),
                code_ttl: Duration::from_secs(
                    parsed.oidc.code_ttl_secs.unwrap_or(DEFAULT_CODE_TTL_SECS),
                ),
                session_ttl: Duration::from_secs(
                    parsed
                        .oidc
                        .session_ttl_secs
                        .unwrap_or(DEFAULT_SESSION_TTL_SECS),
                ),
                access_ttl: Duration::from_secs(
                    parsed
                        .oidc
                        .access_ttl_secs
                        .unwrap_or(DEFAULT_ACCESS_TTL_SECS),
                ),
                id_bytes,
            },
            security: SecurityConfig {
                password_min_length: parsed
                    .security
                    .password_min_length
                    .unwrap_or(DEFAULT_PASSWORD_MIN_LENGTH),
                login_max_attempts: parsed
                    .security
                    .login_max_attempts
                    .unwrap_or(DEFAULT_LOGIN_MAX_ATTEMPTS),
            },
            system_sensors: parsed.system_sensors,
        })
    }

    pub fn log_summary(&self) {
        tracing::info!(
            app_url = %self.app_url,
            bind_addr = %self.bind_addr,
            db_path = %self.db_path,
            ldap_url = %self.ldap.url,
            ldap_base_dn = %self.ldap.base_dn,
            ldap_people_ou = %self.ldap.people_ou,
            ldap_groups_ou = %self.ldap.groups_ou,
            ldap_admin_pw_configured = %!self.ldap.admin_pw.is_empty(),
            ldap_tls_ca = %self.ldap.tls_ca,
            oidc_issuer_url = %self.oidc.issuer_url,
            oidc_client_id = %self.oidc.client_id,
            oidc_secrets_dir = %self.oidc.secrets_dir,
            oidc_client_ids = %self.oidc.clients.iter().map(|c| c.id.as_str()).collect::<Vec<_>>().join(","),
            oidc_key_bits = %self.oidc.key_bits,
            oidc_secret_logs = %self.oidc.secret_logs,
            oidc_redirect_uri = %self.oidc.redirect_uri,
            oidc_pending_ttl_secs = %self.oidc.pending_ttl.as_secs(),
            oidc_code_ttl_secs = %self.oidc.code_ttl.as_secs(),
            oidc_session_ttl_secs = %self.oidc.session_ttl.as_secs(),
            oidc_access_ttl_secs = %self.oidc.access_ttl.as_secs(),
            password_min_length = %self.security.password_min_length,
            login_max_attempts = %self.security.login_max_attempts,
            oidc_id_bytes = %self.oidc.id_bytes,
            system_sensors = %self.system_sensors.join(","),
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

[ldap]
url = "ldap://127.0.0.1:3890"
base_dn = "dc=dev,dc=example,dc=com"

[oidc]
issuer_url = "http://127.0.0.1:3000"
client_id = "portail-dev"
redirect_uri = "http://localhost:5173/callback"
secrets_dir = "/tmp/portail-test-secrets"
clients = [{ id = "portail-dev", secret_file = "portail-dev_secret", enabled = true, redirect_uris = ["http://localhost:5173/callback"], scopes = ["openid"] }]
"#
        .to_string()
    }

    fn with_client_claims(claims: &str) -> String {
        minimal_toml().replace(
            r#"scopes = ["openid"] }]"#,
            &format!(r#"scopes = ["openid"], custom_claims = {claims} }}]"#),
        )
    }

    #[test]
    fn load_applies_defaults_for_optional_fields() {
        let path = write_temp_config("defaults", &minimal_toml());

        let config = Config::load(&path).unwrap();

        assert_eq!(config.ldap.people_ou, "people", "default OU expected");
        assert_eq!(config.ldap.groups_ou, "groups", "default OU expected");
        assert_eq!(
            config.ldap.admin_pw, "admin",
            "default admin password expected"
        );
        assert_eq!(config.ldap.tls_ca, "", "default empty TLS CA expected");
        assert_eq!(
            config.db_path,
            format!("{}/portail.db", std::env::temp_dir().display()),
            "default db path resolved against the config dir expected"
        );
        assert_eq!(
            config.oidc.key_bits, DEFAULT_KEY_BITS,
            "default RSA key size expected"
        );
        assert!(config.oidc.secret_logs, "secret logs default on");
        assert_eq!(
            config.oidc.pending_ttl,
            Duration::from_secs(DEFAULT_PENDING_TTL_SECS),
            "default pending ttl expected"
        );
        assert_eq!(
            config.oidc.code_ttl,
            Duration::from_secs(DEFAULT_CODE_TTL_SECS),
            "default code ttl expected"
        );
        assert_eq!(
            config.oidc.session_ttl,
            Duration::from_secs(DEFAULT_SESSION_TTL_SECS),
            "default session ttl expected"
        );
        assert_eq!(
            config.oidc.access_ttl,
            Duration::from_secs(DEFAULT_ACCESS_TTL_SECS),
            "default access token ttl expected"
        );
        assert_eq!(
            config.security.password_min_length, DEFAULT_PASSWORD_MIN_LENGTH,
            "default password min length expected"
        );
        assert_eq!(
            config.security.login_max_attempts, DEFAULT_LOGIN_MAX_ATTEMPTS,
            "default login attempt cap expected"
        );
        assert_eq!(
            config.oidc.id_bytes, DEFAULT_OIDC_ID_BYTES,
            "default id entropy expected"
        );
        assert_eq!(
            config.system_sensors,
            DEFAULT_SYSTEM_SENSORS
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            "default NAS-relevant sensors expected"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_unknown_system_sensor() {
        let path = write_temp_config(
            "badsensor",
            &minimal_toml().replace(
                "bind_addr = \"0.0.0.0:3000\"\n",
                "bind_addr = \"0.0.0.0:3000\"\nsystem_sensors = [\"memory\", \"voltage\"]\n",
            ),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("voltage") && err.0.contains("unknown sensor"),
            "an unknown sensor name must be rejected: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_id_bytes_below_floor() {
        let path = write_temp_config("weakid", &format!("{}id_bytes = 4\n", minimal_toml()));

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("id_bytes") && err.0.contains("at least"),
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
                r#"app_url = "http://localhost:5173"
bind_addr = "0.0.0.0:3000"
db_path = "data/portail.db"
system_sensors = ["memory", "fans", "tcp"]

[ldap]
url = "ldap://127.0.0.1:3890"
base_dn = "dc=dev,dc=example,dc=com"
people_ou = "humains"
admin_pw_file = "{}"

[oidc]
issuer_url = "http://127.0.0.1:3000"
client_id = "portail-dev"
redirect_uri = "http://localhost:5173/callback"
secrets_dir = "/tmp/portail-test-secrets"
clients = [{{ id = "portail-dev", secret_file = "portail-dev_secret", enabled = true, redirect_uris = ["http://localhost:5173/callback"], scopes = ["openid"] }}]
code_ttl_secs = 90
key_bits = 4096
access_ttl_secs = 120
id_bytes = 32

[security]
password_min_length = 12
login_max_attempts = 5
"#,
                pw_file.to_str().unwrap()
            ),
        );

        let config = Config::load(&path).unwrap();

        assert_eq!(
            config.ldap.admin_pw, "p4ss",
            "secret file content (newline trimmed) expected"
        );
        assert_eq!(config.ldap.people_ou, "humains", "custom OU expected");
        assert_eq!(
            config.db_path,
            format!("{}/data/portail.db", std::env::temp_dir().display()),
            "custom db path resolved against the config dir expected"
        );
        assert_eq!(
            config.oidc.code_ttl,
            Duration::from_secs(90),
            "custom code ttl expected"
        );
        assert_eq!(config.oidc.key_bits, 4096, "custom key size expected");
        assert_eq!(
            config.security.password_min_length, 12,
            "custom password min length expected"
        );
        assert_eq!(
            config.security.login_max_attempts, 5,
            "custom login attempt cap expected"
        );
        assert_eq!(
            config.oidc.access_ttl,
            Duration::from_secs(120),
            "custom access token ttl expected"
        );
        assert_eq!(
            config.oidc.id_bytes, 32,
            "custom id entropy should be allowed upward"
        );
        assert_eq!(
            config.system_sensors,
            vec!["memory".to_string(), "fans".to_string(), "tcp".to_string()],
            "custom sensor selection expected"
        );
        assert_eq!(config.oidc.clients.len(), 1, "one inline client expected");
        assert_eq!(
            config.oidc.clients[0].id, "portail-dev",
            "inline client id expected"
        );
        assert_eq!(
            config.oidc.clients[0].secret_file.as_deref(),
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
                r#"secrets_dir = "/tmp/portail-test-secrets""#,
                r#"secrets_dir = ".devenv/state/secrets""#,
            ),
        );
        let config_dir = std::path::Path::new(&path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let config = Config::load(&path).unwrap();

        assert_eq!(
            config.oidc.secrets_dir,
            format!("{config_dir}/.devenv/state/secrets"),
            "relative path should anchor at the config file directory"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_reserved_custom_claim_name() {
        let path = write_temp_config(
            "reserved",
            &with_client_claims(r#"[{ name = "sub", value = "evil" }]"#),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("reserved claim") && err.0.contains("sub"),
            "a custom claim shadowing a reserved name must be rejected: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_reads_group_scoped_custom_claims() {
        let path = write_temp_config(
            "customclaims",
            &with_client_claims(
                r#"[{ name = "env", value = "test" }, { name = "role", value = "admin", group = "admins" }, { name = "outsider", value = "yes", not_group = "admins" }]"#,
            ),
        );

        let config = Config::load(&path).unwrap();
        let rules = &config.oidc.clients[0].custom_claims;

        assert_eq!(rules.len(), 3, "three custom claim rules expected");
        assert_eq!(
            rules[0].group, "",
            "first rule should be unconditional (empty group)"
        );
        assert_eq!(
            rules[1].group, "admins",
            "second rule should be scoped to the admins group"
        );
        assert_eq!(
            rules[2].not_group, "admins",
            "third rule should exclude the admins group"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_rejects_conflicting_group_gates() {
        let path = write_temp_config(
            "conflict",
            &with_client_claims(r#"[{ name = "x", value = "y", group = "a", not_group = "b" }]"#),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("both group and not_group"),
            "a rule setting both gates must be rejected: {err}"
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
            &minimal_toml().replace(
                "base_dn = \"dc=dev,dc=example,dc=com\"\n",
                "base_dn = \"dc=dev,dc=example,dc=com\"\nadmin_pw_file = \"/tmp/portail-no-such-secret-file\"\n",
            ),
        );

        let err = Config::load(&path).unwrap_err();

        assert!(
            err.0.contains("admin_pw_file") && err.0.contains("cannot read secret file"),
            "unreadable secret file should be reported: {err}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
