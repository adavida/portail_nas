use crate::common::config::set_test_config_ldap::set_test_config_ldap;
use crate::common::db::purge::purge_db;
use crate::common::ldap::purge::purge_test_ldap;

#[ctor::ctor(unsafe)]
fn init_ldap_suite() {
    set_test_config_ldap();
    purge_db();
    purge_test_ldap();
}
