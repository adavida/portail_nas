use crate::domain::groups::{Group, NewGroup};
use crate::repository::user_not_found;
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_group(new: NewGroup) -> Result<Group, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;

    for uid in new.members.as_slice() {
        if !portail_db::users::exists(&connection, uid.as_str()).map_err(AppError::Db)? {
            return Err(user_not_found(uid));
        }
    }

    portail_db::groups::insert(
        &connection,
        new.gid.as_str(),
        new.name.as_str(),
        new.description.as_str(),
    )
    .map_err(AppError::Db)?;
    for uid in new.members.as_slice() {
        portail_db::groups::add_member(&connection, new.gid.as_str(), uid.as_str())
            .map_err(AppError::Db)?;
    }

    Ok(Group {
        gid: new.gid,
        name: new.name,
        description: new.description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ldap::{Description, Members, Name};
    use crate::repository::sqlite::groups::{delete_group, list_uids};
    use crate::repository::sqlite::users::{create_user, delete_user};
    use crate::test_helpers::seed::{gen_gid, gen_uid, new_group, new_user};

    #[tokio::test]
    async fn create_group_persists_members() {
        let uid = gen_uid("scgm");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("scg");
        let new = new_group(&gid, std::slice::from_ref(&uid));

        let created = create_group(new.clone()).await.unwrap();

        assert_eq!(created.gid, new.gid, "created group should echo gid");
        let groups = list_uids().await.unwrap();
        let group = groups
            .iter()
            .find(|g| g.gid == gid)
            .expect("group should be listed");
        assert_eq!(
            group.members,
            new.members.as_slice().to_vec(),
            "members should be persisted"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn create_group_fails_for_missing_member() {
        let ghost = gen_uid("scgghost");
        let new = NewGroup {
            gid: gen_gid("scgko"),
            name: Name::try_new("Ghost Group".into()).unwrap(),
            description: Description::try_new(String::new()),
            members: Members::try_new(vec![ghost]).unwrap(),
        };

        let result = create_group(new).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "missing member should be NotFound, got {result:?}"
        );
    }

    #[tokio::test]
    async fn create_group_fails_on_duplicate_gid() {
        let uid = gen_uid("scgdm");
        create_user(new_user(&uid, "Member", "", "pw"))
            .await
            .unwrap();
        let gid = gen_gid("scgd");
        let new = new_group(&gid, std::slice::from_ref(&uid));
        create_group(new.clone()).await.unwrap();

        let again = create_group(new.clone()).await;

        assert!(
            matches!(again, Err(AppError::Db(_))),
            "duplicate gid should be rejected, got {again:?}"
        );

        let _ = delete_group(gid).await;
        let _ = delete_user(&uid).await;
    }
}
