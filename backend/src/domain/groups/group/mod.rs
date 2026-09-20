pub mod errors;
pub use errors::GroupError;

use serde::Serialize;

use super::{Description, Gid, Name};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub gid: Gid,
    pub name: Name,
    pub description: Description,
    pub members: Vec<String>,
}

impl Group {
    pub fn from_attrs(
        cn: Option<String>,
        o: Option<String>,
        description: Option<String>,
        member_dns: Vec<String>,
    ) -> Result<Self, GroupError> {
        let cn = cn.ok_or(GroupError::MissingCn)?;
        let gid = Gid::try_new(cn).map_err(|_| GroupError::InvalidGid)?;
        let raw_name = o.unwrap_or_else(|| gid.as_str().to_string());
        let name = Name::try_new(raw_name).map_err(|_| GroupError::InvalidName)?;
        let description = Description::try_new(description.unwrap_or_default());

        let members = member_dns
            .iter()
            .filter_map(|dn| {
                let rest = dn.strip_prefix("uid=")?;
                rest.split(',').next().map(|uid| uid.to_string())
            })
            .collect();

        Ok(Self {
            gid,
            name,
            description,
            members,
        })
    }

    pub fn from_search(entries: Vec<std::collections::HashMap<String, Vec<String>>>) -> Vec<Self> {
        let mut groups: Vec<Self> = entries
            .into_iter()
            .filter_map(|attrs| {
                let cn = attrs.get("cn").and_then(|v| v.first().cloned());
                let o = attrs.get("o").and_then(|v| v.first().cloned());
                let description = attrs.get("description").and_then(|v| v.first().cloned());
                let member = attrs.get("member").cloned().unwrap_or_default();
                Self::from_attrs(cn, o, description, member).ok()
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
            vec![],
        )
        .unwrap();

        assert_eq!(g.gid.as_str(), "devs");
        assert_eq!(g.name.as_str(), "Devs");
        assert_eq!(g.description.as_str(), "Devs team");
        assert!(g.members.is_empty());
    }

    #[test]
    fn member_dns_map_to_uids() {
        let base = "dc=dev,dc=example,dc=com";
        let g = Group::from_attrs(
            Some("devs".into()),
            None,
            None,
            vec![
                format!("uid=alice,ou=people,{base}"),
                format!("uid=bob,ou=people,{base}"),
                format!("cn=devs,ou=groups,{base}"),
            ],
        )
        .unwrap();

        assert_eq!(
            g.members.len(),
            2,
            "placeholder self-DN should be filtered out"
        );
        assert!(g.members.contains(&"alice".to_string()));
        assert!(g.members.contains(&"bob".to_string()));
    }

    #[test]
    fn needs_cn() {
        let result = Group::from_attrs(None, None, None, vec![]).unwrap_err();

        assert_eq!(result, GroupError::MissingCn);
    }

    #[test]
    fn rejects_invalid_gid() {
        let result = Group::from_attrs(Some("Devs!".into()), None, None, vec![]).unwrap_err();

        assert_eq!(result, GroupError::InvalidGid);
    }
}
