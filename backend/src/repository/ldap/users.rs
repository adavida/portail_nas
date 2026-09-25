use ldap3::{Mod, Scope, SearchEntry};
use std::collections::HashSet;

use crate::{
    domain::users::{NewUser, Password, Uid, UpdateUser, User},
    error::AppError,
};

use super::{MapLdap, connect, ldap_base, people_search_base, user_dn};

fn one_set(v: String) -> HashSet<String> {
    [v].into_iter().collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_users() -> Result<Vec<User>, AppError> {
    let base = ldap_base();
    let search_base = people_search_base(&base);

    let mut ldap = connect(&base).await?;

    let (rs, _res) = ldap_conn_search(&mut ldap, &search_base).await?;
    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();
    let attrs_list: Vec<std::collections::HashMap<String, Vec<String>>> =
        entries.into_iter().map(|e| e.attrs).collect();
    let users = User::from_search(attrs_list);
    let _ = ldap.unbind().await;
    Ok(users)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn ldap_conn_search(
    ldap: &mut ldap3::Ldap,
    base: &str,
) -> Result<(Vec<ldap3::ResultEntry>, ldap3::LdapResult), AppError> {
    ldap.search(
        base,
        Scope::Subtree,
        "(objectClass=inetOrgPerson)",
        vec!["uid", "cn", "displayName", "mail"],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_user(new: NewUser) -> Result<User, AppError> {
    let base = ldap_base();
    let dn = user_dn(&new.uid, &base);
    let mut ldap = connect(&base).await?;

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
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user_password(uid: Uid, password: Password) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = user_dn(&uid, &base);
    let mut ldap = connect(&base).await?;

    ldap.modify(
        &dn,
        vec![Mod::Replace(
            "userPassword".to_string(),
            one_set(password.as_str().to_string()),
        )],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user(uid: Uid, data: UpdateUser) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = user_dn(&uid, &base);
    let mut ldap = connect(&base).await?;

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

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_user(uid: Uid) -> Result<(), AppError> {
    let base = ldap_base();
    let dn = user_dn(&uid, &base);
    let mut ldap = connect(&base).await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn authenticate_user(uid: Uid, password: Password) -> Result<bool, AppError> {
    let base = ldap_base();
    let dn = user_dn(&uid, &base);

    let mut ldap = connect(&base).await?;

    let res = ldap.simple_bind(&dn, password.as_str()).await.map_ldap()?;
    let rc = res.rc;
    let _ = ldap.unbind().await;
    match rc {
        0 => Ok(true),
        49 => Ok(false),
        _ => Err(AppError::Ldap(format!("bind rc {rc}: {}", res.text))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::{Email, Name};
    use crate::repository::ldap::ldap_url;
    use ldap3::LdapConnAsync;
    use std::time::{SystemTime, UNIX_EPOCH};

    // user_dn_from is for tests only — the general import would create an
    // unused_imports warning in non-test builds.
    use crate::repository::ldap::user_dn_from;

    fn gen_uid(prefix: &str) -> String {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1000000;
        format!("{prefix}{n}")
    }

    fn new_user(uid: &str, name: &str, email: &str, password: &str) -> NewUser {
        NewUser {
            uid: Uid::try_new(uid.to_string()).unwrap(),
            name: Name::try_new(name.into()).unwrap(),
            email: Email::try_new(email.into()).unwrap(),
            password: Password::try_new(password.into()).unwrap(),
        }
    }

    async fn ensure_clean(uid: &str) {
        let url = ldap_url();
        let base = ldap_base();
        if let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await {
            ldap3::drive!(conn);
            let bind_dn = format!("cn=admin,{base}");
            if ldap.simple_bind(&bind_dn, "admin").await.is_ok() {
                let dn = user_dn_from(uid, &base);
                let _ = ldap.delete(&dn).await;
                let _ = ldap.unbind().await;
            }
        }
    }

    fn is_ldap_down<T>(r: &Result<T, AppError>) -> bool {
        matches!(r, Err(AppError::Ldap(_)))
    }

    async fn seed_and_create_user(uid: &str, name: &str, email: &str, password: &str) -> bool {
        ensure_clean(uid).await;

        let new = new_user(uid, name, email, password);

        let r = create_user(new).await;
        if is_ldap_down(&r) {
            return false;
        }

        r.unwrap();
        true
    }

    async fn assert_auth_ok(uid: &str, password: &str) {
        let ok = authenticate_user(
            Uid::try_new(uid.to_string()).unwrap(),
            Password::try_new(password.to_string()).unwrap(),
        )
        .await
        .unwrap();

        assert!(
            ok,
            "authenticate should succeed for uid={uid} with correct password"
        );
    }

    async fn assert_auth_fail(uid: &str, password: &str) {
        let ok = authenticate_user(
            Uid::try_new(uid.to_string()).unwrap(),
            Password::try_new(password.to_string()).unwrap(),
        )
        .await
        .unwrap();

        assert!(
            !ok,
            "authenticate should fail for uid={uid} with wrong password"
        );
    }

    async fn assert_auth_err(uid: &str, password: &str) {
        let uid_res = Uid::try_new(uid.to_string());
        let pw_res = Password::try_new(password.to_string());

        let result = match (uid_res, pw_res) {
            (Ok(u), Ok(p)) => authenticate_user(u, p).await,
            _ => Err(AppError::Internal("missing uid/password".into())),
        };

        assert!(
            result.is_err(),
            "authenticate should error for uid={uid} with empty password"
        );
    }

    #[tokio::test]
    async fn update_user_changes_name_and_email() {
        let uid = gen_uid("upd");
        if !seed_and_create_user(&uid, "Old Name", "old@ex.com", "pw").await {
            return;
        }

        let req: UpdateUser =
            serde_json::from_str(r#"{"name":"New Name","email":"new@ex.com"}"#).unwrap();

        update_user(Uid::try_new(uid.clone()).unwrap(), req)
            .await
            .unwrap();

        let users = list_users().await.unwrap();
        let updated = users.iter().find(|u| u.uid.as_str() == uid);

        assert!(
            updated.is_some(),
            "user {uid} should still exist after update"
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
    async fn delete_user_removes_entry() {
        let uid = gen_uid("del");
        if !seed_and_create_user(&uid, "Delete Me", "", "pw").await {
            return;
        }

        delete_user(Uid::try_new(uid.clone()).unwrap())
            .await
            .unwrap();

        let again = delete_user(Uid::try_new(uid.clone()).unwrap()).await;

        assert!(
            again.is_err(),
            "second delete should fail: entry no longer exists"
        );

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn authenticate_ok_and_wrong() {
        let uid = gen_uid("auth");
        if !seed_and_create_user(&uid, "Auth Test", "a@ex.com", "secret123").await {
            return;
        }

        assert_auth_ok(&uid, "secret123").await;
        assert_auth_fail(&uid, "wrong").await;
        assert_auth_err(&uid, "").await;

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn authenticate_after_password_update() {
        let uid = gen_uid("auth2");
        if !seed_and_create_user(&uid, "T", "", "old").await {
            return;
        }

        update_user_password(
            Uid::try_new(uid.clone()).unwrap(),
            Password::try_new("newpass".into()).unwrap(),
        )
        .await
        .unwrap();

        assert_auth_fail(&uid, "old").await;
        assert_auth_ok(&uid, "newpass").await;

        ensure_clean(&uid).await;
    }
}
