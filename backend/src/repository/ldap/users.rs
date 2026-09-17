use ldap3::{LdapConnAsync, Mod, Scope, SearchEntry};
use std::collections::HashSet;

use crate::{
    domain::users::{NewUser, UpdatePassword, User},
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
    new.validate()
        .map_err(|e| AppError::Internal(e.to_string()))?;
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

pub async fn update_user_password(uid: String, req: UpdatePassword) -> Result<(), AppError> {
    req.validate()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if uid.trim().is_empty() {
        return Err(AppError::Internal("missing uid".into()));
    }
    let url = ldap_url();
    let base = ldap_base();
    let dn = format!("uid={uid},ou=people,{base}");

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);
    bind_admin(&mut ldap, &base).await?;

    let mut set = HashSet::new();
    set.insert(req.password);
    ldap.modify(&dn, vec![Mod::Replace("userPassword".to_string(), set)])
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?
        .success()
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    let _ = ldap.unbind().await;
    Ok(())
}

pub async fn authenticate_user(uid: String, password: String) -> Result<bool, AppError> {
    if uid.trim().is_empty() || password.is_empty() {
        return Err(AppError::Internal("missing uid/password".into()));
    }
    let url = ldap_url();
    let base = ldap_base();
    let dn = format!("uid={uid},ou=people,{base}");

    let (conn, mut ldap) = LdapConnAsync::new(&url)
        .await
        .map_err(|e| AppError::Ldap(e.to_string()))?;
    ldap3::drive!(conn);

    let res = ldap
        .simple_bind(&dn, &password)
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
    use crate::domain::users::NewUser;
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
        assert!(
            authenticate_user(uid.to_string(), password.to_string())
                .await
                .unwrap()
        );
    }

    async fn assert_auth_fail(uid: &str, password: &str) {
        assert!(
            !authenticate_user(uid.to_string(), password.to_string())
                .await
                .unwrap()
        );
    }

    async fn assert_auth_err(uid: &str, password: &str) {
        assert!(
            authenticate_user(uid.to_string(), password.to_string())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn authenticate_ok_and_wrong() {
        let uid = gen_uid("auth");
        ensure_clean(&uid).await;

        let new: NewUser = NewUser {
            uid: uid.clone(),
            name: "Auth Test".into(),
            email: "a@ex.com".into(),
            password: "secret123".into(),
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
            uid: uid.clone(),
            name: "T".into(),
            email: "".into(),
            password: "old".into(),
        };

        let r = create_user(new).await;
        if is_ldap_down(&r) {
            return;
        }

        r.unwrap();

        let v = crate::domain::users::UpdatePassword {
            password: "newpass".into(),
        }
        .validate();

        assert!(v.is_ok());

        update_user_password(
            uid.clone(),
            crate::domain::users::UpdatePassword {
                password: "newpass".into(),
            },
        )
        .await
        .unwrap();

        assert_auth_fail(&uid, "old").await;
        assert_auth_ok(&uid, "newpass").await;

        ensure_clean(&uid).await;
    }
}
