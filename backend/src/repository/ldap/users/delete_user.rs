use crate::{
    domain::users::Uid,
    error::AppError,
    repository::ldap::{MapLdap, connect, find_user_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_user(uid: &Uid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_user_dn(&mut ldap, uid).await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};

    #[tokio::test]
    async fn delete_user_removes_entry() {
        let uid = gen_uid("del");
        if !seed_and_create_user(&uid, "Delete Me", "", "pw").await {
            return;
        }

        delete_user(&uid).await.unwrap();

        let again = delete_user(&uid).await;

        assert!(
            again.is_err(),
            "second delete should fail: entry no longer exists"
        );

        ensure_clean(&uid).await;
    }
}
