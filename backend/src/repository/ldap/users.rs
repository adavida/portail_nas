use ldap3::{LdapConnAsync, Mod, Scope, SearchEntry};
use std::collections::HashSet;

use crate::{
    domain::users::{NewUser, Password, Uid, User},
    error::AppError,
};

use super::{bind_admin, ldap_base, ldap_url};

pub async fn list_users() -> Result<Vec<User>, AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let search_base = format!("ou=people,{base}");

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);

    let (rs, _res) = ldap
        .search(
            &search_base,
            Scope::Subtree,
            "(objectClass=inetOrgPerson)",
            vec!["uid", "cn", "displayName", "mail"],
        )
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;

    let entries: Vec<SearchEntry> = rs.into_iter().map(SearchEntry::construct).collect();
    let attrs_list: Vec<std::collections::HashMap<String, Vec<String>>> =
        entries.into_iter().map(|e| e.attrs).collect();
    let users = User::from_search(attrs_list);
    let _ = ldap.unbind().await;
    Ok(users)
}

pub async fn create_user(new: NewUser) -> Result<User, AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let dn = new.dn(&base);

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);
    bind_admin(&mut ldap, &base).await?;

    let attrs = new.to_attrs();
    ldap.add(&dn, attrs)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    let _ = ldap.unbind().await;

    Ok(User {
        uid: new.uid,
        name: new.name,
        email: new.email,
    })
}

pub async fn update_user_password(uid: Uid, password: Password) -> Result<(), AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let dn = format!("uid={},ou=people,{base}", uid.as_str());

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);
    bind_admin(&mut ldap, &base).await?;

    let mut set = HashSet::new();
    set.insert(password.as_str().to_string());
    ldap.modify(&dn, vec![Mod::Replace("userPassword".to_string(), set)])
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    let _ = ldap.unbind().await;
    Ok(())
}

pub async fn delete_user(uid: Uid) -> Result<(), AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let dn = format!("uid={},ou=people,{base}", uid.as_str());

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);
    bind_admin(&mut ldap, &base).await?;

    ldap.delete(&dn)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    let _ = ldap.unbind().await;
    Ok(())
}

pub async fn authenticate_user(uid: Uid, password: Password) -> Result<bool, AppError> {
    let url = ldap_url();
    let base = ldap_base();
    let dn = format!("uid={},ou=people,{base}", uid.as_str());

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);

    let res = ldap
        .simple_bind(&dn, password.as_str())
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn gen_uid(prefix: &str) -> String {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            % 1000000;
        format!("{prefix}{n}")
    }

    async fn ensure_clean(uid: &str) {
        let url = ldap_url();
        let base = ldap_base();
        if let Ok((conn, mut ldap)) = LdapConnAsync::new(&url).await {
            ldap3::drive!(conn);
            let bind_dn = format!("cn=admin,{base}");
            if ldap.simple_bind(&bind_dn, "admin").await.is_ok() {
                let dn = format!("uid={uid},ou=people,{base}");
                let _ = ldap.delete(&dn).await;
                let _ = ldap.unbind().await;
            }
        }
    }

    fn is_ldap_down<T>(r: &Result<T, AppError>) -> bool {
        matches!(r, Err(AppError::Ldap(_)))
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
    async fn delete_user_removes_entry() {
        let uid = gen_uid("del");
        ensure_clean(&uid).await;

        let new = NewUser {
            uid: Uid::try_new(uid.clone()).unwrap(),
            name: Name::try_new("Delete Me".into()).unwrap(),
            email: Email::try_new("".into()).unwrap(),
            password: Password::try_new("pw".into()).unwrap(),
        };

        let r = create_user(new).await;
        if is_ldap_down(&r) {
            return;
        }

        r.unwrap();

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
        ensure_clean(&uid).await;

        let new: NewUser = NewUser {
            uid: Uid::try_new(uid.clone()).unwrap(),
            name: Name::try_new("Auth Test".into()).unwrap(),
            email: Email::try_new("a@ex.com".into()).unwrap(),
            password: Password::try_new("secret123".into()).unwrap(),
        };

        let r = create_user(new).await;
        if is_ldap_down(&r) {
            return;
        }

        r.unwrap();

        assert_auth_ok(&uid, "secret123").await;
        assert_auth_fail(&uid, "wrong").await;
        assert_auth_err(&uid, "").await;

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn authenticate_after_password_update() {
        let uid = gen_uid("auth2");
        ensure_clean(&uid).await;

        let new = NewUser {
            uid: Uid::try_new(uid.clone()).unwrap(),
            name: Name::try_new("T".into()).unwrap(),
            email: Email::try_new("".into()).unwrap(),
            password: Password::try_new("old".into()).unwrap(),
        };

        let r = create_user(new).await;
        if is_ldap_down(&r) {
            return;
        }

        r.unwrap();

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
