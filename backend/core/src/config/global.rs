use super::Config;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn set(config: Config) -> &'static Config {
        let _ = CONFIG.set(config);
        CONFIG.get().unwrap()
    }

    pub fn global() -> &'static Config {
        CONFIG
            .get()
            .expect("config not loaded: call Config::set in main")
    }

    pub fn ensure_init(path: &str) {
        if CONFIG.get().is_none() {
            let config = Self::load(path).unwrap_or_else(|e| panic!("{e}"));
            let _ = CONFIG.set(config);
        }
    }
}
