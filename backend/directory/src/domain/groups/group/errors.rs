#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupError {
    MissingCn,
    InvalidGid,
    InvalidName,
}

impl std::fmt::Display for GroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCn => write!(f, "missing cn"),
            Self::InvalidGid => write!(f, "invalid gid"),
            Self::InvalidName => write!(f, "invalid name"),
        }
    }
}

impl std::error::Error for GroupError {}
