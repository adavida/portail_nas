use serde::Deserialize;

use super::{Description, Gid, Members, Name};

#[derive(Deserialize, Clone, Debug)]
pub struct NewGroup {
    pub gid: Gid,
    pub name: Name,
    pub description: Description,
    pub members: Members,
}

impl NewGroup {
    /// LDAP attrs minus `member`: DNs are repository concern (env-driven OUs),
    /// `create_group` appends them.
    pub fn to_attrs(&self) -> Vec<(String, std::collections::HashSet<String>)> {
        let mut attrs = vec![
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
        ];
        let desc = self.description.as_str();
        if !desc.trim().is_empty() {
            attrs.push((
                "description".to_string(),
                [desc.to_string()].into_iter().collect(),
            ));
        }
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::Uid;

    fn test_group() -> NewGroup {
        NewGroup {
            gid: Gid::try_new("devs".into()).unwrap(),
            name: Name::try_new("Devs".into()).unwrap(),
            description: Description::try_new("Devs team".into()),
            members: Members::try_new(vec![
                Uid::try_new("alice".into()).unwrap(),
                Uid::try_new("bob".into()).unwrap(),
            ])
            .unwrap(),
        }
    }

    #[test]
    fn attrs_basic_shape() {
        let g = test_group();

        let attrs = g.to_attrs();

        assert!(
            attrs.iter().all(|(k, _)| k != "member"),
            "member DNs belong to the repository, not to_attrs"
        );
        assert!(
            attrs.iter().any(|(k, _)| k == "cn"),
            "attrs should contain cn"
        );
        assert!(
            attrs.iter().any(|(k, _)| k == "o"),
            "attrs should contain o"
        );
    }

    #[test]
    fn attrs_omit_description_when_empty() {
        let mut g = test_group();
        g.description = Description::try_new("".into());

        let attrs = g.to_attrs();

        assert!(
            attrs.iter().all(|(k, _)| k != "description"),
            "empty description should not be sent to LDAP"
        );
    }

    #[test]
    fn deserialize_validates_gid() {
        let json = r#"{"gid":"","name":"x","description":"x","members":["alice"]}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("gid is empty"),
            "empty gid should be rejected via Deserialize"
        );
    }

    #[test]
    fn deserialize_validates_name() {
        let json = r#"{"gid":"devs","name":"","description":"x","members":["alice"]}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("name is empty"),
            "empty name should be rejected"
        );
    }

    #[test]
    fn deserialize_rejects_empty_members() {
        let json = r#"{"gid":"devs","name":"x","description":"","members":[]}"#;

        let err = serde_json::from_str::<NewGroup>(json).unwrap_err();

        assert!(
            err.to_string().contains("members is empty"),
            "group needs at least one member at creation, got {err}"
        );
    }
}
