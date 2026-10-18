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
    env::Env,
    error::AppError,
};

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
/// `invalid_grant`).
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
    if client.secret.as_str() != client_secret {
        return Err(AppError::BadRequest("invalid_client".into()));
    }
    let Some(flow) = store::take_code(code) else {
        return Err(AppError::BadRequest("invalid_grant".into()));
    };
    if flow.client_id != client.id.as_str() || flow.redirect_uri != redirect_uri {
        return Err(AppError::BadRequest("invalid_grant".into()));
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
    verify_with_aud(env, token, &env.oidc_client_id)
}

/// Like `verify` with an explicit audience (self-test for another client).
pub fn verify_with_aud(env: &Env, token: &str, aud: &str) -> Result<Claims, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[aud]);
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
        let env = crate::env::Env::global();
        let Ok(clients) = clients(env) else { return };
        let Some(client) = clients.first() else {
            return;
        };

        let err = exchange(
            env,
            client.id.as_str(),
            client.secret.as_str(),
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
}
