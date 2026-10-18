use crate::domain::oidc::{ClientIdError, RedirectUriError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    Id(ClientIdError),
    Redirect(RedirectUriError),
    NoRedirects,
    Json(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(e) => write!(f, "client id: {e}"),
            Self::Redirect(e) => write!(f, "client redirect uri: {e}"),
            Self::NoRedirects => write!(f, "client has no redirect uris"),
            Self::Json(e) => write!(f, "clients file invalid json: {e}"),
        }
    }
}

impl std::error::Error for ClientError {}
