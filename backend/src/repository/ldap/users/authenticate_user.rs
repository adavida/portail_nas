use crate::{
    domain::users::{Password, Uid},
    error::AppError,
    repository::ldap::{MapLdap, connect, find_user_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn authenticate_user(uid: &Uid, password: Password) -> Result<bool, AppError> {
    let mut ldap = connect().await?;
    let dn = match find_user_dn(&mut ldap, uid).await {
        Ok(dn) => dn,
        Err(AppError::NotFound(_)) => {
            let _ = ldap.unbind().await;
            return Ok(false);
        }
        Err(e) => return Err(e),
    };

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
    use crate::repository::ldap::users::update_user_password;
    use crate::test_helpers::assert::{assert_auth_err, assert_auth_fail, assert_auth_ok};
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};

    #[tokio::test]
    async fn authenticate_ok_and_wrong() {
        let uid = gen_uid("auth");
        if !seed_and_create_user(&uid, "Auth Test", "a@ex.com", "secret123").await {
            return;
        }

        assert_auth_ok(uid.as_str(), "secret123").await;
        assert_auth_fail(uid.as_str(), "wrong").await;
        assert_auth_err(uid.as_str(), "").await;

        ensure_clean(&uid).await;
    }

    #[tokio::test]
    async fn authenticate_after_password_update() {
        let uid = gen_uid("auth2");
        if !seed_and_create_user(&uid, "T", "", "old").await {
            return;
        }

        update_user_password(&uid, Password::try_new("newpass".into()).unwrap())
            .await
            .unwrap();

        assert_auth_fail(uid.as_str(), "old").await;
        assert_auth_ok(uid.as_str(), "newpass").await;

        ensure_clean(&uid).await;
    }
}
