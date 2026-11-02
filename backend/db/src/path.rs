use std::path::PathBuf;

use portail_core::config::Config;

pub fn portal_db_path(config: &Config) -> PathBuf {
    PathBuf::from(&config.db_path)
}
