#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasswordError {
    Empty,
    TooShort(usize),
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "password is empty"),
            Self::TooShort(min) => write!(f, "password is shorter than {min} characters"),
        }
    }
}

impl std::error::Error for PasswordError {}
