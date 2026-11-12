use serde::Serialize;

use super::user_error::UserError;
use crate::domain::ldap::{Email, Gid, Name, Uid};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub uid: Uid,
    pub name: Name,
    pub email: Email,
    pub groups: Vec<Gid>,
}

impl User {
    pub fn from_attrs(
        uid: Option<String>,
        cn: Option<String>,
        display_name: Option<String>,
        mail: Option<String>,
    ) -> Result<Self, UserError> {
        let raw_uid = uid
            .clone()
            .or_else(|| cn.clone())
            .ok_or(UserError::MissingUid)?;
        let uid = Uid::try_new(raw_uid).map_err(|_| UserError::InvalidUid)?;
        let raw_name = display_name
            .filter(|s| !s.is_empty())
            .or(cn)
            .unwrap_or_else(|| uid.as_str().to_string());
        let name = Name::try_new(raw_name).map_err(|_| UserError::InvalidName)?;
        let email =
            Email::try_new(mail.unwrap_or_default()).map_err(|_| UserError::InvalidEmail)?;
        Ok(Self {
            uid,
            name,
            email,
            groups: Vec::new(),
        })
    }

    pub fn from_search(
        entry: std::collections::HashMap<String, Vec<String>>,
    ) -> Result<Self, UserError> {
        let uid = entry.get("uid").and_then(|v| v.first().cloned());
        let cn = entry.get("cn").and_then(|v| v.first().cloned());
        let display_name = entry.get("displayName").and_then(|v| v.first().cloned());
        let mail = entry.get("mail").and_then(|v| v.first().cloned());
        Self::from_attrs(uid, cn, display_name, mail)
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
            u.name.as_str(),
            "Alice D",
            "displayName should win over cn for user name"
        );
    }

    #[test]
    fn cn_fallback_when_no_display() {
        let u = User::from_attrs(Some("bob".into()), Some("Bob C".into()), None, None).unwrap();

        assert_eq!(
            u.name.as_str(),
            "Bob C",
            "cn should be fallback when displayName is None"
        );
        assert_eq!(
            u.email.as_str(),
            "",
            "email should be empty when mail is None"
        );
    }

    #[test]
    fn needs_uid() {
        let result = User::from_attrs(None, None, None, None).unwrap_err();

        assert_eq!(
            result,
            UserError::MissingUid,
            "from_attrs without uid or cn should fail with MissingUid"
        );
    }

    #[test]
    fn uid_falls_back_to_cn_when_no_uid_attribute() {
        let u = User::from_attrs(None, Some("carol".into()), None, None).unwrap();

        assert_eq!(
            u.uid.as_str(),
            "carol",
            "cn should be used as uid when the entry has no uid attribute"
        );
        assert_eq!(
            u.name.as_str(),
            "carol",
            "name should fall back to the uid when no displayName"
        );
    }

    #[test]
    fn rejects_invalid_uid() {
        let result =
            User::from_attrs(Some("BOB".into()), Some("Bob".into()), None, None).unwrap_err();

        assert_eq!(
            result,
            UserError::InvalidUid,
            "uppercase uid should be rejected"
        );
    }
}
