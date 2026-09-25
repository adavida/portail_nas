use crate::{
    domain::users::{NewUser, User},
    error::AppError,
    repository::ldap::{MapLdap, connect, user_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_user(new: NewUser) -> Result<User, AppError> {
    let dn = user_dn(&new.uid);
    let mut ldap = connect().await?;

    ldap.add(&dn, new.to_attrs())
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let _ = ldap.unbind().await;

    Ok(User {
        uid: new.uid,
        name: new.name,
        email: new.email,
        groups: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ldap::list_users;
    use crate::test_helpers::seed::{ensure_clean, gen_uid, new_user, seed_and_create_user};

    #[tokio::test]
    async fn create_user_persists_entry() {
        let uid = gen_uid("cre");
        ensure_clean(&uid).await;

        let new = new_user(&uid, "Create Test", "c@ex.com", "pw");
        let created = create_user(new).await;

        // LDAP indisponible => skip
        let created = match created {
            Ok(u) => u,
            Err(AppError::Ldap(_)) => return,
            Err(e) => panic!("create_user should succeed, got {e}"),
        };

        assert_eq!(
            created.uid.as_str(),
            uid.as_str(),
            "created uid should match request"
        );
        assert_eq!(
            created.name.as_str(),
            "Create Test",
            "created name should match request"
        );

        let users = list_users().await.unwrap();
        let found = users.iter().find(|u| u.uid.as_str() == uid.as_str());

        assert!(
            found.is_some(),
            "created uid should be visible in list_users"
        );
        assert_eq!(
            found.unwrap().email.as_str(),
            "c@ex.com",
            "persisted email should match request"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn create_user_fails_when_uid_already_exists() {
        let uid = gen_uid("credup");
        if !seed_and_create_user(&uid, "First", "", "pw").await {
            return;
        }

        let again = create_user(new_user(&uid, "Second", "", "pw")).await;

        assert!(again.is_err(), "duplicate uid should be rejected by LDAP");

        ensure_clean(&uid).await;
    }
}
