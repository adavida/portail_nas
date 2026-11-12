use portail_core::config::Config;

pub(crate) fn purge_db() {
    std::fs::remove_file(Config::global().db_path.as_str()).ok();
}
