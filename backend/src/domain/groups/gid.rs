use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Gid(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GidError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for GidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "gid is empty"),
            Self::InvalidFormat => write!(f, "gid invalid format"),
        }
    }
}

impl std::error::Error for GidError {}

impl Gid {
    pub fn try_new(raw: String) -> Result<Self, GidError> {
        let s = raw.trim().to_string();
        if s.is_empty() {
            return Err(GidError::Empty);
        }
        if s.len() < 2 || s.len() > 32 {
            return Err(GidError::InvalidFormat);
        }
        if !s.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        }) {
            return Err(GidError::InvalidFormat);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Gid {
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
        let gid = Gid::try_new("devs".into()).unwrap();

        assert_eq!(gid.as_str(), "devs", "valid gid should be kept as is");
    }

    #[test]
    fn trims_whitespace() {
        let gid = Gid::try_new("  devs  ".into()).unwrap();

        assert_eq!(gid.as_str(), "devs", "gid should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = Gid::try_new("".into()).unwrap_err();

        assert_eq!(err, GidError::Empty, "empty gid should be Empty");
    }

    #[test]
    fn rejects_invalid_chars() {
        let err = Gid::try_new("Devs!".into()).unwrap_err();

        assert_eq!(err, GidError::InvalidFormat, "uppercase/symbols invalid");
    }

    #[test]
    fn rejects_too_short() {
        let err = Gid::try_new("a".into()).unwrap_err();

        assert_eq!(err, GidError::InvalidFormat, "gid len <2 should be invalid");
    }

    #[test]
    fn rejects_too_long() {
        let err = Gid::try_new("a".repeat(33)).unwrap_err();

        assert_eq!(
            err,
            GidError::InvalidFormat,
            "gid len >32 should be invalid"
        );
    }

    #[test]
    fn deserialize_validates() {
        let gid: Gid = serde_json::from_str("\"devs\"").unwrap();

        assert_eq!(gid.as_str(), "devs");
    }
}
