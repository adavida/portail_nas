use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub uid: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    MissingUid,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUid => write!(f, "missing uid"),
        }
    }
}

impl std::error::Error for UserError {}

impl User {
    pub fn from_attrs(
        uid: Option<String>,
        cn: Option<String>,
        display_name: Option<String>,
        mail: Option<String>,
    ) -> Result<Self, UserError> {
        let uid = uid.ok_or(UserError::MissingUid)?;
        let name = display_name
            .filter(|s| !s.is_empty())
            .or(cn)
            .unwrap_or_default();
        Ok(Self {
            uid,
            name,
            email: mail.unwrap_or_default(),
        })
    }

    pub fn from_search(entries: Vec<std::collections::HashMap<String, Vec<String>>>) -> Vec<Self> {
        let mut users: Vec<Self> = entries
            .into_iter()
            .filter_map(|attrs| {
                let uid = attrs.get("uid").and_then(|v| v.first().cloned());
                let cn = attrs.get("cn").and_then(|v| v.first().cloned());
                let display_name = attrs.get("displayName").and_then(|v| v.first().cloned());
                let mail = attrs.get("mail").and_then(|v| v.first().cloned());
                Self::from_attrs(uid, cn, display_name, mail).ok()
            })
            .collect();
        users.sort_by(|a, b| a.uid.cmp(&b.uid));
        users
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_preferred_over_cn() {
        let u = User::from_attrs(
            Some("alice".into()),
            Some("Alice C".into()),
            Some("Alice D".into()),
            Some("a@ex.com".into()),
        )
        .unwrap();

        assert_eq!(
            u.name, "Alice D",
            "displayName should win over cn for user name"
        );
    }

    #[test]
    fn cn_fallback_when_no_display() {
        let u = User::from_attrs(Some("bob".into()), Some("Bob C".into()), None, None).unwrap();

        assert_eq!(
            u.name, "Bob C",
            "cn should be fallback when displayName is None"
        );
        assert_eq!(u.email, "", "email should be empty when mail is None");
    }

    #[test]
    fn needs_uid() {
        let result = User::from_attrs(None, Some("x".into()), None, None).unwrap_err();

        assert_eq!(
            result,
            UserError::MissingUid,
            "from_attrs without uid should fail with MissingUid"
        );
    }
}
