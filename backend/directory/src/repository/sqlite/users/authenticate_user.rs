use crate::domain::ldap::{Password, Uid};
use crate::repository::sqlite;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn authenticate_user(uid: &Uid, password: Password) -> Result<bool, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let stored =
        portail_db::users::find_password_hash(&connection, uid.as_str()).map_err(AppError::Db)?;
    Ok(stored.is_some_and(|hash| sqlite::verify_password(&hash, password.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn authenticate_ok_and_wrong() {
        let uid = gen_uid("sauth");
        create_user(new_user(&uid, "Auth Test", "a@ex.com", "secret123"))
            .await
            .unwrap();

        let ok = authenticate_user(
            &uid,
            Password::for_authentication("secret123".into()).unwrap(),
        )
        .await
        .unwrap();
        let wrong = authenticate_user(&uid, Password::for_authentication("wrong".into()).unwrap())
            .await
            .unwrap();

        assert!(ok, "correct password should authenticate");
        assert!(!wrong, "wrong password should not authenticate");

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn authenticate_missing_uid_is_false() {
        let uid = gen_uid("sauthko");

        let result = authenticate_user(
            &uid,
            Password::for_authentication("anything".into()).unwrap(),
        )
        .await
        .unwrap();

        assert!(!result, "unknown uid should never authenticate");
    }
}
