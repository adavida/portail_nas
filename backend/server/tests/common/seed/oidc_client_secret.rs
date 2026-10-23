use serde_json::Value;

/// Reads the portal's real `(client_id, secret)` pair from
/// `OIDC_CLIENTS_FILE` + its `secret_file` — files written by the nix /
/// devenv bootstrap. `None` if either is inaccessible.
pub fn oidc_client_secret() -> Option<(String, String)> {
    let clients = std::env::var("OIDC_CLIENTS_FILE")
        .ok()
        .filter(|s| !s.trim().is_empty())?;
    let raw = std::fs::read_to_string(&clients).ok()?;
    let id = std::env::var("OIDC_CLIENT_ID").ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let entry = v.as_array()?.iter().find(|c| c["id"] == id.as_str())?;
    let name = entry["secret_file"].as_str()?;
    let path = std::path::Path::new(name);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::path::Path::new(&clients).parent()?.join(path)
    };
    let secret = std::fs::read_to_string(path).ok()?;
    Some((id, secret.trim().to_string()))
}
