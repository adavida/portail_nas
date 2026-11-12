use crate::domain::ldap::Gid;
use crate::repository::group_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let rows = portail_db::groups::delete(&connection, gid.as_str()).map_err(AppError::Db)?;
    if rows == 0 {
        return Err(group_not_found(&gid));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::groups::{create_group, list_groups};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn delete_group_removes_entry() {
        let uid = gen_uid("sdgm");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("sdg");
        create_group(new_group(&gid, std::slice::from_ref(&uid)))
            .await
            .unwrap();

        delete_group(gid.clone()).await.unwrap();

        let again = delete_group(gid.clone()).await;
        assert!(
            matches!(again, Err(AppError::NotFound(_))),
            "second delete should be NotFound, got {again:?}"
        );
        let groups = list_groups().await.unwrap();
        assert!(
            !groups.iter().any(|g| g.gid == gid),
            "deleted group should be gone"
        );

        let _ = delete_user(&uid).await;
    }
}
