use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomClaim {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub group: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientEntry {
    pub id: String,
    #[serde(default)]
    pub secret_file: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub custom_claims: Vec<CustomClaim>,
}
