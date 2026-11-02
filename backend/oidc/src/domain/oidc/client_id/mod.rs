pub mod errors;
pub use errors::ClientIdError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ClientId(String);

impl ClientId {
    pub fn try_new(raw: String) -> Result<Self, ClientIdError> {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            return Err(ClientIdError::Empty);
        }
        if trimmed.len() > 64
            || !trimmed
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || ".-_".contains(c))
        {
            return Err(ClientIdError::InvalidFormat);
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ClientId {
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
        let id = ClientId::try_new("portail-dev".into()).unwrap();

        assert_eq!(id.as_str(), "portail-dev", "valid id kept as is");
    }

    #[test]
    fn trims_whitespace() {
        let id = ClientId::try_new("  app1  ".into()).unwrap();

        assert_eq!(id.as_str(), "app1", "id should be trimmed");
    }

    #[test]
    fn rejects_empty() {
        let err = ClientId::try_new("".into()).unwrap_err();

        assert_eq!(err, ClientIdError::Empty, "empty id should be Empty");
    }

    #[test]
    fn rejects_uppercase() {
        let err = ClientId::try_new("Portail".into()).unwrap_err();

        assert_eq!(
            err,
            ClientIdError::InvalidFormat,
            "uppercase should be invalid"
        );
    }

    #[test]
    fn rejects_too_long() {
        let err = ClientId::try_new("a".repeat(65)).unwrap_err();

        assert_eq!(
            err,
            ClientIdError::InvalidFormat,
            "len >64 should be invalid"
        );
    }
}
