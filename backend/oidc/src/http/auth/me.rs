use axum::{Extension, Json};
use portail_core::config::{AttrSpec, Config};
use portail_directory::{
    controllers::users::list_attrs,
    domain::{ldap::Uid, users::UserAttr},
};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::auth::{Claims, middleware::AuthUser};

#[derive(Serialize)]
pub struct AttrView {
    label: String,
    value: String,
    #[serde(rename = "type")]
    kind: &'static str,
    max_chars: Option<usize>,
    min: Option<i64>,
    max: Option<i64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    names: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub struct Me {
    sub: String,
    preferred_username: Option<String>,
    name: Option<String>,
    email: Option<String>,
    groups: Vec<String>,
    is_admin: bool,
    attrs: Vec<AttrView>,
}

pub fn merge_attrs(catalog: &[AttrSpec], stored: &[UserAttr]) -> Vec<AttrView> {
    let mut views: Vec<AttrView> = catalog
        .iter()
        .map(|spec| AttrView {
            label: spec.name.clone(),
            value: stored
                .iter()
                .find(|attr| attr.label == spec.name)
                .map(|attr| attr.value.clone())
                .unwrap_or_default(),
            kind: spec.kind.as_str(),
            max_chars: spec.max_chars,
            min: spec.min,
            max: spec.max,
            names: spec.names.clone(),
        })
        .collect();
    let known: Vec<&str> = catalog.iter().map(|spec| spec.name.as_str()).collect();
    views.extend(
        stored
            .iter()
            .filter(|attr| !known.contains(&attr.label.as_str()))
            .map(|attr| AttrView {
                label: attr.label.clone(),
                value: attr.value.clone(),
                kind: "text",
                max_chars: None,
                min: None,
                max: None,
                names: BTreeMap::new(),
            }),
    );
    views
}

impl Me {
    fn from_claims(claims: &Claims) -> Self {
        Self {
            sub: claims.sub.clone(),
            preferred_username: claims.preferred_username.clone(),
            name: None,
            email: claims.email.clone(),
            groups: claims.groups.clone(),
            is_admin: claims.is_admin(),
            attrs: Vec::new(),
        }
    }
}

pub async fn me(Extension(AuthUser(claims)): Extension<AuthUser>) -> Json<Me> {
    let mut me = Me::from_claims(&claims);
    let Some(username) = claims.preferred_username.clone() else {
        return Json(me);
    };
    let fresh = portail_directory::controllers::users::list()
        .await
        .ok()
        .and_then(|users| users.into_iter().find(|u| u.uid.as_str() == username));
    if let Some(user) = fresh {
        me.name = Some(user.name.as_str().to_string());
        me.email = Some(user.email.as_str().to_string());
    }
    if let Ok(uid) = Uid::try_new(username) {
        let stored = list_attrs(&uid).await.unwrap_or_default();
        me.attrs = merge_attrs(&Config::global().attrs, &stored);
    }
    Json(me)
}

#[cfg(test)]
mod tests {
    use super::*;
    use portail_core::config::AttrKind;

    fn spec(name: &str, kind: AttrKind) -> AttrSpec {
        AttrSpec {
            name: name.into(),
            kind,
            max_chars: None,
            min: None,
            max: None,
            names: [(String::from("fr"), String::from("Ville"))]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn merge_attrs_lists_catalog_then_extra_stored() {
        let catalog = vec![spec("city", AttrKind::Text)];
        let stored = vec![
            UserAttr::from_stored("city".into(), "Nice".into()),
            UserAttr::from_stored("legacy".into(), "x".into()),
        ];

        let views = merge_attrs(&catalog, &stored);

        assert_eq!(views.len(), 2, "one catalogued + one unlisted");
        assert_eq!(views[0].label, "city");
        assert_eq!(views[0].value, "Nice");
        assert_eq!(
            views[1].label, "legacy",
            "stored-but-uncatalogued must show"
        );
        assert_eq!(views[1].kind, "text", "unlisted falls back to text");
    }

    #[test]
    fn merge_attrs_yields_empty_value_for_unset_catalog_label() {
        let catalog = vec![spec("age", AttrKind::Number)];

        let views = merge_attrs(&catalog, &[]);

        assert_eq!(views[0].value, "", "an unset catalogued label is empty");
    }

    #[test]
    fn merge_attrs_carries_localized_names_and_empty_map_for_unlisted() {
        let catalog = vec![spec("city", AttrKind::Text)];
        let stored = vec![UserAttr::from_stored("legacy".into(), "x".into())];

        let views = merge_attrs(&catalog, &stored);

        assert_eq!(
            views[0].names.get("fr").map(String::as_str),
            Some("Ville"),
            "catalogued names must reach the view"
        );
        assert!(
            views[1].names.is_empty(),
            "unlisted labels carry no localized names"
        );
    }
}
