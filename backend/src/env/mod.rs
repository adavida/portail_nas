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
}

static ENV: OnceLock<Env> = OnceLock::new();

impl Env {
    /// Reads all required vars from `std::env`; collects every missing/empty
    /// name and returns a single error listing them all.
    pub fn create() -> Result<Self, EnvError> {
        let mut missing: Vec<&'static str> = Vec::new();

        let take = |name: &'static str, missing: &mut Vec<&'static str>| -> String {
            match std::env::var(name) {
                Ok(v) if !v.trim().is_empty() => v,
                _ => {
                    missing.push(name);
                    String::new()
                }
            }
        };

        let oidc_issuer_url = take("OIDC_ISSUER_URL", &mut missing);
        let oidc_client_id = take("OIDC_CLIENT_ID", &mut missing);
        let oidc_client_secret = take("OIDC_CLIENT_SECRET", &mut missing);
        let oidc_redirect_uri = take("OIDC_REDIRECT_URI", &mut missing);
        let app_url = take("APP_URL", &mut missing);
        let bind_addr = take("BIND_ADDR", &mut missing);
        let ldap_url = take("LDAP_URL", &mut missing);
        let ldap_base_dn = take("LDAP_BASE_DN", &mut missing);

        if !missing.is_empty() {
            return Err(EnvError(format!(
                "missing or empty env vars: {}. set them in devenv.nix env",
                missing.join(", ")
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
        })
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
}
