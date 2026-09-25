use ldap3::Mod;
use std::collections::HashSet;

use crate::{
    domain::groups::{Gid, UpdateGroup},
    error::AppError,
    repository::ldap::{MapLdap, connect, find_group_dn},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_group(gid: Gid, data: UpdateGroup) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = find_group_dn(&mut ldap, &gid).await?;

    let set_description = if data.description.as_str().is_empty() {
        HashSet::new()
    } else {
        [data.description.as_str().to_string()]
            .into_iter()
            .collect()
    };

    ldap.modify(
        &dn,
        vec![
            Mod::Replace(
                "o".to_string(),
                [data.name.as_str().to_string()].into_iter().collect(),
            ),
            Mod::Replace("description".to_string(), set_description),
        ],
    )
    .await
    .map_ldap()?
    .success()
    .map_ldap()?;
    let _ = ldap.unbind().await;
    Ok(())
}
