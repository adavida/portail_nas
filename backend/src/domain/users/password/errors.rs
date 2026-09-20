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
