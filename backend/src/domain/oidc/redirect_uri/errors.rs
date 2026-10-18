#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectUriError {
    Empty,
    NotHttp,
}

impl std::fmt::Display for RedirectUriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "redirect uri is empty"),
            Self::NotHttp => write!(f, "redirect uri must start with http:// or https://"),
        }
    }
}

impl std::error::Error for RedirectUriError {}
