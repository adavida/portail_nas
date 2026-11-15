use ldap3::Mod;
use std::collections::HashSet;

use crate::{
    domain::{ldap::Uid, users::UserAttr},
    repository::ldap::{MapLdap, connect},
};
use portail_core::error::AppError;

use super::{ATTR_ELEMENT, ATTR_OBJECT_CLASS, catalog_labels, one_set, read_attr_entry};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn set_user_attrs(uid: &Uid, attrs: Vec<UserAttr>) -> Result<(), AppError> {
    let mut ldap = connect().await?;
    let dn = crate::repository::ldap::find_user_dn(&mut ldap, uid).await?;
    let (classes, existing) = read_attr_entry(&mut ldap, &dn).await?;
    let catalog: HashSet<String> = catalog_labels().into_iter().collect();

    let mut mods: Vec<Mod<String>> = Vec::new();
    for attr in &attrs {
        if catalog.contains(&attr.label) {
            let set = if attr.value.is_empty() {
                HashSet::new()
            } else {
                one_set(attr.value.clone())
            };
            mods.push(Mod::Replace(attr.label.clone(), set));
        }
    }

    let payload_labels: HashSet<&str> = attrs.iter().map(|a| a.label.as_str()).collect();
    let mut kept: Vec<String> = existing
        .get(ATTR_ELEMENT)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|raw| {
            raw.split_once('=')
                .map(|(label, _)| !payload_labels.contains(label) && !catalog.contains(label))
                .unwrap_or(true)
        })
        .collect();
    let mut packed: Vec<String> = attrs
        .iter()
        .filter(|attr| !catalog.contains(&attr.label) && !attr.value.is_empty())
        .map(|attr| format!("{}={}", attr.label, attr.value))
        .collect();
    kept.append(&mut packed);
    let legacy_set: HashSet<String> = kept.into_iter().collect();
    mods.push(Mod::Replace(ATTR_ELEMENT.to_string(), legacy_set));

    let writes_something = attrs.iter().any(|attr| !attr.value.is_empty());
    if writes_something && !classes.iter().any(|c| c == ATTR_OBJECT_CLASS) {
        mods.insert(
            0,
            Mod::Add(
                "objectClass".to_string(),
                one_set(ATTR_OBJECT_CLASS.to_string()),
            ),
        );
    }

    ldap.modify(&dn, mods)
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
    use crate::repository::ldap::users::{delete_user, list_user_attrs};
    use crate::test_helpers::seed::{ensure_clean, gen_uid, seed_and_create_user};

    #[tokio::test]
    async fn set_user_attrs_round_trips_through_ldap() {
        let uid = gen_uid("lattr");
        if !seed_and_create_user(&uid, "Ldap Attrs", "la@ex.com", "pw").await {
            return;
        }

        set_user_attrs(
            &uid,
            vec![
                UserAttr::try_new("city".into(), "Nice".into()).unwrap(),
                UserAttr::try_new("age".into(), "30".into()).unwrap(),
            ],
        )
        .await
        .unwrap();
        set_user_attrs(
            &uid,
            vec![UserAttr::try_new("city".into(), "".into()).unwrap()],
        )
        .await
        .unwrap();
        let attrs = list_user_attrs(&uid).await.unwrap();

        assert_eq!(
            attrs,
            vec![UserAttr::from_stored("age".into(), "30".into())],
            "the emptied label must be removed from its LDAP attribute"
        );

        let _ = delete_user(&uid).await;
        ensure_clean(&uid).await;
    }
}
