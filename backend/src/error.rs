#[derive(Debug)]
pub enum AppError {
    Ldap(String),
    NotFound(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ldap(m) => write!(f, "ldap: {m}"),
            Self::NotFound(m) => write!(f, "not found: {m}"),
            Self::Internal(m) => write!(f, "internal: {m}"),
        }
    }
}

impl std::error::Error for AppError {}
