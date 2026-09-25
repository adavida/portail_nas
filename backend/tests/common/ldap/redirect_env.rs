pub fn redirect_ldap_env_to_test() {
    std::env::set_var(
        "LDAP_URL",
        std::env::var("LDAP_TEST_URL").unwrap_or_else(|_| "ldap://127.0.0.1:3891".into()),
    );
    std::env::set_var(
        "LDAP_BASE_DN",
        std::env::var("LDAP_TEST_BASE_DN").unwrap_or_else(|_| "dc=test,dc=example,dc=com".into()),
    );
}

#[ctor::ctor(unsafe)]
fn redirect_ldap_env() {
    redirect_ldap_env_to_test();
}
