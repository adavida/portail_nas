use std::path::Path;

use portail_core::config::Config;

pub fn oidc_client_secret() -> Option<(String, String)> {
    let config = Config::global();
    let entry = config
        .oidc
        .clients
        .iter()
        .find(|c| c.id == config.oidc.client_id)?;
    let name = entry.secret_file.as_deref()?;
    let path = Path::new(name);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(&config.oidc.secrets_dir).join(path)
    };
    let secret = std::fs::read_to_string(path).ok()?;
    Some((config.oidc.client_id.clone(), secret.trim().to_string()))
}
