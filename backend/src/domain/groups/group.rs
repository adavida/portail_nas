use serde::Serialize;

use super::{Description, Gid, Name};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub gid: Gid,
    pub name: Name,
    pub description: Description,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupError {
    MissingCn,
    InvalidGid,
    InvalidName,
}

impl std::fmt::Display for GroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCn => write!(f, "missing cn"),
            Self::InvalidGid => write!(f, "invalid gid"),
            Self::InvalidName => write!(f, "invalid name"),
        }
    }
}

impl std::error::Error for GroupError {}

impl Group {
    pub fn from_attrs(
        cn: Option<String>,
        o: Option<String>,
        description: Option<String>,
    ) -> Result<Self, GroupError> {
        let cn = cn.ok_or(GroupError::MissingCn)?;
        let gid = Gid::try_new(cn).map_err(|_| GroupError::InvalidGid)?;
        let raw_name = o.unwrap_or_else(|| gid.as_str().to_string());
        let name = Name::try_new(raw_name).map_err(|_| GroupError::InvalidName)?;
        let description = Description::try_new(description.unwrap_or_default());
        Ok(Self {
            gid,
            name,
            description,
        })
    }

    pub fn from_search(entries: Vec<std::collections::HashMap<String, Vec<String>>>) -> Vec<Self> {
        let mut groups: Vec<Self> = entries
            .into_iter()
            .filter_map(|attrs| {
                let cn = attrs.get("cn").and_then(|v| v.first().cloned());
                let o = attrs.get("o").and_then(|v| v.first().cloned());
                let description = attrs.get("description").and_then(|v| v.first().cloned());
                Self::from_attrs(cn, o, description).ok()
            })
            .collect();
        groups.sort_by(|a, b| a.gid.as_str().cmp(b.gid.as_str()));
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_maps_to_name() {
        let g = Group::from_attrs(
            Some("devs".into()),
            Some("Devs".into()),
            Some("Devs team".into()),
        )
        .unwrap();

        assert_eq!(g.gid.as_str(), "devs");
        assert_eq!(g.name.as_str(), "Devs");
        assert_eq!(g.description.as_str(), "Devs team");
    }

    #[test]
    fn needs_cn() {
        let result = Group::from_attrs(None, None, None).unwrap_err();

        assert_eq!(result, GroupError::MissingCn);
    }

    #[test]
    fn rejects_invalid_gid() {
        let result = Group::from_attrs(Some("Devs!".into()), None, None).unwrap_err();

        assert_eq!(result, GroupError::InvalidGid);
    }
}
