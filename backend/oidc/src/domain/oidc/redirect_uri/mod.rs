pub mod errors;
pub use errors::RedirectUriError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct RedirectUri(String);

impl RedirectUri {
    pub fn try_new(raw: String) -> Result<Self, RedirectUriError> {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            return Err(RedirectUriError::Empty);
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(RedirectUriError::NotHttp);
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RedirectUri {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::try_new(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https() {
        let http = RedirectUri::try_new("http://localhost:5173/callback".into()).unwrap();
        let https = RedirectUri::try_new("https://app.example.com/cb".into()).unwrap();

        assert_eq!(http.as_str(), "http://localhost:5173/callback");
        assert_eq!(https.as_str(), "https://app.example.com/cb");
    }

    #[test]
    fn rejects_empty() {
        let err = RedirectUri::try_new("".into()).unwrap_err();

        assert_eq!(err, RedirectUriError::Empty, "empty uri should be Empty");
    }

    #[test]
    fn rejects_non_http_scheme() {
        let err = RedirectUri::try_new("ftp://example.com".into()).unwrap_err();

        assert_eq!(
            err,
            RedirectUriError::NotHttp,
            "non-http scheme should be NotHttp"
        );
    }
}
