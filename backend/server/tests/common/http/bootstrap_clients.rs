use portail_core::env::Env;
use portail_oidc::auth::secrets::clients_path;

/// Test bootstrap: the clients file is written by nix in production, so the
/// test app builders create a minimal one (enabled portal client) when it is
/// absent. Idempotent — an existing file is never touched.
pub fn bootstrap_clients_file(env: &Env) {
    let path = clients_path(env);
    if path.exists() {
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let entry = serde_json::json!([{
        "id": env.oidc_client_id,
        "secret_file": format!("{}_secret", env.oidc_client_id),
        "enabled": true,
        "redirect_uris": [env.oidc_redirect_uri],
        "scopes": ["openid", "profile", "email", "groups"],
    }]);
    let _ = std::fs::write(&path, entry.to_string());
}
