use crate::{
    domain::{ldap::Uid, users::UserAttr},
    repository::user_not_found,
};
use portail_core::{config::Config, error::AppError};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn set_user_attrs(uid: &Uid, attrs: Vec<UserAttr>) -> Result<(), AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    if !portail_db::users::exists(&connection, uid.as_str()).map_err(AppError::Db)? {
        return Err(user_not_found(uid));
    }
    for attr in attrs {
        if attr.is_empty() {
            portail_db::attrs::delete_attr(&connection, uid.as_str(), &attr.label)
                .map_err(AppError::Db)?;
        } else {
            portail_db::attrs::set_attr(&connection, uid.as_str(), &attr.label, &attr.value)
                .map_err(AppError::Db)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user, list_user_attrs};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn set_user_attrs_writes_replaces_and_deletes() {
        let uid = gen_uid("ssattr");
        create_user(new_user(&uid, "Attr Set", "as@ex.com", "pw"))
            .await
            .unwrap();
        set_user_attrs(
            &uid,
            vec![UserAttr::try_new("city".into(), "Nice".into()).unwrap()],
        )
        .await
        .unwrap();

        set_user_attrs(
            &uid,
            vec![
                UserAttr::try_new("age".into(), "20".into()).unwrap(),
                UserAttr::try_new("city".into(), "".into()).unwrap(),
            ],
        )
        .await
        .unwrap();
        let attrs = list_user_attrs(&uid).await.unwrap();

        assert_eq!(
            attrs,
            vec![UserAttr::from_stored("age".into(), "20".into())],
            "an empty value must delete the row, a new label must be inserted"
        );

        let _ = delete_user(&uid).await;
    }

    #[tokio::test]
    async fn set_user_attrs_fails_for_missing_uid() {
        let uid = gen_uid("ssattnx");

        let result = set_user_attrs(&uid, vec![]).await;

        assert!(
            matches!(result, Err(AppError::NotFound(_))),
            "writing attrs of a missing uid should be NotFound, got {result:?}"
        );
    }
}
