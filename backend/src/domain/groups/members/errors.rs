#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MembersError {
    Empty,
    InvalidUid,
}

impl std::fmt::Display for MembersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(
                f,
                "members is empty: groupOfNames requires at least one member"
            ),
            Self::InvalidUid => write!(f, "members contains invalid uid"),
        }
    }
}

impl std::error::Error for MembersError {}
