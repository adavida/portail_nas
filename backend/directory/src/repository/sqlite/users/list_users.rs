use crate::domain::users::User;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_users() -> Result<Vec<User>, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    portail_db::users::find_all(&connection)
        .map_err(AppError::Db)
        .map(|users| {
            users
                .into_iter()
                .filter_map(|record| {
                    User::from_attrs(
                        Some(record.uid),
                        None,
                        Some(record.name),
                        Some(record.email),
                    )
                    .ok()
                })
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn list_users_contains_created_user_sorted_by_uid() {
        let uid = gen_uid("slst");
        create_user(new_user(&uid, "Sort List", "s@ex.com", "pw"))
            .await
            .unwrap();

        let users = list_users().await.unwrap();
        let found = users.iter().find(|u| u.uid.as_str() == uid.as_str());

        assert!(found.is_some(), "created uid should be listed");
        assert_eq!(
            found.unwrap().email.as_str(),
            "s@ex.com",
            "listed email should match"
        );
        assert!(
            users
                .windows(2)
                .all(|w| w[0].uid.as_str() <= w[1].uid.as_str()),
            "list should be sorted by uid"
        );

        let _ = delete_user(&uid).await;
    }
}
