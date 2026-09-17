use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Description(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionError {
    Empty,
}

impl std::fmt::Display for DescriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "description is empty"),
        }
    }
}

impl std::error::Error for DescriptionError {}

impl Description {
    pub fn try_new(raw: String) -> Result<Self, DescriptionError> {
        let s = raw.trim().to_string();
        if s.is_empty() {
            return Err(DescriptionError::Empty);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Description {
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
        let d = Description::try_new("Devs team".into()).unwrap();

        assert_eq!(d.as_str(), "Devs team");
    }

    #[test]
    fn trims_whitespace() {
        let d = Description::try_new("  Devs  ".into()).unwrap();

        assert_eq!(d.as_str(), "Devs", "description should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = Description::try_new("   ".into()).unwrap_err();

        assert_eq!(err, DescriptionError::Empty);
    }

    #[test]
    fn deserialize_validates() {
        let d: Description = serde_json::from_str("\"x\"").unwrap();

        assert_eq!(d.as_str(), "x");
    }
}
