use crate::{
    domain::{ldap::Uid, users::UserAttr},
    repository::ldap::{MapLdap, connect, find_user_dn},
};
use portail_core::{config::Config, error::AppError};
use std::collections::{HashMap, HashSet};

pub(crate) const ATTR_OBJECT_CLASS: &str = "portailUser";
pub(crate) const ATTR_ELEMENT: &str = "portailAttr";

pub(crate) fn catalog_labels() -> Vec<String> {
    Config::global()
        .attrs
        .iter()
        .map(|spec| spec.name.clone())
        .collect()
}

pub(crate) fn parse_attr_values(values: &[String]) -> Vec<UserAttr> {
    values
        .iter()
        .filter_map(|raw| {
            let (label, value) = raw.split_once('=')?;
            Some(UserAttr::from_stored(label.to_string(), value.to_string()))
        })
        .collect()
}

pub(crate) fn collect_attrs(
    all: &HashMap<String, Vec<String>>,
    catalog: &[String],
) -> Vec<UserAttr> {
    let legacy: Vec<UserAttr> = all
        .get(ATTR_ELEMENT)
        .map(|values| parse_attr_values(values))
        .unwrap_or_default();
    let mut attrs: Vec<UserAttr> = catalog
        .iter()
        .filter_map(|label| {
            all.get(label)
                .and_then(|values| values.first())
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    legacy
                        .iter()
                        .find(|attr| &attr.label == label && !attr.value.is_empty())
                        .map(|attr| &attr.value)
                })
                .map(|value| UserAttr::from_stored(label.clone(), value.clone()))
        })
        .collect();
    let known: HashSet<&str> = catalog.iter().map(|label| label.as_str()).collect();
    for attr in legacy {
        if !known.contains(attr.label.as_str()) {
            attrs.push(attr);
        }
    }
    attrs
}

pub(crate) type EntryAttributes = (Vec<String>, HashMap<String, Vec<String>>);

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn read_attr_entry(
    ldap: &mut ldap3::Ldap,
    dn: &str,
) -> Result<EntryAttributes, AppError> {
    let mut wanted = vec!["objectClass".to_string(), ATTR_ELEMENT.to_string()];
    wanted.extend(catalog_labels());
    let (rs, _res) = ldap
        .search(dn, ldap3::Scope::Base, "(objectClass=*)", wanted)
        .await
        .map_ldap()?
        .success()
        .map_ldap()?;
    let mut classes = Vec::new();
    let mut all = HashMap::new();
    for entry in rs {
        let entry = ldap3::SearchEntry::construct(entry);
        if let Some(values) = entry.attrs.get("objectClass") {
            classes = values.clone();
        }
        for (name, values) in entry.attrs {
            all.insert(name, values);
        }
    }
    Ok((classes, all))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_user_attrs(uid: &Uid) -> Result<Vec<UserAttr>, AppError> {
    let mut ldap = connect().await?;
    let dn = find_user_dn(&mut ldap, uid).await?;
    let (_classes, all) = read_attr_entry(&mut ldap, &dn).await?;
    let _ = ldap.unbind().await;
    Ok(collect_attrs(&all, &catalog_labels()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_label_value_pairs_and_skips_malformed() {
        let values = vec![
            "note=bonjour".to_string(),
            "ref=x-1".to_string(),
            "no-equals-sign".to_string(),
        ];

        let parsed = parse_attr_values(&values);

        assert_eq!(
            parsed,
            vec![
                UserAttr::from_stored("note".into(), "bonjour".into()),
                UserAttr::from_stored("ref".into(), "x-1".into()),
            ],
            "pairs must round-trip and malformed entries be skipped"
        );
    }

    #[test]
    fn collect_prefers_direct_attributes_over_legacy_pairs() {
        let catalog = vec!["city".to_string()];
        let mut all = HashMap::new();
        all.insert("city".to_string(), vec!["Strasbourg".to_string()]);
        all.insert(
            ATTR_ELEMENT.to_string(),
            vec!["city=Paris".to_string(), "note=keep".to_string()],
        );

        let attrs = collect_attrs(&all, &catalog);

        assert_eq!(
            attrs,
            vec![
                UserAttr::from_stored("city".into(), "Strasbourg".into()),
                UserAttr::from_stored("note".into(), "keep".into()),
            ],
            "the direct attribute wins and only unlisted legacy pairs are added"
        );
    }

    #[test]
    fn collect_drops_empty_direct_attributes() {
        let catalog = vec!["city".to_string()];
        let mut all = HashMap::new();
        all.insert("city".to_string(), vec!["".to_string()]);

        let attrs = collect_attrs(&all, &catalog);

        assert!(
            attrs.is_empty(),
            "an empty attribute value must not surface as a stored pair"
        );
    }

    #[test]
    fn collect_falls_back_to_legacy_pair_for_newly_catalogued_label() {
        let catalog = vec!["telephone".to_string()];
        let mut all = HashMap::new();
        all.insert(
            ATTR_ELEMENT.to_string(),
            vec!["telephone=+33612345678".to_string()],
        );

        let attrs = collect_attrs(&all, &catalog);

        assert_eq!(
            attrs,
            vec![UserAttr::from_stored(
                "telephone".into(),
                "+33612345678".into()
            )],
            "a value packed before the label was catalogued must stay visible until rewritten"
        );
    }
}
