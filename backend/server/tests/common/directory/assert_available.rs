use portail_core::error::AppError;
use portail_directory::repository::list_users;

pub(crate) async fn assert_available() {
    match list_users().await {
        Ok(_) => {}
        Err(AppError::Ldap(message)) => {
            panic!(
                "ldap directory backend unavailable, run devenv up with openldap-test: {message}"
            )
        }
        Err(error) => panic!("directory backend unavailable: {error}"),
    }
}
