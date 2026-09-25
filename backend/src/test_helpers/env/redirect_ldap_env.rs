#[ctor::ctor(unsafe)]
fn redirect_ldap_to_test() {
    std::env::set_var("LDAP_URL", crate::env::ldap_test_url());
    std::env::set_var("LDAP_BASE_DN", crate::env::ldap_test_base_dn());
    crate::env::Env::ensure_init();
}
