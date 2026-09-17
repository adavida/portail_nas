use serde::Deserialize;

use super::{Description, Gid};
use crate::domain::shared::Name;

#[derive(Deserialize, Clone, Debug)]
pub struct NewGroup {
    pub gid: Gid,
    pub name: Name,
    pub description: Description,
}

impl NewGroup {
    pub fn dn(&self, base: &str) -> String {
        format!("cn={},ou=groups,{base}", self.gid.as_str())
    }

    pub fn to_attrs(&self) -> Vec<(String, std::collections::HashSet<String>)> {
        vec![
            (
                "objectClass".to_string(),
                ["groupOfNames".to_string()].into_iter().collect(),
            ),
            (
                "cn".to_string(),
                [self.gid.as_str().to_string()].into_iter().collect(),
            ),
            (
                "o".to_string(),
                [self.name.as_str().to_string()].into_iter().collect(),
            ),
            (
                "description".to_string(),
                [self.description.as_str().to_string()]
                    .into_iter()
                    .collect(),
            ),
            (
                "member".to_string(),
                [format!("cn={},ou=groups", self.gid.as_str())]
                    .into_iter()
                    .collect(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_group_dn_and_attrs() {
        let g = NewGroup {
            gid: Gid::try_new("devs".into()).unwrap(),
            name: Name::try_new("Devs".into()).unwrap(),
            description: Description::try_new("Devs team".into()).unwrap(),
        };

        let dn = g.dn("dc=dev,dc=example,dc=com");

        assert_eq!(
            dn, "cn=devs,ou=groups,dc=dev,dc=example,dc=com",
            "dn should be cn + ou=groups + base"
        );

        let attrs = g.to_attrs();
        let find = |k: &str| attrs.iter().find(|(kk, _)| kk == k).unwrap().1.clone();

        assert!(
            find("objectClass").contains("groupOfNames"),
            "attrs should contain groupOfNames"
        );
        assert!(find("cn").contains("devs"), "attrs should contain cn=gid");
        assert!(find("o").contains("Devs"), "attrs should contain o=name");
        assert!(
            find("description").contains("Devs team"),
            "attrs should contain description"
        );
    }

    #[test]
    fn deserialize_validates_gid() {
        let json = r#"{"gid":"","name":"x","description":"x"}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("gid is empty"),
            "empty gid should be rejected via Deserialize"
        );
    }

    #[test]
    fn deserialize_validates_name() {
        let json = r#"{"gid":"devs","name":"","description":"x"}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("name is empty"),
            "empty name should be rejected"
        );
    }

    #[test]
    fn deserialize_validates_description() {
        let json = r#"{"gid":"devs","name":"x","description":""}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("description is empty"),
            "empty description should be rejected"
        );
    }
}
