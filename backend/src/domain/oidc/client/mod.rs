pub mod errors;
pub use errors::ClientError;

use serde::Deserialize;

use crate::domain::oidc::{ClientId, ClientSecret, RedirectUri};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub secret: ClientSecret,
    pub redirect_uris: Vec<RedirectUri>,
    pub scopes: Vec<String>,
}

#[derive(Deserialize)]
struct RawClient {
    id: String,
    secret: String,
    redirect_uris: Vec<String>,
    #[serde(default)]
    scopes: Vec<String>,
}

impl Client {
    pub fn try_new(
        id: String,
        secret: String,
        redirect_uris: Vec<String>,
        scopes: Vec<String>,
    ) -> Result<Self, ClientError> {
        let id = ClientId::try_new(id).map_err(ClientError::Id)?;
        let secret = ClientSecret::new(secret);
        if redirect_uris.is_empty() {
            return Err(ClientError::NoRedirects);
        }
        let redirect_uris = redirect_uris
            .into_iter()
            .map(|r| RedirectUri::try_new(r).map_err(ClientError::Redirect))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            secret,
            redirect_uris,
            scopes,
        })
    }

    /// Reads the clients file content (JSON) — re-read on each request, so
    /// adding a client does not require a restart.
    pub fn parse_all(raw: &str) -> Result<Vec<Self>, ClientError> {
        let parsed: Vec<RawClient> =
            serde_json::from_str(raw).map_err(|e| ClientError::Json(e.to_string()))?;
        parsed
            .into_iter()
            .map(|c| Self::try_new(c.id, c.secret, c.redirect_uris, c.scopes))
            .collect()
    }

    pub fn find_in<'a>(clients: &'a [Client], id: &str) -> Option<&'a Client> {
        clients.iter().find(|c| c.id.as_str() == id)
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
        r#"[{"id":"portail-dev","secret":"s3cr3t","redirect_uris":["http://localhost:5173/callback"],"scopes":["openid","groups","email","profile"]}]"#.to_string()
    }

    #[test]
    fn parses_valid_file() {
        let clients = Client::parse_all(&raw()).unwrap();

        assert_eq!(clients.len(), 1, "one client expected");
        assert_eq!(clients[0].id.as_str(), "portail-dev");
        assert!(
            clients[0].grants("groups"),
            "declared scope should be granted"
        );
    }

    #[test]
    fn try_new_rejects_empty_redirects() {
        let err = Client::try_new("app1".into(), "s".into(), vec![], vec![]).unwrap_err();

        assert_eq!(
            err,
            ClientError::NoRedirects,
            "client without redirect_uris should fail"
        );
    }

    #[test]
    fn try_new_accepts_empty_secret_as_public_client() {
        let client = Client::try_new(
            "app1".into(),
            "".into(),
            vec!["https://app/cb".into()],
            vec![],
        );

        let client = client.expect("empty secret marks a public client, not an error");
        assert!(client.secret.is_public(), "empty secret must be public");
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
