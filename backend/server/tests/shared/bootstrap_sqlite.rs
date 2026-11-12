use crate::common::config::set_test_config_sqlite::set_test_config_sqlite;
use crate::common::db::purge::purge_db;

#[ctor::ctor(unsafe)]
fn init_sqlite_suite() {
    set_test_config_sqlite();
    purge_db();
}
