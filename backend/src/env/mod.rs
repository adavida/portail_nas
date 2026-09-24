pub mod errors;
pub use errors::EnvError;

use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Env {
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub oidc_redirect_uri: String,
    pub app_url: String,
    pub bind_addr: String,
    pub ldap_url: String,
    pub ldap_base_dn: String,
    pub ldap_people_ou: String,
    pub ldap_groups_ou: String,
    pub ldap_admin_pw: String,
}

static ENV: OnceLock<Env> = OnceLock::new();

/// Secret vars (`OIDC_CLIENT_SECRET`, `LDAP_ADMIN_PW`) hold the **path** of a
/// file whose content is the secret. Trims whitespace/newline around the content.
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

        let take = |name: &'static str, missing: &mut Vec<&'static str>| -> String {
            match std::env::var(name) {
                Ok(v) if !v.trim().is_empty() => v,
                _ => {
                    missing.push(name);
                    String::new()
                }
            }
        };

        // Optional: absent/empty value falls back to the documented default.
        let take_opt = |name: &'static str, default: &str| -> String {
            std::env::var(name)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| default.to_string())
        };

        // Secret: var value is a path whose file content is the value.
        let take_secret = |name: &'static str,
                           missing: &mut Vec<&'static str>,
                           file_errors: &mut Vec<String>|
         -> String {
            match read_secret(name) {
                Some(Ok(v)) => v,
                Some(Err(e)) => {
                    file_errors.push(e);
                    String::new()
                }
                None => {
                    missing.push(name);
                    String::new()
                }
            }
        };
        // Optional secret: absent -> default; broken file -> reported like required.
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
        let oidc_client_secret = take_secret("OIDC_CLIENT_SECRET", &mut missing, &mut file_errors);
        let oidc_redirect_uri = take("OIDC_REDIRECT_URI", &mut missing);
        let app_url = take("APP_URL", &mut missing);
        let bind_addr = take("BIND_ADDR", &mut missing);
        let ldap_url = take("LDAP_URL", &mut missing);
        let ldap_base_dn = take("LDAP_BASE_DN", &mut missing);
        let ldap_people_ou = take_opt("LDAP_PEOPLE_OU", "people");
        let ldap_groups_ou = take_opt("LDAP_GROUPS_OU", "groups");
        let ldap_admin_pw = take_secret_opt("LDAP_ADMIN_PW", "admin", &mut file_errors);

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

        Ok(Self {
            oidc_issuer_url,
            oidc_client_id,
            oidc_client_secret,
            oidc_redirect_uri,
            app_url,
            bind_addr,
            ldap_url,
            ldap_base_dn,
            ldap_people_ou,
            ldap_groups_ou,
            ldap_admin_pw,
        })
    }

    /// Logs every value at startup for production debugging. The secret vars
    /// hold file paths (never the secret itself), so the full readout is safe.
    pub fn log_summary(&self) {
        tracing::info!(
            oidc_issuer_url = %self.oidc_issuer_url,
            oidc_client_id = %self.oidc_client_id,
            oidc_client_secret = %self.oidc_client_secret,
            oidc_redirect_uri = %self.oidc_redirect_uri,
            app_url = %self.app_url,
            bind_addr = %self.bind_addr,
            ldap_url = %self.ldap_url,
            ldap_base_dn = %self.ldap_base_dn,
            ldap_people_ou = %self.ldap_people_ou,
            ldap_groups_ou = %self.ldap_groups_ou,
            ldap_admin_pw = %self.ldap_admin_pw,
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
        // ponytail: secret vars hold paths at runtime; when vars are missing
        // (test fallback) we return literal dev secrets — a set-but-unreadable
        // secret file silently degrades to the dev default. Prod fails at `create()`.
        Self {
            oidc_issuer_url: option_env!("OIDC_ISSUER_URL")
                .unwrap_or("https://127.0.0.1:9091")
                .to_string(),
            oidc_client_id: option_env!("OIDC_CLIENT_ID")
                .unwrap_or("portail-dev")
                .to_string(),
            oidc_client_secret: option_env!("OIDC_CLIENT_SECRET")
                .unwrap_or("portail-dev-secret")
                .to_string(),
            oidc_redirect_uri: option_env!("OIDC_REDIRECT_URI")
                .unwrap_or("http://localhost:5173/callback")
                .to_string(),
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
            // Optional vars: standard LDAP infra defaults.
            ldap_people_ou: option_env!("LDAP_PEOPLE_OU")
                .unwrap_or("people")
                .to_string(),
            ldap_groups_ou: option_env!("LDAP_GROUPS_OU")
                .unwrap_or("groups")
                .to_string(),
            ldap_admin_pw: option_env!("LDAP_ADMIN_PW").unwrap_or("admin").to_string(),
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

    /// Set all required vars (except `secret`) to fixed values; secret override is
    /// set by the test itself. Returns the saved values to restore afterwards.
    fn save_and_set_all(secret: (&'static str, String)) -> Vec<(String, Option<String>)> {
        let names = [
            "OIDC_ISSUER_URL",
            "OIDC_CLIENT_ID",
            secret.0,
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
        for (k, v) in [
            ("OIDC_ISSUER_URL", "https://127.0.0.1:9091"),
            ("OIDC_CLIENT_ID", "portail-dev"),
            (secret.0, &secret.1),
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
            "OIDC_CLIENT_SECRET",
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
        );
        unsafe { std::env::remove_var("LDAP_PEOPLE_OU") };
        unsafe { std::env::remove_var("LDAP_GROUPS_OU") };
        unsafe { std::env::remove_var("LDAP_ADMIN_PW") };

        let env = super::Env::create().unwrap();

        assert_eq!(env.ldap_people_ou, "people", "default people OU expected");
        assert_eq!(env.ldap_groups_ou, "groups", "default groups OU expected");
        assert_eq!(env.ldap_admin_pw, "admin", "default admin pw expected");

        if let Some(v) = saved.0 {
            unsafe { std::env::set_var("LDAP_PEOPLE_OU", v) };
        }
        if let Some(v) = saved.1 {
            unsafe { std::env::set_var("LDAP_GROUPS_OU", v) };
        }
        if let Some(v) = saved.2 {
            unsafe { std::env::set_var("LDAP_ADMIN_PW", v) };
        }
    }

    #[test]
    fn optional_vars_custom_and_empty_falls_back() {
        let _g = LOCK.lock().unwrap();
        let saved = (
            std::env::var("LDAP_PEOPLE_OU").ok(),
            std::env::var("LDAP_GROUPS_OU").ok(),
        );
        unsafe { std::env::set_var("LDAP_PEOPLE_OU", "humains") };
        unsafe { std::env::set_var("LDAP_GROUPS_OU", "") };

        let env = super::Env::create().unwrap();

        assert_eq!(env.ldap_people_ou, "humains", "custom OU expected");
        assert_eq!(env.ldap_groups_ou, "groups", "empty should fall back");

        if let Some(v) = saved.0 {
            unsafe { std::env::set_var("LDAP_PEOPLE_OU", v) };
        } else {
            unsafe { std::env::remove_var("LDAP_PEOPLE_OU") };
        }
        if let Some(v) = saved.1 {
            unsafe { std::env::set_var("LDAP_GROUPS_OU", v) };
        } else {
            unsafe { std::env::remove_var("LDAP_GROUPS_OU") };
        }
    }

    #[test]
    fn secret_vars_read_file_content() {
        let _g = LOCK.lock().unwrap();
        let secret_file =
            std::env::temp_dir().join(format!("portail-oidc-secret-{}", std::process::id()));
        let pw_file = std::env::temp_dir().join(format!("portail-ldap-pw-{}", std::process::id()));
        std::fs::write(&secret_file, "portail-dev-secret\n").unwrap();
        std::fs::write(&pw_file, "p4ss\n").unwrap();
        let saved = save_and_set_all((
            "OIDC_CLIENT_SECRET",
            secret_file.to_str().unwrap().to_string(),
        ));
        unsafe { std::env::set_var("LDAP_ADMIN_PW", pw_file.to_str().unwrap()) };

        let env = super::Env::create().unwrap();

        assert_eq!(
            env.oidc_client_secret, "portail-dev-secret",
            "file content (newline trimmed) should be the secret"
        );
        assert_eq!(
            env.ldap_admin_pw, "p4ss",
            "optional secret file read expected"
        );

        restore_env(saved);
        let _ = std::fs::remove_file(&secret_file);
        let _ = std::fs::remove_file(&pw_file);
    }

    #[test]
    fn secret_file_error_is_reported() {
        let _g = LOCK.lock().unwrap();
        let missing_path = std::env::temp_dir().join("portail-no-such-secret-file");
        let saved = save_and_set_all((
            "OIDC_CLIENT_SECRET",
            missing_path.to_str().unwrap().to_string(),
        ));

        let err = super::Env::create().unwrap_err();

        assert!(
            err.0.contains("OIDC_CLIENT_SECRET") && err.0.contains("cannot read secret file"),
            "unreadable secret file should be reported: {err}"
        );

        restore_env(saved);
    }
}
