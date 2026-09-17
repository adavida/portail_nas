use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct NewUser {
    pub uid: String,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateUserError {
    MissingUid,
    MissingName,
    MissingPassword,
}

impl std::fmt::Display for CreateUserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUid => write!(f, "missing uid"),
            Self::MissingName => write!(f, "missing name"),
            Self::MissingPassword => write!(f, "missing password"),
        }
    }
}

impl std::error::Error for CreateUserError {}

impl NewUser {
    pub fn validate(&self) -> Result<(), CreateUserError> {
        if self.uid.trim().is_empty() {
            return Err(CreateUserError::MissingUid);
        }
        if self.name.trim().is_empty() {
            return Err(CreateUserError::MissingName);
        }
        if self.password.is_empty() {
            return Err(CreateUserError::MissingPassword);
        }
        Ok(())
    }

    pub fn dn(&self, base: &str) -> String {
        format!("uid={},ou=people,{base}", self.uid)
    }

    pub fn to_attrs(&self) -> Vec<(String, std::collections::HashSet<String>)> {
        let sn = self
            .name
            .split_whitespace()
            .last()
            .unwrap_or(&self.name)
            .to_string();
        vec![
            (
                "objectClass".to_string(),
                ["inetOrgPerson".to_string()].into_iter().collect(),
            ),
            ("uid".to_string(), [self.uid.clone()].into_iter().collect()),
            ("cn".to_string(), [self.name.clone()].into_iter().collect()),
            ("sn".to_string(), [sn].into_iter().collect()),
            (
                "displayName".to_string(),
                [self.name.clone()].into_iter().collect(),
            ),
            (
                "mail".to_string(),
                [self.email.clone()].into_iter().collect(),
            ),
            (
                "userPassword".to_string(),
                [self.password.clone()].into_iter().collect(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_user_validate() {
        let err1 = NewUser {
            uid: "".into(),
            name: "x".into(),
            email: "".into(),
            password: "p".into(),
        }
        .validate()
        .unwrap_err();

        assert_eq!(err1, CreateUserError::MissingUid);

        let err2 = NewUser {
            uid: "a".into(),
            name: "".into(),
            email: "".into(),
            password: "p".into(),
        }
        .validate()
        .unwrap_err();

        assert_eq!(err2, CreateUserError::MissingName);

        let err3 = NewUser {
            uid: "a".into(),
            name: "n".into(),
            email: "".into(),
            password: "".into(),
        }
        .validate()
        .unwrap_err();

        assert_eq!(err3, CreateUserError::MissingPassword);
    }

    #[test]
    fn new_user_dn_and_attrs() {
        let u = NewUser {
            uid: "bob".into(),
            name: "Bob Dupont".into(),
            email: "bob@example.com".into(),
            password: "secret".into(),
        };

        let dn = u.dn("dc=dev,dc=example,dc=com");

        assert_eq!(dn, "uid=bob,ou=people,dc=dev,dc=example,dc=com");

        let attrs = u.to_attrs();
        let find = |k: &str| attrs.iter().find(|(kk, _)| kk == k).unwrap().1.clone();

        assert!(find("uid").contains("bob"));
        assert!(find("cn").contains("Bob Dupont"));
        assert!(find("sn").contains("Dupont"));
        assert!(find("userPassword").contains("secret"));
    }
}
