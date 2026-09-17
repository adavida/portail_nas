use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct UpdatePassword {
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    MissingPassword,
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPassword => write!(f, "missing password"),
        }
    }
}

impl std::error::Error for PasswordError {}

impl UpdatePassword {
    pub fn validate(&self) -> Result<(), PasswordError> {
        if self.password.is_empty() {
            return Err(PasswordError::MissingPassword);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ok() {
        assert!(
            UpdatePassword {
                password: "secret".into()
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn missing_password() {
        assert_eq!(
            UpdatePassword {
                password: "".into()
            }
            .validate()
            .unwrap_err(),
            PasswordError::MissingPassword
        );
    }
}
