pub mod errors;
pub use errors::ClientSecretError;

use serde::Deserialize;

/// Client secret — never serialized (never exposed in a response).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSecret(String);

impl ClientSecret {
    pub fn try_new(raw: String) -> Result<Self, ClientSecretError> {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            return Err(ClientSecretError::Empty);
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ClientSecret {
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
        let secret = ClientSecret::try_new("s3cr3t".into()).unwrap();

        assert_eq!(secret.as_str(), "s3cr3t", "valid secret kept as is");
    }

    #[test]
    fn rejects_empty() {
        let err = ClientSecret::try_new("  ".into()).unwrap_err();

        assert_eq!(
            err,
            ClientSecretError::Empty,
            "blank secret should be Empty"
        );
    }
}
