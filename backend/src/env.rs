//! Centralized environment helpers — single source of truth for all env vars.
//!
//! Each var is `const` via `option_env!().expect` so a missing var fails the
//! build with an English message (no `unwrap_or` fallback with hard-coded URL).

#![allow(clippy::option_env_unwrap)]

pub const OIDC_ISSUER_URL: &str = option_env!("OIDC_ISSUER_URL")
    .expect("OIDC_ISSUER_URL missing: set OIDC_ISSUER_URL in devenv.nix env");
pub const OIDC_CLIENT_ID: &str = option_env!("OIDC_CLIENT_ID")
    .expect("OIDC_CLIENT_ID missing: set OIDC_CLIENT_ID in devenv.nix env");
pub const OIDC_CLIENT_SECRET: &str = option_env!("OIDC_CLIENT_SECRET")
    .expect("OIDC_CLIENT_SECRET missing: set OIDC_CLIENT_SECRET in devenv.nix env");
pub const OIDC_REDIRECT_URI: &str = option_env!("OIDC_REDIRECT_URI")
    .expect("OIDC_REDIRECT_URI missing: set OIDC_REDIRECT_URI in devenv.nix env");
pub const APP_URL: &str =
    option_env!("APP_URL").expect("APP_URL missing: set APP_URL in devenv.nix env");
pub const BIND_ADDR: &str =
    option_env!("BIND_ADDR").expect("BIND_ADDR missing: set BIND_ADDR in devenv.nix env");
pub fn ldap_url() -> String {
    std::env::var("LDAP_URL").expect("LDAP_URL missing: set LDAP_URL in devenv.nix env")
}
pub fn ldap_base_dn() -> String {
    std::env::var("LDAP_BASE_DN").expect("LDAP_BASE_DN missing: set LDAP_BASE_DN in devenv.nix env")
}
pub fn ldap_test_url() -> String {
    std::env::var("LDAP_TEST_URL")
        .expect("LDAP_TEST_URL missing: set LDAP_TEST_URL in devenv.nix env")
}
pub fn ldap_test_base_dn() -> String {
    std::env::var("LDAP_TEST_BASE_DN")
        .expect("LDAP_TEST_BASE_DN missing: set LDAP_TEST_BASE_DN in devenv.nix env")
}

pub fn oidc_issuer_url() -> String {
    OIDC_ISSUER_URL.to_string()
}
pub fn oidc_client_id() -> String {
    OIDC_CLIENT_ID.to_string()
}
pub fn oidc_client_secret() -> String {
    OIDC_CLIENT_SECRET.to_string()
}
pub fn oidc_redirect_uri() -> String {
    OIDC_REDIRECT_URI.to_string()
}
pub fn app_url() -> String {
    APP_URL.to_string()
}
pub fn bind_addr() -> String {
    BIND_ADDR.to_string()
}
