use crate::domain::groups::UpdateGroup;
use crate::domain::ldap::Gid;
use crate::repository::group_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_group(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let rows = portail_db::groups::update(
        &connection,
        gid.as_str(),
        data.name.as_str(),
        data.description.as_str(),
    )
    .map_err(AppError::Db)?;
    if rows == 0 {
        return Err(group_not_found(&gid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ldap::{Description, Name};
    use crate::repository::sqlite::groups::{create_group, delete_group, list_groups};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn update_group_changes_name_and_clears_description() {
        let uid = gen_uid("sugm");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("sug");
        create_group(new_group(&gid, std::slice::from_ref(&uid)))
            .await
            .unwrap();

        update_group(
            gid.clone(),
            UpdateGroup {
                name: Name::try_new("Renamed".into()).unwrap(),
                description: Description::try_new(String::new()),
            },
        )
        .await
        .unwrap();

        let groups = list_groups().await.unwrap();
        let updated = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should still exist");
        assert_eq!(updated.name.as_str(), "Renamed", "name should be persisted");
        assert_eq!(
            updated.description.as_str(),
            "",
            "empty description should clear the stored one"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn update_group_fails_for_missing_gid() {
        let gid = gen_gid("sugko");

        let result = update_group(
            gid.clone(),
            UpdateGroup {
                name: Name::try_new("Ghost".into()).unwrap(),
                description: Description::try_new(String::new()),
            },
        )
        .await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing gid should be NotFound, got {result:?}"
        );
    }
}
