use portail_core::config::Config;

pub(crate) const TEST_CONFIG_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../config.test.toml");

pub(crate) fn set_test_config() {
    Config::ensure_init(TEST_CONFIG_PATH);
}
