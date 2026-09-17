use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    Empty,
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "password is empty"),
        }
    }
}

impl std::error::Error for PasswordError {}

impl Password {
    pub fn try_new(raw: String) -> Result<Self, PasswordError> {
        if raw.is_empty() {
            return Err(PasswordError::Empty);
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::try_new(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct UpdatePassword {
    pub password: Password,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let pw = Password::try_new("secret".into()).unwrap();

        assert_eq!(pw.as_str(), "secret");
    }

    #[test]
    fn rejects_empty() {
        let err = Password::try_new("".into()).unwrap_err();

        assert_eq!(err, PasswordError::Empty);
    }

    #[test]
    fn update_password_deserialize_validates() {
        let json = r#"{"password":"secret"}"#;

        let up: UpdatePassword = serde_json::from_str(json).unwrap();

        assert_eq!(up.password.as_str(), "secret");
    }

    #[test]
    fn update_password_rejects_empty() {
        let json = r#"{"password":""}"#;

        let err = serde_json::from_str::<UpdatePassword>(json).unwrap_err();

        assert!(
            err.to_string().contains("password is empty"),
            "should propagate PasswordError"
        );
    }
}
