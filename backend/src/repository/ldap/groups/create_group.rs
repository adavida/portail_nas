use crate::{
    domain::groups::{Group, NewGroup},
    error::AppError,
    repository::ldap::{MapLdap, connect, find_user_dn, group_dn_from},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn create_group(new: NewGroup) -> Result<Group, AppError> {
    let dn = group_dn_from(&new.gid);
    let mut ldap = connect().await?;

    let mut member_dns: Vec<String> = Vec::new();
    for uid in new.members.as_slice() {
        member_dns.push(find_user_dn(&mut ldap, uid).await?);
    }

    let mut attrs = new.to_attrs();
    attrs.push(("member".to_string(), member_dns.into_iter().collect()));

    ldap.add(&dn, attrs)
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let _ = ldap.unbind().await;

    Ok(Group {
        gid: new.gid,
        name: new.name,
        description: new.description,
    })
}
