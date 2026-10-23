#[ctor::ctor(unsafe)]
fn redirect_ldap_to_test() {
    std::env::set_var("LDAP_URL", portail_core::env::ldap_test_url());
    std::env::set_var("LDAP_BASE_DN", portail_core::env::ldap_test_base_dn());
    portail_core::env::Env::ensure_init();
}
