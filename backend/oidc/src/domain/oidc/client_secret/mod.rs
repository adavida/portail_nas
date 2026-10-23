/// Client secret as loaded from its `secret_file` — never stored in the
/// clients JSON, never exposed in a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSecret(String);

impl ClientSecret {
    pub fn new(raw: String) -> Self {
        Self(raw.trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        let secret = ClientSecret::new("  s3cr3t  ".into());

        assert_eq!(secret.as_str(), "s3cr3t", "valid secret trimmed");
    }
}
