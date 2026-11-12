use crate::domain::ldap::Uid;
use crate::domain::users::UpdateUser;
use crate::repository::user_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user(uid: &Uid, data: UpdateUser) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let rows = portail_db::users::update_profile(
        &connection,
        uid.as_str(),
        data.name.as_str(),
        data.email.as_str(),
    )
    .map_err(AppError::Db)?;
    if rows == 0 {
        return Err(user_not_found(uid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user, list_users};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn update_user_changes_name_and_email() {
        let uid = gen_uid("supd");
        create_user(new_user(&uid, "Old Name", "old@ex.com", "pw"))
            .await
            .unwrap();
        let req: UpdateUser =
            serde_json::from_str(r#"{"name":"New Name","email":"new@ex.com"}"#).unwrap();

        update_user(&uid, req).await.unwrap();

        let users = list_users().await.unwrap();
        let updated = users
            .iter()
            .find(|u| u.uid.as_str() == uid.as_str())
            .unwrap();

        assert_eq!(updated.name.as_str(), "New Name", "name should be updated");
        assert_eq!(
            updated.email.as_str(),
            "new@ex.com",
            "email should be updated"
        );

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn update_user_fails_for_missing_uid() {
        let uid = gen_uid("supdnx");
        let req: UpdateUser = serde_json::from_str(r#"{"name":"Ghost","email":""}"#).unwrap();

        let result = update_user(&uid, req).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "updating a missing uid should be NotFound, got {result:?}"
        );
    }
}
