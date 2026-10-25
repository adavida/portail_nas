pub mod errors;
pub use errors::ClientError;

use serde::Deserialize;

use crate::domain::oidc::{ClientId, RedirectUri};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub secret_file: Option<String>,
    pub enabled: bool,
    pub redirect_uris: Vec<RedirectUri>,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
struct RawClient {
    id: String,
    #[serde(default)]
    secret_file: Option<String>,
    #[serde(default)]
    enabled: bool,
    redirect_uris: Vec<String>,
    #[serde(default)]
    scopes: Vec<String>,
}

impl Client {
    pub fn try_new(
        id: String,
        secret_file: Option<String>,
        enabled: bool,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
    ) -> Result<Self, ClientError> {
        let id = ClientId::try_new(id).map_err(ClientError::Id)?;
        let secret_file = secret_file
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if redirect_uris.is_empty() {
            return Err(ClientError::NoRedirects);
        }
        let redirect_uris = redirect_uris
            .into_iter()
            .map(|r| RedirectUri::try_new(r).map_err(ClientError::Redirect))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            secret_file,
            enabled,
            redirect_uris,
            scopes,
        })
    }

    pub fn parse_all(raw: &str) -> Result<Vec<Self>, ClientError> {
        let parsed: Vec<RawClient> =
            serde_json::from_str(raw).map_err(|e| ClientError::Json(e.to_string()))?;
        parsed
            .into_iter()
            .map(|c| Self::try_new(c.id, c.secret_file, c.enabled, c.redirect_uris, c.scopes))
            .collect()
    }

    pub fn find_in<'a>(clients: &'a [Client], id: &str) -> Option<&'a Client> {
        clients.iter().find(|c| c.id.as_str() == id)
    }

    pub fn is_public(&self) -> bool {
        self.secret_file.is_none()
    }

    pub fn allows_redirect(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|r| r.as_str() == uri)
    }

    pub fn grants(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw() -> String {
        r#"[{"id":"portail-dev","secret_file":"portail-dev_secret","enabled":true,"redirect_uris":["http://localhost:5173/callback"],"scopes":["openid","groups","email","profile"]}]"#.to_string()
    }

    #[test]
    fn parses_valid_file() {
        let clients = Client::parse_all(&raw()).unwrap();

        assert_eq!(clients.len(), 1, "one client expected");
        assert_eq!(clients[0].id.as_str(), "portail-dev");
        assert!(clients[0].enabled, "enabled flag must be parsed");
        assert_eq!(
            clients[0].secret_file.as_deref(),
            Some("portail-dev_secret"),
            "secret file name must be parsed"
        );
        assert!(
            clients[0].grants("groups"),
            "declared scope should be granted"
        );
    }

    #[test]
    fn try_new_rejects_empty_redirects() {
        let err =
            Client::try_new("app1".into(), Some("s".into()), true, vec![], vec![]).unwrap_err();

        assert_eq!(
            err,
            ClientError::NoRedirects,
            "client without redirect_uris should fail"
        );
    }

    #[test]
    fn missing_or_blank_secret_file_marks_public_client() {
        let public =
            Client::parse_all(r#"[{"id":"app1","redirect_uris":["https://app/cb"],"scopes":[]}]"#)
                .unwrap()
                .remove(0);
        let blank = Client::try_new(
            "app2".into(),
            Some("   ".into()),
            true,
            vec!["https://app/cb".into()],
            vec![],
        )
        .unwrap();

        assert!(
            public.is_public(),
            "absent secret_file must mark a public client"
        );
        assert!(blank.is_public(), "blank secret_file trims to public");
    }

    #[test]
    fn enabled_defaults_to_false() {
        let clients = Client::parse_all(
            r#"[{"id":"app1","secret_file":"a","redirect_uris":["https://app/cb"]}]"#,
        )
        .unwrap();

        assert!(
            !clients[0].enabled,
            "client without explicit enabled must be disabled"
        );
    }

    #[test]
    fn parse_all_rejects_bad_json() {
        let err = Client::parse_all("not json").unwrap_err();

        assert!(
            matches!(err, ClientError::Json(_)),
            "bad json should map to Json error, got {err}"
        );
    }

    #[test]
    fn find_in_returns_matching_client() {
        let clients = Client::parse_all(&raw()).unwrap();

        let found = Client::find_in(&clients, "portail-dev");
        let missing = Client::find_in(&clients, "nope");

        assert!(found.is_some(), "declared client should be found");
        assert!(missing.is_none(), "unknown client should be None");
    }

    #[test]
    fn allows_redirect_exact_match() {
        let clients = Client::parse_all(&raw()).unwrap();
        let client = &clients[0];

        assert!(
            client.allows_redirect("http://localhost:5173/callback"),
            "declared redirect should be allowed"
        );
        assert!(
            !client.allows_redirect("http://evil.example.com/cb"),
            "undeclared redirect must be rejected"
        );
    }
}
