pub mod errors;
pub use errors::CnError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Cn(String);

impl Cn {
    pub fn try_new(raw: String) -> Result<Self, CnError> {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            return Err(CnError::Empty);
        }
        if trimmed.len() < 2 || trimmed.len() > 32 {
            return Err(CnError::InvalidFormat);
        }
        if !trimmed.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        }) {
            return Err(CnError::InvalidFormat);
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Cn {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::try_new(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let cn = Cn::try_new("bob".into()).unwrap();

        assert_eq!(cn.as_str(), "bob", "valid cn should be kept as is");
    }

    #[test]
    fn trims_whitespace() {
        let cn = Cn::try_new("  bob  ".into()).unwrap();

        assert_eq!(cn.as_str(), "bob", "cn should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = Cn::try_new("".into()).unwrap_err();

        assert_eq!(err, CnError::Empty, "empty cn should be Empty");
    }

    #[test]
    fn rejects_blank() {
        let err = Cn::try_new("   ".into()).unwrap_err();

        assert_eq!(err, CnError::Empty, "blank cn should be Empty");
    }

    #[test]
    fn rejects_invalid_chars() {
        let err = Cn::try_new("Bob".into()).unwrap_err();

        assert_eq!(err, CnError::InvalidFormat, "uppercase should be invalid");
    }

    #[test]
    fn rejects_too_short() {
        let err = Cn::try_new("a".into()).unwrap_err();

        assert_eq!(err, CnError::InvalidFormat, "cn len <2 should be invalid");
    }

    #[test]
    fn rejects_too_long() {
        let err = Cn::try_new("a".repeat(33)).unwrap_err();

        assert_eq!(err, CnError::InvalidFormat, "cn len >32 should be invalid");
    }

    #[test]
    fn deserialize_validates() {
        let json = "\"bob\"";

        let cn: Cn = serde_json::from_str(json).unwrap();

        assert_eq!(cn.as_str(), "bob");
    }

    #[test]
    fn deserialize_rejects_invalid() {
        let json = "\"\"";

        let err = serde_json::from_str::<Cn>(json).unwrap_err();

        assert!(
            err.to_string().contains("cn is empty"),
            "deserialize should propagate CnError"
        );
    }
}
