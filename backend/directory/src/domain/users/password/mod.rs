pub mod errors;
pub use errors::PasswordError;

use serde::Deserialize;

use portail_core::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    pub fn try_new(raw: String) -> Result<Self, PasswordError> {
        let min_length = Config::global().password_min_length;
        if raw.is_empty() {
            return Err(PasswordError::Empty);
        }
        if raw.chars().count() < min_length {
            return Err(PasswordError::TooShort(min_length));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::try_new(raw).map_err(serde::de::Error::custom)
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
    fn rejects_short() {
        let err = Password::try_new("x".into()).unwrap_err();

        assert_eq!(
            err,
            PasswordError::TooShort(2),
            "len < configured minimum should be TooShort: {err}"
        );
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
