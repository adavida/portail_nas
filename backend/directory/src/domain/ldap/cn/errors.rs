#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CnError {
    Empty,
    InvalidFormat,
}

impl std::fmt::Display for CnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "cn is empty"),
            Self::InvalidFormat => write!(f, "cn invalid format"),
        }
    }
}

impl std::error::Error for CnError {}
