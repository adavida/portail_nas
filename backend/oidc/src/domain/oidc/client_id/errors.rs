#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientIdError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for ClientIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "client id is empty"),
            Self::InvalidFormat => write!(f, "client id invalid format"),
        }
    }
}

impl std::error::Error for ClientIdError {}
