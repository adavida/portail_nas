use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    auth::{Claims, key, store},
    domain::oidc::Client,
};
use portail_core::config::Config;
use portail_core::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

pub fn check_pkce(verifier: Option<&str>, challenge: Option<&str>) -> bool {
    match (verifier, challenge) {
        (Some(v), Some(c)) => URL_SAFE_NO_PAD.encode(Sha256::digest(v.as_bytes())) == c,
        (None, Some(_)) => false,
        (_, None) => true,
    }
}

pub fn clients(config: &Config) -> Result<Vec<Client>, AppError> {
    crate::auth::secrets::clients(config).map_err(AppError::Internal)
}

pub fn issuer(config: &Config) -> String {
    config.oidc_issuer_url.trim_end_matches('/').to_string()
}

fn secrets_match(presented: &str, stored: &str) -> bool {
    Sha256::digest(presented) == Sha256::digest(stored)
}

pub fn exchange(
    config: &Config,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
    verifier: Option<&str>,
) -> Result<TokenResponse, AppError> {
    let clients = clients(config)?;
    let Some(client) = Client::find_in(&clients, client_id) else {
        return Err(AppError::InvalidClient("invalid_client".into()));
    };
    if !client.enabled {
        return Err(AppError::InvalidClient("invalid_client".into()));
    }
    let stored = crate::auth::secrets::stored_secret(config, client)
        .map_err(AppError::Internal)?
        .map(|s| s.as_str().to_string())
        .unwrap_or_default();
    if !secrets_match(client_secret, &stored) {
        return Err(AppError::InvalidClient("invalid_client".into()));
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
    tokens_for(config, &flow)
}

fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
}

pub fn claims(config: &Config, flow: &store::AuthFlow) -> Result<Claims, AppError> {
    let mut custom_claims = HashMap::new();
    if let Some(entry) = config.oidc_clients.iter().find(|e| e.id == flow.client_id) {
        for rule in &entry.custom_claims {
            if rule.group.is_empty() || flow.groups.contains(&rule.group) {
                custom_claims.insert(rule.name.clone(), rule.value.clone());
            }
        }
    }
    let groups = if flow.scopes.iter().any(|s| s == "groups") {
        flow.groups.clone()
    } else {
        Vec::new()
    };
    Ok(Claims {
        sub: crate::auth::subject::subject_for(config, &flow.client_id, &flow.sub)
            .map_err(AppError::Internal)?,
        aud: flow.client_id.clone(),
        iss: issuer(config),
        iat: now(),
        exp: now() + config.oidc_access_ttl.as_secs() as usize,
        groups,
        email: flow
            .email
            .clone()
            .filter(|e| !e.is_empty() && flow.scopes.iter().any(|s| s == "email")),
        preferred_username: Some(flow.sub.clone()),
        nonce: flow.nonce.clone(),
        azp: None,
        custom_claims,
    })
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

pub fn tokens_for(config: &Config, flow: &store::AuthFlow) -> Result<TokenResponse, AppError> {
    let mut id_claims = claims(config, flow)?;
    id_claims.azp = Some(flow.client_id.clone());
    let mut access_claims = claims(config, flow)?;
    access_claims.nonce = None;
    Ok(TokenResponse {
        access_token: sign(&access_claims)?,
        id_token: sign(&id_claims)?,
        token_type: "Bearer",
        expires_in: config.oidc_access_ttl.as_secs(),
    })
}

pub fn verify(config: &Config, token: &str) -> Result<Claims, AppError> {
    decode_token(config, token, Some(&config.oidc_client_id))
}

pub fn verify_with_aud(config: &Config, token: &str, aud: &str) -> Result<Claims, AppError> {
    decode_token(config, token, Some(aud))
}

pub fn verify_any_aud(config: &Config, token: &str) -> Result<Claims, AppError> {
    decode_token(config, token, None)
}

fn decode_token(config: &Config, token: &str, aud: Option<&str>) -> Result<Claims, AppError> {
    let key = key::global().ok_or_else(|| {
        AppError::Internal("signing key not available: generate it in the OIDC secrets dir".into())
    })?;
    let mut validation = Validation::new(Algorithm::RS256);
    match aud {
        Some(aud) => validation.set_audience(&[aud]),
        None => validation.validate_aud = false,
    }
    validation.set_issuer(&[&issuer(config)]);
    decode::<Claims>(token, key.decoding(), &validation)
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("token invalid: {e}")))
}

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

    fn env_with_public_client(tag: &str) -> Config {
        let mut config = portail_core::config::Config::global().clone();
        config.oidc_secrets_dir = format!(
            "{}/portail-public-test-{tag}-{}",
            std::env::temp_dir().display(),
            std::process::id()
        );
        std::fs::create_dir_all(&config.oidc_secrets_dir).unwrap();
        config.db_path = format!("{}/portail.db", config.oidc_secrets_dir);
        config.oidc_clients = vec![portail_core::config::ClientEntry {
            id: PUBLIC_ID.into(),
            secret_file: None,
            enabled: true,
            redirect_uris: vec![PUBLIC_REDIRECT.into()],
            scopes: vec![
                "openid".into(),
                "profile".into(),
                "email".into(),
                "groups".into(),
            ],
            custom_claims: vec![
                portail_core::config::CustomClaim {
                    name: "role".into(),
                    value: "tester".into(),
                    group: String::new(),
                },
                portail_core::config::CustomClaim {
                    name: "superpower".into(),
                    value: "fly".into(),
                    group: "birds".into(),
                },
            ],
        }];
        config
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
            groups: vec!["birds".into()],
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
        let config = portail_core::config::Config::global();
        let Ok(clients) = clients(config) else { return };
        let Some(client) = clients.iter().find(|c| c.enabled) else {
            return;
        };
        let Ok(stored) = crate::auth::secrets::stored_secret(config, client) else {
            return;
        };
        let presented = stored.map(|s| s.as_str().to_string()).unwrap_or_default();

        let err = exchange(
            config,
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
        let config = env_with_public_client("ok");
        let code = store::save_code(public_flow(Some(
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        )))
        .unwrap();

        let tokens = exchange(
            &config,
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
        let claims = verify_any_aud(&config, &tokens.access_token)
            .expect("token for another audience must verify for userinfo");
        assert_eq!(
            claims.sub,
            crate::auth::subject::subject_for(&config, PUBLIC_ID, "tester").unwrap(),
            "sub must be the persisted pairwise uuid for this client+user"
        );
        assert_eq!(claims.sub.len(), 36, "sub must be a dashed uuid");
        assert_ne!(claims.sub, "tester", "sub must not leak the uid");
        assert_eq!(
            claims.preferred_username.as_deref(),
            Some("tester"),
            "preferred_username keeps the readable uid"
        );
    }

    #[test]
    fn issued_tokens_carry_iat_claim() {
        let config = env_with_public_client("iat");
        let code = store::save_code(public_flow(Some(
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        )))
        .unwrap();

        let tokens = exchange(
            &config,
            PUBLIC_ID,
            "",
            &code,
            PUBLIC_REDIRECT,
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        )
        .expect("public client with PKCE should get tokens");
        let claims = verify_any_aud(&config, &tokens.id_token).expect("id_token must verify");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        assert!(
            claims.iat > 0 && claims.iat <= now,
            "id_token iat must be present and not in the future, got {}",
            claims.iat
        );
    }

    #[test]
    fn issued_id_token_carries_client_custom_claims() {
        let config = env_with_public_client("customclaims");
        let code = store::save_code(public_flow(Some(
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
        )))
        .unwrap();

        let tokens = exchange(
            &config,
            PUBLIC_ID,
            "",
            &code,
            PUBLIC_REDIRECT,
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        )
        .expect("public client with PKCE should get tokens");
        let claims = verify_any_aud(&config, &tokens.id_token).expect("id_token must verify");

        let role = claims
            .custom_claims
            .get("role")
            .cloned()
            .unwrap_or_default();
        let superpower = claims
            .custom_claims
            .get("superpower")
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            role, "tester",
            "unconditional rule must be signed into the id_token"
        );
        assert_eq!(
            superpower, "fly",
            "group-scoped rule must be signed for a member of that group"
        );
        assert!(
            claims.groups.is_empty(),
            "groups claim itself stays filtered by scope, got {:?}",
            claims.groups
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn group_scoped_custom_claim_absent_for_non_member() {
        let config = env_with_public_client("customclaimsneg");
        let mut flow = public_flow(Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"));
        flow.groups = vec![];
        let code = store::save_code(flow).unwrap();

        let tokens = exchange(
            &config,
            PUBLIC_ID,
            "",
            &code,
            PUBLIC_REDIRECT,
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        )
        .expect("public client with PKCE should get tokens");
        let claims = verify_any_aud(&config, &tokens.id_token).expect("id_token must verify");

        assert!(
            !claims.custom_claims.contains_key("superpower"),
            "non-members must not get the group-scoped claim: {:?}",
            claims.custom_claims
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn exchange_public_client_without_pkce_is_rejected() {
        let config = env_with_public_client("nopkce");
        let code = store::save_code(public_flow(None)).unwrap();

        let err = exchange(&config, PUBLIC_ID, "", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::BadRequest(m) if m.contains("PKCE")),
            "public client without challenge must be refused, got {err}"
        );
    }

    #[test]
    fn exchange_public_client_rejects_presented_secret() {
        let config = env_with_public_client("secret");
        let code = store::save_code(public_flow(Some("chal"))).unwrap();

        let err = exchange(
            &config,
            PUBLIC_ID,
            "not-empty",
            &code,
            PUBLIC_REDIRECT,
            None,
        )
        .unwrap_err();

        assert!(
            matches!(&err, AppError::InvalidClient(m) if m == "invalid_client"),
            "public client must refuse a presented secret, got {err}"
        );
    }

    #[test]
    fn id_token_and_access_token_are_distinct_jwt() {
        let config = env_with_public_client("split");
        let mut flow = public_flow(Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"));
        flow.nonce = Some("n-123".into());
        let code = store::save_code(flow).unwrap();

        let tokens = exchange(
            &config,
            PUBLIC_ID,
            "",
            &code,
            PUBLIC_REDIRECT,
            Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        )
        .expect("public client with PKCE should get tokens");

        assert_eq!(
            tokens.token_type, "Bearer",
            "token_type must be the case-sensitive RFC 6749 value"
        );
        assert_ne!(
            tokens.id_token, tokens.access_token,
            "id_token and access_token must be distinct"
        );
        let id_claims = verify_any_aud(&config, &tokens.id_token).expect("id_token must verify");
        let access_claims =
            verify_any_aud(&config, &tokens.access_token).expect("access_token must verify");
        assert_eq!(
            id_claims.nonce.as_deref(),
            Some("n-123"),
            "nonce belongs to the id_token only"
        );
        assert_eq!(
            id_claims.azp.as_deref(),
            Some(PUBLIC_ID),
            "id_token must carry azp"
        );
        assert_eq!(
            access_claims.nonce, None,
            "access_token must not carry the nonce"
        );
        assert_eq!(
            id_claims.iss,
            issuer(&config),
            "iss must be the normalized issuer"
        );
    }

    fn env_with_secret_client(tag: &str, enabled: bool) -> Config {
        let mut config = portail_core::config::Config::global().clone();
        config.oidc_secrets_dir = format!(
            "{}/portail-confid-test-{tag}-{}",
            std::env::temp_dir().display(),
            std::process::id()
        );
        config.oidc_clients = vec![portail_core::config::ClientEntry {
            id: "confid".into(),
            secret_file: Some("confid_secret".into()),
            enabled,
            redirect_uris: vec![PUBLIC_REDIRECT.into()],
            scopes: vec!["openid".into()],
            custom_claims: Default::default(),
        }];
        std::fs::create_dir_all(&config.oidc_secrets_dir).unwrap();
        config.db_path = format!("{}/portail.db", config.oidc_secrets_dir);
        std::fs::write(
            crate::auth::secrets::secret_path(&config, "confid_secret"),
            "s3cr3t\n",
        )
        .unwrap();
        config
    }

    #[test]
    fn exchange_confidential_client_reads_secret_from_file() {
        let config = env_with_secret_client("ok", true);
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

        let err = exchange(
            &config,
            "confid",
            "wrong-secret",
            &code,
            PUBLIC_REDIRECT,
            None,
        )
        .unwrap_err();

        assert!(
            matches!(&err, AppError::InvalidClient(m) if m == "invalid_client"),
            "wrong secret must be invalid_client, got {err}"
        );
        let tokens = exchange(&config, "confid", "s3cr3t", &code, PUBLIC_REDIRECT, None)
            .expect("secret from the file must authenticate the client");
        assert!(!tokens.access_token.is_empty(), "tokens expected");
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }

    #[test]
    fn exchange_disabled_client_is_invalid_client() {
        let config = env_with_secret_client("disabled", false);
        let code = store::save_code(public_flow(Some("chal"))).unwrap();

        let err = exchange(&config, "confid", "s3cr3t", &code, PUBLIC_REDIRECT, None).unwrap_err();

        assert!(
            matches!(&err, AppError::InvalidClient(m) if m == "invalid_client"),
            "disabled client must be invalid_client, got {err}"
        );
        let _ = std::fs::remove_dir_all(&config.oidc_secrets_dir);
    }
}
