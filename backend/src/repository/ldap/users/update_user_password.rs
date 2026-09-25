use ldap3::Mod;

use crate::{
    domain::users::{Password, Uid},
    error::AppError,
    repository::ldap::{MapLdap, connect, find_user_dn},
};

use super::one_set;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_user_password(uid: &Uid, password: Password) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_user_dn(&mut ldap, uid).await?;

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
