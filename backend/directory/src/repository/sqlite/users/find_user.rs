use crate::domain::ldap::Uid;
use crate::repository::user_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn find_user(uid: &Uid) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    if !portail_db::users::exists(&connection, uid.as_str()).map_err(AppError::Db)? {
        return Err(user_not_found(uid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn find_user_ok_for_created_user() {
        let uid = gen_uid("sfnok");
        create_user(new_user(&uid, "Find OK", "", "pw"))
            .await
            .unwrap();

        let found = find_user(&uid).await;

        assert!(found.is_ok(), "created uid should be found");

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn find_user_not_found_for_missing_uid() {
        let uid = gen_uid("sfnko");

        let result = find_user(&uid).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing uid should be NotFound, got {result:?}"
        );
    }
}
