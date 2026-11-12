use crate::domain::users::{NewUser, User};
use crate::repository::sqlite;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_user(new: NewUser) -> Result<User, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let password_hash = sqlite::hash_password(new.password.as_str())?;

    portail_db::users::insert(
        &connection,
        new.uid.as_str(),
        new.name.as_str(),
        new.email.as_str(),
        &password_hash,
    )
    .map_err(AppError::Db)?;

    Ok(User {
        uid: new.uid,
        name: new.name,
        email: new.email,
        groups: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{delete_user, list_users};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn create_user_persists_entry() {
        let uid = gen_uid("scre");

        let created = create_user(new_user(&uid, "Sqlite Create", "c@ex.com", "pw"))
            .await
            .unwrap();

        assert_eq!(
            created.uid.as_str(),
            uid.as_str(),
            "created uid should match request"
        );
        assert!(
            list_users()
                .await
                .unwrap()
                .iter()
                .any(|u| u.uid == created.uid),
            "created uid should be visible in list_users"
        );

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn create_user_fails_when_uid_already_exists() {
        let uid = gen_uid("scredup");
        create_user(new_user(&uid, "First", "", "pw"))
            .await
            .unwrap();

        let again = create_user(new_user(&uid, "Second", "", "pw")).await;

        assert!(
            matches!(again, Err(AppError::Db(_))),
            "duplicate uid should be rejected, got {again:?}"
        );

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn create_user_never_stores_plaintext_password() {
        let uid = gen_uid("schash");

        create_user(new_user(&uid, "Hashed", "", "plaintext1"))
            .await
            .unwrap();
        let connection = portail_db::connect(Config::global()).unwrap();
        let hash = portail_db::users::find_password_hash(&connection, uid.as_str())
            .unwrap()
            .expect("hash should be stored");

        assert!(
            hash.starts_with("$argon2") && hash != "plaintext1",
            "password must be hashed, got {hash}"
        );

        let _ = delete_user(&uid).await;
    }
}
