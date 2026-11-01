#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "name is empty"),
            Self::InvalidFormat => write!(f, "name invalid format"),
        }
    }
}

impl std::error::Error for NameError {}
