use crate::{
    domain::groups::Gid,
    error::AppError,
    repository::ldap::{MapLdap, connect, find_group_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn delete_group(gid: Gid) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_group_dn(&mut ldap, &gid).await?;

    ldap.delete(&dn).await.map_ldap()?.success().map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}
