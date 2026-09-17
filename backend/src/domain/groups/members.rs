use serde::Deserialize;

use crate::domain::users::Uid;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Members(Vec<Uid>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MembersError {
    Empty,
    InvalidUid,
}

impl std::fmt::Display for MembersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(
                f,
                "members is empty: groupOfNames requires at least one member"
            ),
            Self::InvalidUid => write!(f, "members contains invalid uid"),
        }
    }
}

impl std::error::Error for MembersError {}

impl Members {
    pub fn try_new(raw: Vec<Uid>) -> Result<Self, MembersError> {
        if raw.is_empty() {
            return Err(MembersError::Empty);
        }
        Ok(Self(raw))
    }

    pub fn as_slice(&self) -> &[Uid] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Members {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Vec::<Uid>::deserialize(deserializer)?;
        Self::try_new(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let members = Members::try_new(vec![Uid::try_new("alice".into()).unwrap()]).unwrap();

        assert_eq!(members.as_slice().len(), 1);
        assert_eq!(members.as_slice()[0].as_str(), "alice");
    }

    #[test]
    fn rejects_empty() {
        let err = Members::try_new(vec![]).unwrap_err();

        assert_eq!(err, MembersError::Empty);
    }

    #[test]
    fn deserialize_validates() {
        let members: Members = serde_json::from_str(r#"["alice"]"#).unwrap();

        assert_eq!(members.as_slice()[0].as_str(), "alice");
    }

    #[test]
    fn deserialize_rejects_invalid_uid() {
        let err = serde_json::from_str::<Members>(r#"["BAD UID"]"#).unwrap_err();

        assert!(
            err.to_string().contains("invalid"),
            "invalid uid should be rejected, got {err}"
        );
    }

    #[test]
    fn deserialize_rejects_empty_list() {
        let err = serde_json::from_str::<Members>("[]").unwrap_err();

        assert!(
            err.to_string().contains("members is empty"),
            "empty members should be rejected, got {err}"
        );
    }
}
