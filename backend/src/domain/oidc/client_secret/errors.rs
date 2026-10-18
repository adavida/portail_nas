#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientSecretError {
    Empty,
}

impl std::fmt::Display for ClientSecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "client secret is empty"),
        }
    }
}

impl std::error::Error for ClientSecretError {}
