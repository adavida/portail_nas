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
