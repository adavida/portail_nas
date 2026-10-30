use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub iat: usize,
    pub exp: usize,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub azp: Option<String>,
    #[serde(default, flatten)]
    pub custom_claims: HashMap<String, String>,
}

impl Claims {
    pub fn is_admin(&self) -> bool {
        self.groups.iter().any(|g| g == "admin")
    }
}
