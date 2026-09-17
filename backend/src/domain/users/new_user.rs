use serde::Deserialize;

use super::{Email, Password, Uid};
use crate::domain::shared::Name;

#[derive(Deserialize, Clone, Debug)]
pub struct NewUser {
    pub uid: Uid,
    pub name: Name,
    pub email: Email,
    pub password: Password,
}

impl NewUser {
    pub fn dn(&self, base: &str) -> String {
        format!("uid={},ou=people,{base}", self.uid.as_str())
    }

    pub fn to_attrs(&self) -> Vec<(String, std::collections::HashSet<String>)> {
        let sn = self
            .name
            .as_str()
            .split_whitespace()
            .last()
            .unwrap_or(self.name.as_str())
            .to_string();
        vec![
            (
                "objectClass".to_string(),
                ["inetOrgPerson".to_string()].into_iter().collect(),
            ),
            (
                "uid".to_string(),
                [self.uid.as_str().to_string()].into_iter().collect(),
            ),
            (
                "cn".to_string(),
                [self.name.as_str().to_string()].into_iter().collect(),
            ),
            ("sn".to_string(), [sn].into_iter().collect()),
            (
                "displayName".to_string(),
                [self.name.as_str().to_string()].into_iter().collect(),
            ),
            (
                "mail".to_string(),
                [self.email.as_str().to_string()].into_iter().collect(),
            ),
            (
                "userPassword".to_string(),
                [self.password.as_str().to_string()].into_iter().collect(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_user_dn_and_attrs() {
        let u = NewUser {
            uid: Uid::try_new("bob".into()).unwrap(),
            name: Name::try_new("Bob Dupont".into()).unwrap(),
            email: Email::try_new("bob@example.com".into()).unwrap(),
            password: Password::try_new("secret".into()).unwrap(),
        };

        let dn = u.dn("dc=dev,dc=example,dc=com");

        assert_eq!(
            dn, "uid=bob,ou=people,dc=dev,dc=example,dc=com",
            "dn should be uid + ou=people + base"
        );

        let attrs = u.to_attrs();
        let find = |k: &str| attrs.iter().find(|(kk, _)| kk == k).unwrap().1.clone();

        assert!(find("uid").contains("bob"), "attrs should contain uid=bob");
        assert!(
            find("cn").contains("Bob Dupont"),
            "attrs should contain cn=Bob Dupont"
        );
        assert!(
            find("sn").contains("Dupont"),
            "sn should be last word of name"
        );
        assert!(
            find("userPassword").contains("secret"),
            "attrs should contain userPassword"
        );
    }

    #[test]
    fn deserialize_validates_uid() {
        let json = r#"{"uid":"","name":"Bob","email":"","password":"secret"}"#;

        let err = serde_json::from_str::<NewUser>(json).unwrap_err();

        assert!(
            err.to_string().contains("uid is empty"),
            "empty uid should be rejected via Deserialize"
        );
    }

    #[test]
    fn deserialize_validates_name() {
        let json = r#"{"uid":"bob","name":"","email":"","password":"secret"}"#;

        let err = serde_json::from_str::<NewUser>(json).unwrap_err();

        assert!(
            err.to_string().contains("name is empty"),
            "empty name should be rejected"
        );
    }

    #[test]
    fn deserialize_validates_password() {
        let json = r#"{"uid":"bob","name":"Bob","email":"","password":""}"#;

        let err = serde_json::from_str::<NewUser>(json).unwrap_err();

        assert!(
            err.to_string().contains("password is empty"),
            "empty password should be rejected"
        );
    }
}
