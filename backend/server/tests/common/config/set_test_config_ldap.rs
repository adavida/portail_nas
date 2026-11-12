use portail_core::config::Config;

pub(crate) const TEST_CONFIG_LDAP_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../config.test.ldap.toml");

pub(crate) fn set_test_config_ldap() {
    Config::ensure_init(TEST_CONFIG_LDAP_PATH);
}
