use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub aud: serde_json::Value,
    pub iss: String,
    pub exp: usize,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.groups.iter().any(|g| g == "admin")
    }
}
