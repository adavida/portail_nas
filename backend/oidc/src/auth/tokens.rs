use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    auth::{Claims, key, store},
    domain::oidc::Client,
};
use portail_core::env::Env;
use portail_core::error::AppError;

pub const ACCESS_TOKEN_TTL: i64 = 3600;

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

/// PKCE S256: `challenge == base64url(sha256(verifier))` (RFC 7636).
pub fn check_pkce(verifier: Option<&str>, challenge: Option<&str>) -> bool {
    match (verifier, challenge) {
        (Some(v), Some(c)) => URL_SAFE_NO_PAD.encode(Sha256::digest(v.as_bytes())) == c,
        (None, Some(_)) => false,
        (_, None) => true,
    }
}

pub fn clients(env: &Env) -> Result<Vec<Client>, AppError> {
    let raw = crate::auth::secrets::read_clients(env).map_err(AppError::Internal)?;
    Client::parse_all(&raw).map_err(|e| AppError::Internal(e.to_string()))
}

/// `authorization_code` grant: code one-shot, client + redirect_uri + PKCE
/// verified, then signed tokens. Errors in OAuth format (`invalid_client`,
/// `invalid_grant`). Confidential clients must present the secret read from
/// their `secret_file`; public clients (no `secret_file`, e.g. the OAuth
/// debugger) skip it but must have sent a PKCE S256 challenge at authorize.
/// Disabled clients (`enabled: false` in the file) are rejected as if unknown.
pub fn exchange(
    env: &Env,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
    verifier: Option<&str>,
) -> Result<TokenResponse, AppError> {
    let clients = clients(env)?;
    let Some(client) = Client::find_in(&clients, client_id) else {
        return Err(AppError::BadRequest("invalid_client".into()));
    };
    if !client.enabled {
        return Err(AppError::BadRequest("invalid_client".into()));
    }
    let stored = crate::auth::secrets::stored_secret(env, client)
        .map_err(AppError::Internal)?
        .map(|s| s.as_str().to_string())
        .unwrap_or_default();
    if stored != client_secret {
        return Err(AppError::BadRequest("invalid_client".into()));
    }
    let Some(flow) = store::take_code(code) else {
        return Err(AppError::BadRequest("invalid_grant".into()));
    };
    if flow.client_id != client.id.as_str() || flow.redirect_uri != redirect_uri {
        return Err(AppError::BadRequest("invalid_grant".into()));
    }
    if client.is_public() && flow.challenge.is_none() {
        return Err(AppError::BadRequest(
            "invalid_grant: public clients must use PKCE".into(),
        ));
    }
    if !check_pkce(verifier, flow.challenge.as_deref()) {
        return Err(AppError::BadRequest("invalid_grant".into()));
    }
    tokens_for(env, &flow)
}

fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}

fn claims(env: &Env, flow: &store::AuthFlow) -> Claims {
    Claims {
        sub: flow.sub.clone(),
        aud: serde_json::json!(flow.client_id),
        iss: env.oidc_issuer_url.clone(),
        exp: now() + ACCESS_TOKEN_TTL as usize,
        groups: flow.groups.clone(),
        email: flow
            .email
            .clone()
            .filter(|e| !e.is_empty() && flow.scopes.iter().any(|s| s == "email")),
        preferred_username: Some(flow.sub.clone()),
        nonce: flow.nonce.clone(),
    }
}

fn sign(claims: &Claims) -> Result<String, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    let header = Header {
        kid: Some(key.kid.clone()),
        ..Header::new(Algorithm::RS256)
    };
    encode(&header, claims, key.encoding())
        .map_err(|e| AppError::Internal(format!("token signing failed: {e}")))
}

pub fn tokens_for(env: &Env, flow: &store::AuthFlow) -> Result<TokenResponse, AppError> {
    let access_token = sign(&claims(env, flow))?;
    let id_token = access_token.clone();
    Ok(TokenResponse {
        access_token,
        id_token,
        token_type: "bearer",
        expires_in: ACCESS_TOKEN_TTL,
    })
}

/// Local verification (provider key) — issuer, portal audience, expiry.
pub fn verify(env: &Env, token: &str) -> Result<Claims, AppError> {
    decode_token(env, token, Some(&env.oidc_client_id))
}

/// Like `verify` with an explicit audience (self-test for another client).
pub fn verify_with_aud(env: &Env, token: &str, aud: &str) -> Result<Claims, AppError> {
    decode_token(env, token, Some(aud))
}

/// Verification for the standard `/userinfo` endpoint: any audience, the
/// audience check stays on each resource server (portal middleware).
pub fn verify_any_aud(env: &Env, token: &str) -> Result<Claims, AppError> {
    decode_token(env, token, None)
}

fn decode_token(env: &Env, token: &str, aud: Option<&str>) -> Result<Claims, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    let mut validation = Validation::new(Algorithm::RS256);
    match aud {
        Some(aud) => validation.set_audience(&[aud]),
        None => validation.validate_aud = false,
    }
    validation.set_issuer(&[&env.oidc_issuer_url]);
    decode::<Claims>(token, key.decoding(), &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("token invalid: {e}")))
}

/// Extracts `client_id:secret` from an `Authorization: Basic ...` header.
pub fn basic_credentials(header: &str) -> Option<(String, String)> {
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = String::from_utf8(STANDARD.decode(encoded).ok()?).ok()?;
    let (id, secret) = decoded.split_once(':')?;
    Some((id.to_string(), secret.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLIC_ID: &str = "oauth-debugger";
    const PUBLIC_REDIRECT: &str = "http://127.0.0.1:3030/callback";

    fn env_with_public_client(tag: &str) -> Env {
        let mut env = portail_core::env::Env::global().clone();
        env.oidc_secrets_dir = format!(
            "{}/portail-public-test-{tag}-{}",
            std::env::temp_dir().display(),
            std::process::id()
        );
        env.oidc_clients_file = format!("{}/clients.json", env.oidc_secrets_dir);
        std::fs::create_dir_all(&env.oidc_secrets_dir).unwrap();
        std::fs::write(
            crate::auth::secrets::clients_path(&env),
            format!(
                r#"[{{"id":"{PUBLIC_ID}","enabled":true,"redirect_uris":["{PUBLIC_REDIRECT}"],"scopes":["openid","profile","email","groups"]}}]"#
            ),
        )
        .unwrap();
        env
    }

    fn public_flow(challenge: Option<&str>) -> store::AuthFlow {
        store::AuthFlow {
            client_id: PUBLIC_ID.into(),
            redirect_uri: PUBLIC_REDIRECT.into(),
            state: None,
            nonce: None,
            scopes: vec!["openid".into()],
            challenge: challenge.map(String::from),
            sub: "tester".into(),
            groups: vec![],
            email: None,
        }
    }

    #[test]
    fn pkce_rfc7636_vector() {
        let ok = check_pkce(
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"),
        );

        assert!(ok, "RFC 7636 S256 vector must validate");
    }

    #[test]
    fn pkce_rejects_wrong_verifier() {
        let ok = check_pkce(
            Some("not-the-verifier"),
            Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"),
        );

        assert!(!ok, "wrong verifier must fail");
    }

    #[test]
    fn pkce_rejects_missing_verifier_when_challenged() {
        let ok = check_pkce(None, Some("challenge"));

        assert!(!ok, "missing verifier with challenge must fail");
    }

    #[test]
    fn basic_credentials_parses() {
        let credentials = STANDARD.encode("portail-dev:secret");

        let parsed = basic_credentials(&format!("Basic {credentials}"));

        assert_eq!(
            parsed,
            Some(("portail-dev".to_string(), "secret".to_string())),
            "Basic header should split id:secret"
        );
    }

    #[test]
    fn basic_credentials_rejects_garbage() {
        let parsed = basic_credentials("Bearer abc");

        assert_eq!(parsed, None, "non-Basic header should be None");
    }

    #[test]
    fn exchange_unknown_code_is_invalid_grant() {
        let env = portail_core::env::Env::global();
        let Ok(clients) = clients(env) else { return };
        let Some(client) = clients.iter().find(|c| c.enabled) else {
            return;
        };
        let Ok(stored) = crate::auth::secrets::stored_secret(env, client) else {
            return;
        };
        let presented = stored.map(|s| s.as_str().to_string()).unwrap_or_default();

        let err = exchange(
            env,
            client.id.as_str(),
            &presented,
            "no-such-code",
            client.redirect_uris[0].as_str(),
            None,
        )
        .unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m == "invalid_grant"),
            "unknown code should be invalid_grant, got {err}"
        );
    }

    #[test]
    fn exchange_public_client_exchanges_form_credentials_with_pkce() {
        let env = env_with_public_client("ok");
        let code = store::save_code(public_flow(Some(
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        )))
        .unwrap();

        let tokens = exchange(
            &env,
            PUBLIC_ID,
            "",
            &code,
            PUBLIC_REDIRECT,
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        );

        let tokens = tokens.expect("public client with PKCE should get tokens");
        assert!(
            !tokens.access_token.is_empty(),
            "access_token must be issued to a public client"
        );
        let claims = verify_any_aud(&env, &tokens.access_token)
            .expect("token for another audience must verify for userinfo");
        assert_eq!(claims.sub, "tester", "userinfo verification keeps sub");
    }

    #[test]
    fn exchange_public_client_without_pkce_is_rejected() {
        let env = env_with_public_client("nopkce");
        let code = store::save_code(public_flow(None)).unwrap();

        let err = exchange(&env, PUBLIC_ID, "", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m.contains("PKCE")),
            "public client without challenge must be refused, got {err}"
        );
    }

    #[test]
    fn exchange_public_client_rejects_presented_secret() {
        let env = env_with_public_client("secret");
        let code = store::save_code(public_flow(Some("chal"))).unwrap();

        let err = exchange(&env, PUBLIC_ID, "not-empty", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m == "invalid_client"),
            "public client must refuse a presented secret, got {err}"
        );
    }

    fn env_with_secret_client(tag: &str, enabled: bool) -> Env {
        let mut env = portail_core::env::Env::global().clone();
        env.oidc_secrets_dir = format!(
            "{}/portail-confid-test-{tag}-{}",
            std::env::temp_dir().display(),
            std::process::id()
        );
        env.oidc_clients_file = format!("{}/clients.json", env.oidc_secrets_dir);
        std::fs::create_dir_all(&env.oidc_secrets_dir).unwrap();
        std::fs::write(
            crate::auth::secrets::clients_path(&env),
            format!(
                r#"[{{"id":"confid","secret_file":"confid_secret","enabled":{enabled},"redirect_uris":["{PUBLIC_REDIRECT}"],"scopes":["openid"]}}]"#
            ),
        )
        .unwrap();
        std::fs::write(
            crate::auth::secrets::secret_path(&env, "confid_secret"),
            "s3cr3t\n",
        )
        .unwrap();
        env
    }

    #[test]
    fn exchange_confidential_client_reads_secret_from_file() {
        let env = env_with_secret_client("ok", true);
        let flow = store::AuthFlow {
            client_id: "confid".into(),
            redirect_uri: PUBLIC_REDIRECT.into(),
            state: None,
            nonce: None,
            scopes: vec!["openid".into()],
            challenge: None,
            sub: "tester".into(),
            groups: vec![],
            email: None,
        };
        let code = store::save_code(flow).unwrap();

        let err =
            exchange(&env, "confid", "wrong-secret", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m == "invalid_client"),
            "wrong secret must be invalid_client, got {err}"
        );
        let tokens = exchange(&env, "confid", "s3cr3t", &code, PUBLIC_REDIRECT, None)
            .expect("secret from the file must authenticate the client");
        assert!(!tokens.access_token.is_empty(), "tokens expected");
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }

    #[test]
    fn exchange_disabled_client_is_invalid_client() {
        let env = env_with_secret_client("disabled", false);
        let code = store::save_code(public_flow(Some("chal"))).unwrap();

        let err = exchange(&env, "confid", "s3cr3t", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m == "invalid_client"),
            "disabled client must be invalid_client, got {err}"
        );
        let _ = std::fs::remove_dir_all(&env.oidc_secrets_dir);
    }
}
