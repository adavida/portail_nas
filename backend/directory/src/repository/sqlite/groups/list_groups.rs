use crate::domain::groups::Group;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_groups() -> Result<Vec<Group>, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    portail_db::groups::find_all(&connection)
        .map_err(AppError::Db)
        .map(|records| {
            records
                .into_iter()
                .filter_map(|record| {
                    Group::from_attrs(
                        Some(record.gid),
                        Some(record.name),
                        Some(record.description),
                    )
                    .ok()
                })
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::groups::{create_group, delete_group};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn list_groups_contains_created_group() {
        let uid = gen_uid("slgm");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("slg");
        let new = new_group(&gid, std::slice::from_ref(&uid));

        create_group(new.clone()).await.unwrap();

        let groups = list_groups().await.unwrap();
        let found = groups.iter().find(|g| g.gid == gid);

        assert!(found.is_some(), "created gid should be listed");
        assert_eq!(
            found.unwrap().name.as_str(),
            "Seed Group",
            "listed name should match"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&uid).await;
    }
}
