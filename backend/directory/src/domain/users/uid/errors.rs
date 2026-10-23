#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UidError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for UidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "uid is empty"),
            Self::InvalidFormat => write!(f, "uid invalid format"),
        }
    }
}

impl std::error::Error for UidError {}
