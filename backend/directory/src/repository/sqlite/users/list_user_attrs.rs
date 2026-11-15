use crate::domain::{ldap::Uid, users::UserAttr};
use portail_core::config::Config;
use portail_core::error::AppError;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_user_attrs(uid: &Uid) -> Result<Vec<UserAttr>, AppError> {
    let connection = portail_db::connect(Config::global()).map_err(AppError::Db)?;
    let pairs = portail_db::attrs::list_attrs(&connection, uid.as_str()).map_err(AppError::Db)?;
    Ok(pairs
        .into_iter()
        .map(|(label, value)| UserAttr::from_stored(label, value))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite::users::{create_user, delete_user, set_user_attrs};
    use crate::test_helpers::seed::{gen_uid, new_user};

    #[tokio::test]
    async fn list_user_attrs_reflects_written_attrs() {
        let uid = gen_uid("slattr");
        create_user(new_user(&uid, "Attr List", "al@ex.com", "pw"))
            .await
            .unwrap();
        set_user_attrs(
            &uid,
            vec![
                UserAttr::try_new("city".into(), "Nice".into()).unwrap(),
                UserAttr::try_new("age".into(), "30".into()).unwrap(),
            ],
        )
        .await
        .unwrap();

        let attrs = list_user_attrs(&uid).await.unwrap();

        assert_eq!(
            attrs,
            vec![
                UserAttr::from_stored("age".into(), "30".into()),
                UserAttr::from_stored("city".into(), "Nice".into()),
            ],
            "stored attrs must come back sorted by label"
        );

        let _ = delete_user(&uid).await;
    }
}
