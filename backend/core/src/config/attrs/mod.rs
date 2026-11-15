use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttrKind {
    Text,
    Number,
    Email,
    Telephone,
}

impl AttrKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Email => "email",
            Self::Telephone => "telephone",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttrSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: AttrKind,
    #[serde(default)]
    pub max_chars: Option<usize>,
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
    #[serde(default)]
    pub names: BTreeMap<String, String>,
}

pub const LDAP_NAME_MAX_CHARS: usize = 32;

pub const LDAP_RESERVED_ATTR_NAMES: [&str; 19] = [
    "cn",
    "dc",
    "description",
    "displayName",
    "givenName",
    "l",
    "mail",
    "member",
    "mobile",
    "o",
    "objectClass",
    "ou",
    "postalCode",
    "secretary",
    "sn",
    "street",
    "telephoneNumber",
    "uid",
    "userPassword",
];

impl AttrSpec {
    fn is_ldap_attr_name(name: &str) -> bool {
        let mut chars = name.chars();
        matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
            && name.len() >= 2
            && name.len() <= LDAP_NAME_MAX_CHARS
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    pub fn check(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("attrs label name must not be empty".to_string());
        }
        if !Self::is_ldap_attr_name(&self.name) {
            return Err(format!(
                "attrs label '{}': must be 2-{LDAP_NAME_MAX_CHARS} characters of a-z, 0-9, '-', starting with a letter (catalogued labels become LDAP attribute names, no dots or underscores)",
                self.name
            ));
        }
        if LDAP_RESERVED_ATTR_NAMES.contains(&self.name.as_str()) {
            return Err(format!(
                "attrs label '{}' shadows a standard LDAP attribute name",
                self.name
            ));
        }
        match self.kind {
            AttrKind::Text => {
                if self.min.is_some() || self.max.is_some() {
                    return Err(format!(
                        "attrs label '{}': min/max are only valid for type number",
                        self.name
                    ));
                }
                if self.max_chars == Some(0) {
                    return Err(format!(
                        "attrs label '{}': max_chars must be at least 1",
                        self.name
                    ));
                }
            }
            AttrKind::Number => {
                if self.max_chars.is_some() {
                    return Err(format!(
                        "attrs label '{}': max_chars is only valid for type text",
                        self.name
                    ));
                }
                if let (Some(min), Some(max)) = (self.min, self.max) {
                    if min > max {
                        return Err(format!(
                            "attrs label '{}': min ({min}) must not exceed max ({max})",
                            self.name
                        ));
                    }
                }
            }
            AttrKind::Email | AttrKind::Telephone => {
                if self.max_chars.is_some() || self.min.is_some() || self.max.is_some() {
                    return Err(format!(
                        "attrs label '{}': type {} takes no sizing options",
                        self.name,
                        self.kind.as_str()
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn validate_labels(labels: &[AttrSpec]) -> Result<(), String> {
    for spec in labels {
        spec.check()?;
        if crate::config::RESERVED_CLAIM_NAMES.contains(&spec.name.as_str()) {
            return Err(format!(
                "attrs label '{}' shadows a reserved OIDC claim name",
                spec.name
            ));
        }
    }
    if let Some(dup) = labels.iter().find_map(|a| {
        labels
            .iter()
            .filter(|b| b.name == a.name)
            .nth(1)
            .map(|_| a.name.clone())
    }) {
        return Err(format!("attrs labels contain duplicate name '{dup}'"));
    }
    Ok(())
}
