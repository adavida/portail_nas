use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Email(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailError {
    InvalidFormat,
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "email invalid format"),
        }
    }
}

impl std::error::Error for EmailError {}

impl Email {
    pub fn try_new(raw: String) -> Result<Self, EmailError> {
        let s = raw.trim().to_string();
        if s.is_empty() {
            return Ok(Self(s));
        }
        if s.len() > 254 {
            return Err(EmailError::InvalidFormat);
        }
        if !s.contains('@') || !s.contains('.') {
            return Err(EmailError::InvalidFormat);
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty() {
        let email = Email::try_new("".into()).unwrap();

        assert!(email.is_empty(), "empty email should be allowed");
        assert_eq!(email.as_str(), "");
    }

    #[test]
    fn accepts_valid() {
        let email = Email::try_new("bob@example.com".into()).unwrap();

        assert_eq!(email.as_str(), "bob@example.com");
    }

    #[test]
    fn trims() {
        let email = Email::try_new(" bob@example.com ".into()).unwrap();

        assert_eq!(email.as_str(), "bob@example.com");
    }

    #[test]
    fn rejects_missing_at() {
        let err = Email::try_new("bobexample.com".into()).unwrap_err();

        assert_eq!(err, EmailError::InvalidFormat);
    }

    #[test]
    fn rejects_missing_dot() {
        let err = Email::try_new("bob@example".into()).unwrap_err();

        assert_eq!(err, EmailError::InvalidFormat);
    }

    #[test]
    fn rejects_too_long() {
        let err = Email::try_new(format!("{}@example.com", "a".repeat(250))).unwrap_err();

        assert_eq!(err, EmailError::InvalidFormat);
    }
}
