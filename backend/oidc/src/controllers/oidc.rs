use serde::Serialize;

use crate::{
    auth::{key, store, tokens},
    domain::oidc::{Client, ClientId},
};
use portail_core::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct OidcTestReport {
    pub client_id: ClientId,
    pub client_found: bool,
    pub signing_key_ok: bool,
    pub token_issued: bool,
    pub userinfo_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn test_client(client_id: ClientId) -> Result<OidcTestReport, AppError> {
    let env = portail_core::env::Env::global();
    let signing_key_ok = key::global().is_some();
    let clients = tokens::clients(env)?;
    let Some(client) = Client::find_in(&clients, client_id.as_str()) else {
        return Ok(OidcTestReport {
            client_id,
            client_found: false,
            signing_key_ok,
            token_issued: false,
            userinfo_ok: false,
            error: Some("client not found in clients file".into()),
        });
    };

    let flow = store::AuthFlow {
        client_id: client.id.as_str().to_string(),
        redirect_uri: client.redirect_uris[0].as_str().to_string(),
        state: None,
        nonce: None,
        scopes: client.scopes.clone(),
        challenge: None,
        sub: "oidc-selftest".into(),
        groups: vec!["test".into()],
        email: None,
    };
    let redirect_uri = flow.redirect_uri.clone();
    let code = store::save_code(flow).map_err(AppError::Internal)?;
    let base = OidcTestReport {
        client_id,
        client_found: true,
        signing_key_ok,
        token_issued: false,
        userinfo_ok: false,
        error: None,
    };
    let secret = crate::auth::secrets::stored_secret(env, client)
        .map_err(AppError::Internal)?
        .map(|s| s.as_str().to_string())
        .unwrap_or_default();
    let report = match tokens::exchange(
        env,
        base.client_id.as_str(),
        &secret,
        &code,
        &redirect_uri,
        None,
    ) {
        Ok(tokens) => OidcTestReport {
            userinfo_ok: tokens::verify_with_aud(
                env,
                &tokens.access_token,
                base.client_id.as_str(),
            )
            .is_ok(),
            token_issued: true,
            ..base
        },
        Err(e) => OidcTestReport {
            error: Some(e.to_string()),
            ..base
        },
    };
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::oidc::ClientId;

    #[test]
    fn test_unknown_client_reports_not_found() {
        portail_core::env::Env::ensure_init();
        crate::test_helpers::app::bootstrap_clients_file(portail_core::env::Env::global());
        let Ok(client_id) = ClientId::try_new("no-such-client".into()) else {
            return;
        };

        let report = test_client(client_id).unwrap();

        assert!(!report.client_found, "unknown client must be reported");
        assert!(!report.token_issued, "no token for unknown client");
    }
}
