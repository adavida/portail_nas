pub mod errors;
pub use errors::NameError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Name(String);

impl Name {
    pub fn try_new(raw: String) -> Result<Self, NameError> {
        let s = raw.trim().to_string();
        if s.is_empty() {
            return Err(NameError::Empty);
        }
        if s.len() > 100 {
            return Err(NameError::InvalidFormat);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let n = Name::try_new("Devs".into()).unwrap();

        assert_eq!(n.as_str(), "Devs", "valid name should be kept as is");
    }

    #[test]
    fn trims_whitespace() {
        let n = Name::try_new("  Devs  ".into()).unwrap();

        assert_eq!(n.as_str(), "Devs", "name should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = Name::try_new("".into()).unwrap_err();

        assert_eq!(err, NameError::Empty, "empty name should be Empty");
    }

    #[test]
    fn rejects_blank() {
        let err = Name::try_new("   ".into()).unwrap_err();

        assert_eq!(err, NameError::Empty, "blank name should be Empty");
    }

    #[test]
    fn rejects_too_long() {
        let err = Name::try_new("a".repeat(101)).unwrap_err();

        assert_eq!(
            err,
            NameError::InvalidFormat,
            "too long name should be InvalidFormat"
        );
    }

    #[test]
    fn deserialize_validates() {
        let n: Name = serde_json::from_str("\"Devs\"").unwrap();

        assert_eq!(n.as_str(), "Devs");
    }

    #[test]
    fn deserialize_rejects_invalid() {
        let json = "\"\"";

        let err = serde_json::from_str::<Name>(json).unwrap_err();

        assert!(
            err.to_string().contains("name is empty"),
            "deserialize should propagate NameError"
        );
    }
}
