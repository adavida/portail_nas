#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GidError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for GidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "gid is empty"),
            Self::InvalidFormat => write!(f, "gid invalid format"),
        }
    }
}

impl std::error::Error for GidError {}
