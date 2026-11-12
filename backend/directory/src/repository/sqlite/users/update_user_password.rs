use crate::domain::ldap::{Password, Uid};
use crate::repository::sqlite;
use crate::repository::user_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user_password(uid: &Uid, password: Password) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let password_hash = sqlite::hash_password(password.as_str())?;
    let rows = portail_db::users::set_password_hash(&connection, uid.as_str(), &password_hash)
        .map_err(AppError::Db)?;
    if rows == 0 {
        return Err(user_not_found(uid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{authenticate_user, create_user, delete_user};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn update_user_password_replaces_credentials() {
        let uid = gen_uid("supw");
        create_user(new_user(&uid, "T", "", "oldpass"))
            .await
            .unwrap();

        update_user_password(&uid, Password::try_new("newpass".into()).unwrap())
            .await
            .unwrap();

        assert!(
            !authenticate_user(
                &uid,
                Password::for_authentication("oldpass".into()).unwrap()
            )
            .await
            .unwrap(),
            "old password should stop working"
        );
        assert!(
            authenticate_user(
                &uid,
                Password::for_authentication("newpass".into()).unwrap()
            )
            .await
            .unwrap(),
            "new password should work"
        );

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn update_user_password_fails_for_missing_uid() {
        let uid = gen_uid("supwnx");

        let result =
            update_user_password(&uid, Password::try_new("whatever".into()).unwrap()).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing uid should be NotFound, got {result:?}"
        );
    }
}
