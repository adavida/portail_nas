#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    MissingUid,
    InvalidUid,
    InvalidName,
    InvalidEmail,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingUid => write!(f, "missing uid"),
            Self::InvalidUid => write!(f, "invalid uid"),
            Self::InvalidName => write!(f, "invalid name"),
            Self::InvalidEmail => write!(f, "invalid email"),
        }
    }
}

impl std::error::Error for UserError {}
