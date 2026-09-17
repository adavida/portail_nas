use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Uid(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UidError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for UidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "uid is empty"),
            Self::InvalidFormat => write!(f, "uid invalid format"),
        }
    }
}

impl std::error::Error for UidError {}

impl Uid {
    pub fn try_new(raw: String) -> Result<Self, UidError> {
        let s = raw.trim().to_string();
        if s.is_empty() {
            return Err(UidError::Empty);
        }
        if s.len() < 2 || s.len() > 32 {
            return Err(UidError::InvalidFormat);
        }
        if !s.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        }) {
            return Err(UidError::InvalidFormat);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Uid {
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
        let uid = Uid::try_new("bob".into()).unwrap();

        assert_eq!(uid.as_str(), "bob", "valid uid should be kept as is");
    }

    #[test]
    fn trims_whitespace() {
        let uid = Uid::try_new("  bob  ".into()).unwrap();

        assert_eq!(uid.as_str(), "bob", "uid should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = Uid::try_new("".into()).unwrap_err();

        assert_eq!(err, UidError::Empty, "empty uid should be Empty");
    }

    #[test]
    fn rejects_blank() {
        let err = Uid::try_new("   ".into()).unwrap_err();

        assert_eq!(err, UidError::Empty, "blank uid should be Empty");
    }

    #[test]
    fn rejects_invalid_chars() {
        let err = Uid::try_new("Bob".into()).unwrap_err();

        assert_eq!(err, UidError::InvalidFormat, "uppercase should be invalid");
    }

    #[test]
    fn rejects_too_short() {
        let err = Uid::try_new("a".into()).unwrap_err();

        assert_eq!(err, UidError::InvalidFormat, "uid len <2 should be invalid");
    }

    #[test]
    fn rejects_too_long() {
        let err = Uid::try_new("a".repeat(33)).unwrap_err();

        assert_eq!(
            err,
            UidError::InvalidFormat,
            "uid len >32 should be invalid"
        );
    }

    #[test]
    fn deserialize_validates() {
        let json = "\"bob\"";

        let uid: Uid = serde_json::from_str(json).unwrap();

        assert_eq!(uid.as_str(), "bob");
    }

    #[test]
    fn deserialize_rejects_invalid() {
        let json = "\"\"";

        let err = serde_json::from_str::<Uid>(json).unwrap_err();

        assert!(
            err.to_string().contains("uid is empty"),
            "deserialize should propagate UidError"
        );
    }
}
