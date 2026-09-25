use ldap3::Mod;
use std::collections::HashSet;

use crate::{
    domain::users::{Uid, UpdateUser},
    error::AppError,
    repository::ldap::{MapLdap, connect, find_user_dn},
};

use super::one_set;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user(uid: &Uid, data: UpdateUser) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_user_dn(&mut ldap, uid).await?;

    let sn = data
        .name
        .as_str()
        .split_whitespace()
        .last()
        .unwrap_or(data.name.as_str())
        .to_string();
    let set_name = one_set(data.name.as_str().to_string());
    let set_mail = if data.email.as_str().is_empty() {
        HashSet::new()
    } else {
        one_set(data.email.as_str().to_string())
    };

    ldap.modify(
        &dn,
        vec![
            Mod::Replace("cn".to_string(), set_name.clone()),
            Mod::Replace("displayName".to_string(), set_name),
            Mod::Replace("sn".to_string(), one_set(sn)),
            Mod::Replace("mail".to_string(), set_mail),
        ],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::ldap::list_users;
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};

    #[tokio::test]
    async fn update_user_changes_name_and_email() {
        let uid = gen_uid("upd");
        if !seed_and_create_user(&uid, "Old Name", "old@ex.com", "pw").await {
            return;
        }

        let req: UpdateUser =
            serde_json::from_str(r#"{"name":"New Name","email":"new@ex.com"}"#).unwrap();

        update_user(&uid, req).await.unwrap();

        let users = list_users().await.unwrap();
        let updated = users.iter().find(|u| u.uid.as_str() == uid.as_str());

        assert!(
            updated.is_some(),
            "user {} should still exist after update",
            uid.as_str()
        );

        let updated = updated.unwrap();

        assert_eq!(updated.name.as_str(), "New Name", "name should be updated");
        assert_eq!(
            updated.email.as_str(),
            "new@ex.com",
            "email should be updated"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn update_user_clears_email_when_empty() {
        let uid = gen_uid("updem");
        if !seed_and_create_user(&uid, "Has Email", "e@ex.com", "pw").await {
            return;
        }

        let req: UpdateUser = serde_json::from_str(r#"{"name":"Has Email","email":""}"#).unwrap();

        update_user(&uid, req).await.unwrap();

        let users = list_users().await.unwrap();
        let found = users
            .iter()
            .find(|u| u.uid.as_str() == uid.as_str())
            .unwrap();

        assert_eq!(
            found.email.as_str(),
            "",
            "empty email should clear the mail attribute"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn update_user_fails_for_missing_uid() {
        let uid = gen_uid("updnx");
        if list_users().await.is_err() {
            return; // LDAP indisponible => skip
        }

        let req: UpdateUser = serde_json::from_str(r#"{"name":"Ghost","email":""}"#).unwrap();

        let result = update_user(&uid, req).await;

        assert!(
            result.is_err(),
            "updating a missing uid should fail, got {result:?}"
        );
    }
}
