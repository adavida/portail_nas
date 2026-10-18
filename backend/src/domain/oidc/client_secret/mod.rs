use serde::Deserialize;

/// Client secret — never serialized (never exposed in a response).
/// An empty secret marks a **public client** (OAuth debugger, native apps):
/// no secret at the token endpoint, PKCE S256 is then mandatory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSecret(String);

impl ClientSecret {
    pub fn new(raw: String) -> Self {
        Self(raw.trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_public(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de> Deserialize<'de> for ClientSecret {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let secret = ClientSecret::new("  s3cr3t  ".into());

        assert_eq!(secret.as_str(), "s3cr3t", "valid secret trimmed");
        assert!(!secret.is_public(), "non-empty secret is confidential");
    }

    #[test]
    fn empty_secret_marks_public_client() {
        let secret = ClientSecret::new("".into());

        assert!(secret.is_public(), "empty secret must mark a public client");
    }

    #[test]
    fn blank_secret_trims_to_public() {
        let secret = ClientSecret::new("   ".into());

        assert!(secret.is_public(), "blank secret trims to public client");
    }
}
